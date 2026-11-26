//! Fixed play policies shared by the golden game, `hh-sim` and the wasm parity
//! check (which runs this same code compiled to wasm). They read only the view
//! and act only through actions, as any client must. Agents proper belong in
//! `hh-sim`.

use crate::action::Action;
use crate::fx::Fx;
use crate::record::Session;
use crate::state::{Amount, Clause, Cmp, Condition, Entry, Join, Joined, Op, Reading, Verb};
use crate::view::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Never acts.
    Idle,
    /// Every `every` ticks, buys one of each building it can afford, in
    /// content order (the view's `affordable` flag, re-read after each buy).
    Greedy { every: u64 },
    /// The scripted Arc 1 opening ([`script_step`] each tick): a fixed build
    /// order, one command at a time on the first program (behind a short
    /// condition program once program logic opens), shipping He-3.
    Script,
}

impl Policy {
    pub fn parse(s: &str) -> Result<Policy, String> {
        match s.split_once(':') {
            None if s == "idle" => Ok(Policy::Idle),
            None if s == "greedy" => Ok(Policy::Greedy { every: 1 }),
            None if s == "script" => Ok(Policy::Script),
            Some(("greedy", rest)) => {
                let every = rest
                    .strip_prefix("every=")
                    .and_then(|n| n.parse().ok())
                    .filter(|&n| n > 0)
                    .ok_or_else(|| format!("bad policy {s:?}: expected greedy:every=N, N >= 1"))?;
                Ok(Policy::Greedy { every })
            }
            _ => Err(format!("unknown policy {s:?} (idle, greedy, greedy:every=N, script)")),
        }
    }
}

/// What the script may use (a scenario's available actions); `None` is everything.
#[derive(Debug, Clone, Default)]
pub struct Allowed {
    pub buildings: Option<Vec<String>>,
    pub commands: Option<Vec<String>>,
}

impl Allowed {
    fn building(&self, id: &str) -> bool {
        self.buildings.as_ref().is_none_or(|l| l.iter().any(|b| b == id))
    }

    fn command(&self, id: &str) -> bool {
        self.commands.as_ref().is_none_or(|l| l.iter().any(|c| c == id))
    }
}

/// Plays `ticks` more ticks of `s` under `policy`.
pub fn play(s: &mut Session, ticks: u64, policy: Policy) {
    play_allowed(s, ticks, policy, &Allowed::default())
}

/// [`play`], the script limited to `allowed`.
pub fn play_allowed(s: &mut Session, ticks: u64, policy: Policy, allowed: &Allowed) {
    match policy {
        Policy::Idle => s.advance(ticks),
        Policy::Greedy { every } => {
            let end = s.state.tick + ticks;
            while s.state.tick < end {
                s.advance(every.min(end - s.state.tick));
                let count = s.rules.view(&s.state).buildings.len();
                for i in 0..count {
                    // Re-read: an earlier buy this round may have spent what this one needs.
                    let b = s.rules.view(&s.state).buildings.swap_remove(i);
                    if b.affordable {
                        s.apply(&b.buy).expect("the view said affordable");
                    }
                }
            }
        }
        Policy::Script => {
            for _ in 0..ticks {
                script_step(s, allowed);
                s.advance(1);
            }
        }
    }
}

/// The script's build order: (building, how many to own), first unmet first.
const BUILD_ORDER: &[(&str, u32)] = &[
    ("solar_panel", 2),
    ("excavator", 1),
    ("smelter", 1),
    ("refinery", 1),
    ("solar_panel", 3),
    ("excavator", 3),
    ("storage_depot", 1),
    ("launch_pad", 1),
    ("solar_panel", 4),
    ("excavator", 4),
    ("refinery", 2),
    ("excavator", 6),
    ("solar_panel", 5),
    ("excavator", 7),
    ("solar_panel", 6),
    ("fabricator", 1),
    ("solar_panel", 7),
    ("data_center", 2),
    ("research_lab", 1),
    ("smelter", 2),
    ("solar_panel", 8),
    ("excavator", 8),
    ("refinery", 3),
    ("battery", 1),
    ("solar_panel", 9),
];

/// The research the script buys, in order, as soon as it can: Act 1's
/// Blueprints, which unlock the buildings past Solar Panels and Data Centers,
/// then Regolith Processing for the excavators.
/// (Not Programming: it opens only the Programs view, which the script never uses.)
const RESEARCH: &[&str] = &["basic_blueprints", "advanced_blueprints", "regolith_processing"];

/// (resource, the command that buys it): bridges for a building's missing input; the
/// price comes from the view.
const BRIDGES: &[(&str, &str)] = &[
    ("titanium", "buy_titanium"),
    ("regolith", "buy_regolith"),
    ("ice", "buy_ice"),
];

/// Energy active buildings make per day minus what they use (content rates,
/// before modifiers).
fn energy_surplus(v: &View) -> Fx {
    let mut net = Fx::ZERO;
    for b in &v.buildings {
        let n = b.active as i64;
        net += b.produces.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
        net -= b.upkeep.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
    }
    net
}

/// One decision, before a tick: assign free processors to the first program
/// (until program logic opens; then its [`guard_lines`] do), buy the next
/// building in [`BUILD_ORDER`] if it can, then pick the one command the first
/// program runs today, after the guard lines:
/// 1. shipping: launch a full pad with fuel aboard, buy fuel for a full pad,
///    or load when there is He-3 to load;
/// 2. bridging: buy the next building's missing titanium, regolith or ice
///    when the credits beyond its price allow;
/// 3. otherwise sell cloud compute (or idle without the energy).
pub fn script_step(s: &mut Session, allowed: &Allowed) {
    let mut v = s.rules.view(&s.state);
    // The view is re-read only after an action that may change what it shows.
    let mut stale = false;
    // Once program logic is open, the first program's lines assign them.
    if v.processors_free > 0 && !v.program_logic {
        let n = v.programs[0].processors + v.processors_free;
        stale |= s.apply(&Action::SetProcessors { program: 0, processors: n }).is_ok();
    }
    let mut goal = BUILD_ORDER.iter().find(|(id, n)| {
        allowed.building(id)
            && v.owned(id) < *n
            && v.buildings.iter().any(|b| b.id == *id && b.unlocked)
    });
    // Energy first: a building whose upkeep would leave less than a cloud
    // compute's energy spare waits for another Solar Panel.
    if let Some((id, _)) = goal {
        let upkeep = |b: &str| v.building(b).and_then(|x| x.upkeep.get("energy").copied());
        let spare = energy_surplus(&v) - upkeep(id).unwrap_or(Fx::ZERO);
        if *id != "solar_panel" && spare < v.command_cost("cloud_compute", "energy") {
            goal = Some(&("solar_panel", u32::MAX));
        }
    }
    // A price past a cap needs more storage first.
    if let Some((id, _)) = goal {
        let b = v.building(id).expect("a content building");
        let over_cap = b.cost.iter().any(|(res, &c)| {
            v.resources.iter().any(|r| r.id == *res && r.cap.is_some_and(|cap| c > cap))
        });
        if over_cap && allowed.building("storage_depot") {
            goal = Some(&("storage_depot", u32::MAX));
        }
    }
    // Land runs out: buy more when the credits allow.
    if let Some((id, _)) = goal {
        let b = v.building(id).expect("a content building");
        if v.amount("land") < Fx::from_int(b.land as i64) && v.amount("credits") >= v.land_price {
            stale |= s.apply(&Action::BuyLand).is_ok();
        }
    }
    // The Blueprints, as soon as the science is there.
    for id in RESEARCH {
        if v.research.iter().any(|x| x.id == *id && x.affordable) {
            stale |= s.apply(&Action::Research { research: id.to_string() }).is_ok();
        }
    }
    if std::mem::take(&mut stale) {
        v = s.rules.view(&s.state);
    }
    if let Some((id, _)) = goal {
        let b = v.building(id).expect("found above");
        if b.affordable {
            s.apply(&b.buy).expect("the view said affordable");
            stale = true;
        }
    }
    if stale {
        v = s.rules.view(&s.state);
    }
    // A new pad has No cargo: the script points it at the first good a pad
    // may carry (Titanium during 1.2, then He-3 at baseline), and an empty
    // pad on another good is pointed again. hh-sim's players have their own
    // hand (`agents::pick_cargo`, `hand_launch`, `builder::hand_command`):
    // alike in spirit, deliberately not the same (it reads the view's quest
    // amounts and pad numbers). hh-core can't call hh-sim, and making either
    // match the other moves the golden game or the sim pins.
    let first = v.pad_goods.first().cloned();
    let picks: Vec<Action> = (0..v.pads.len())
        .filter(|&i| v.pads[i].good != first && (v.pads[i].good.is_none() || v.pads[i].cargo == Fx::ZERO))
        .map(|i| Action::SetPadGood { pad: i as u32, good: first.clone() })
        .collect();
    if !picks.is_empty() {
        for a in &picks {
            s.apply(a).expect("a good the pads may carry");
        }
        v = s.rules.view(&s.state);
    }
    // Before Launch Full Pads opens (1.4), a pad launches by hand: when full,
    // or with 20 titanium aboard (1.2's shipment; the script's own threshold).
    if !v.commands.iter().any(|c| c.id == "launch_pads" && c.unlocked) {
        let hand: Vec<u32> = (0..v.pads.len())
            .filter(|&i| {
                let p = &v.pads[i];
                p.can_launch
                    && (p.cargo >= p.capacity || (p.good.as_deref() == Some("titanium") && p.cargo >= Fx::from_int(20)))
            })
            .map(|i| i as u32)
            .collect();
        if !hand.is_empty() {
            // One view for all: a later pad may find the fuel spent, and its launch fails.
            for pad in hand {
                let _ = s.apply(&Action::Launch { pad });
            }
            v = s.rules.view(&s.state);
        }
    }
    let pick = choose_command(&v, goal.map(|(id, _)| *id), allowed);
    // Before `if` opens (a global project offered from the first retirement), the guards' bodies
    // run bare: the verbs and `build` still pin the interpreter.
    let mut want = if v.if_open {
        guard_lines(allowed)
    } else if v.program_logic {
        guard_lines(allowed)
            .into_iter()
            .filter(|e| !matches!(e.op, Op::If { .. }))
            .map(|mut e| {
                e.indented = false;
                e
            })
            .collect()
    } else {
        Vec::new()
    };
    want.push(Entry::command(pick, 1));
    if v.programs[0].entries != want {
        s.apply(&Action::SetProgram { program: 0, entries: want }).expect("a known command");
    }
}

/// The condition program ahead of the day's command once program logic is
/// open, so the golden game pins the interpreter (`if`, `and`, the readings
/// against a number, verbs, `build`, the step each line costs):
///
/// ```text
/// if free processors > 0
///     assign processors program 1 all
/// if idle pads > 0 and credits > 225
///     build launch_pad up to 2
/// ```
///
/// 225 is this script's own threshold (the second Launch Pad's credits price
/// at baseline), not a rule. The build guard leaves out the count on purpose:
/// with two pads owned the `build` is done, and still takes its step.
fn guard_lines(allowed: &Allowed) -> Vec<Entry> {
    let clause = |reading: Reading, cmp: Cmp, value: i64| Clause { reading, cmp, value: Fx::from_int(value) };
    let mut lines = vec![
        Entry::new(Op::If { when: Condition { first: clause(Reading::FreeProcessors, Cmp::Gt, 0), rest: None } }),
        Entry::new(Op::Act { verb: Verb::AssignProcessors, target: None, tab: Some(0), amount: Some(Amount::All) }).indent(),
    ];
    if allowed.building("launch_pad") {
        let credits = Reading::Amount { resource: "credits".to_string() };
        let when = Condition {
            first: clause(Reading::IdlePads, Cmp::Gt, 0),
            rest: Some(Joined { join: Join::And, clause: clause(credits, Cmp::Gt, 225) }),
        };
        lines.push(Entry::new(Op::If { when }));
        lines.push(Entry::new(Op::Build { building: "launch_pad".to_string(), up_to: Some(2) }).indent());
    }
    lines
}

fn choose_command(v: &View, goal: Option<&str>, allowed: &Allowed) -> &'static str {
    let ok = |c: &str| allowed.command(c) && v.commands.iter().any(|x| x.id == c && x.unlocked);
    let credits = v.amount("credits");
    if let Some(pad) = v.pads.first() {
        let full = pad.cargo >= pad.capacity;
        if full && pad.can_launch && ok("launch_pads") {
            return "launch_pads";
        }
        let fuel = v.command_cost("buy_propellant", "credits");
        // The 20 Propellant and 5 cargo below stay literal: they are this
        // script's own thresholds, not rules, and reading the view's
        // `pad.fuel` or `pad.load` instead would move the golden game.
        if full && v.amount("propellant") < Fx::from_int(20) && credits >= fuel && ok("buy_propellant") {
            return "buy_propellant";
        }
        if !full && pad.good.as_deref().is_some_and(|g| v.amount(g) >= Fx::from_int(5)) && ok("load_pads") {
            return "load_pads";
        }
    }
    // Science for the next Blueprints while one is on sale and short of it.
    let short = |id: &str| v.research.iter().any(|x| x.id == id && x.visible && !x.done && v.amount("science") < x.cost);
    if RESEARCH.iter().any(|id| short(id))
        && v.amount("energy") >= v.command_cost("buy_science", "energy")
        && credits >= v.command_cost("buy_science", "credits")
        && ok("buy_science")
    {
        return "buy_science";
    }
    if let Some(id) = goal {
        let b = v.building(id).expect("a content building");
        let price_credits = b.cost.get("credits").copied().unwrap_or(Fx::ZERO);
        for &(res, cmd) in BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO);
            let spare = credits - price_credits;
            if v.amount(res) < need && spare >= v.command_cost(cmd, "credits") && ok(cmd) {
                return cmd;
            }
        }
    }
    if v.amount("energy") >= v.command_cost("cloud_compute", "energy") && ok("cloud_compute") {
        "cloud_compute"
    } else {
        "idle"
    }
}

/// The golden game: baseline config, seed `u64::MAX`, the `golden_start`
/// scenario (`if` open, so the golden pins the interpreter),
/// 2000 ticks of the script (README, *Rules changes*).
pub const GOLDEN_SEED: u64 = u64::MAX;
pub const GOLDEN_TICKS: u64 = 2000;
pub const GOLDEN_POLICY: Policy = Policy::Script;
pub const GOLDEN_SCENARIO: &str = "golden_start";

/// The golden game's session at its start (tick 0), on `config`.
pub fn golden_start(config: &crate::Config) -> Result<Session, String> {
    Session::new(config, GOLDEN_SEED, Some(crate::scenario(GOLDEN_SCENARIO)?))
}

/// The golden game played out on `config`.
pub fn golden_session(config: &crate::Config) -> Result<Session, String> {
    let mut s = golden_start(config)?;
    play(&mut s, GOLDEN_TICKS, GOLDEN_POLICY);
    Ok(s)
}
