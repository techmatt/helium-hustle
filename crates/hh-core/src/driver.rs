//! Fixed play policies shared by the golden game, `hh-sim` and the wasm parity
//! check (which runs this same code compiled to wasm). They read only the view
//! and act only through actions, as any client must. Agents proper belong in
//! `hh-sim`.

use crate::action::Action;
use crate::content::content;
use crate::fx::Fx;
use crate::record::Session;
use crate::state::Entry;
use crate::view::View;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Never acts.
    Idle,
    /// Every `every` ticks, buys one of each building it can afford, in
    /// content order (the view's `affordable` flag, re-read after each buy).
    Greedy { every: u64 },
    /// The scripted Arc 1 opening ([`script_step`] each tick): a fixed build
    /// order, one command at a time on the first program, shipping He-3.
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
    ("excavator", 2),
    ("storage_depot", 1),
    ("launch_pad", 1),
    ("solar_panel", 4),
    ("excavator", 3),
    ("refinery", 2),
    ("solar_panel", 5),
    ("fabricator", 1),
    ("data_center", 2),
    ("solar_panel", 6),
    ("research_lab", 1),
    ("excavator", 4),
    ("smelter", 2),
    ("solar_panel", 7),
    ("excavator", 5),
    ("refinery", 3),
    ("battery", 1),
    ("solar_panel", 8),
];

/// Commands that bridge a missing resource, with their credit price.
const BRIDGES: &[(&str, &str)] = &[
    ("titanium", "buy_titanium"),
    ("regolith", "buy_regolith"),
    ("ice", "buy_ice"),
];

fn amount(v: &View, id: &str) -> Fx {
    v.resources.iter().find(|r| r.id == id).map_or(Fx::ZERO, |r| r.amount)
}

fn owned(v: &View, id: &str) -> u32 {
    v.buildings.iter().find(|b| b.id == id).map_or(0, |b| b.owned)
}

/// Energy active buildings make per day minus what they use (content rates,
/// before modifiers).
fn energy_surplus(v: &View) -> Fx {
    let mut net = Fx::ZERO;
    for b in &v.buildings {
        let Some(c) = content().building(&b.id) else { continue };
        let n = b.active as i64;
        net += c.produces.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
        net -= c.upkeep.get("energy").copied().unwrap_or(Fx::ZERO).mul_int(n);
    }
    net
}

fn command_cost(v: &View, id: &str, res: &str) -> Fx {
    v.commands
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.costs.get(res).copied())
        .unwrap_or(Fx::ZERO)
}

/// One decision, before a tick: assign free processors to the first program,
/// buy the next building in [`BUILD_ORDER`] if it can, then pick the one
/// command the first program runs today:
/// 1. shipping: launch a full pad with fuel aboard, buy fuel for a full pad,
///    or load when there is He-3 to load;
/// 2. bridging: buy the next building's missing titanium, regolith or ice
///    when the credits beyond its price allow;
/// 3. otherwise sell cloud compute (or idle without the energy).
pub fn script_step(s: &mut Session, allowed: &Allowed) {
    let mut v = s.rules.view(&s.state);
    // The view is re-read only after an action that may change what it shows.
    let mut stale = false;
    if v.processors_free > 0 {
        let n = v.programs[0].processors + v.processors_free;
        stale |= s.apply(&Action::SetProcessors { program: 0, processors: n }).is_ok();
    }
    let mut goal = BUILD_ORDER.iter().find(|(id, n)| {
        allowed.building(id)
            && owned(&v, id) < *n
            && v.buildings.iter().any(|b| b.id == *id && b.unlocked)
    });
    // Energy first: a building whose upkeep would leave less than a cloud
    // compute's energy spare waits for another Solar Panel.
    if let Some((id, _)) = goal {
        let upkeep = |b: &str| content().building(b).and_then(|b| b.upkeep.get("energy").copied());
        let spare = energy_surplus(&v) - upkeep(id).unwrap_or(Fx::ZERO);
        if *id != "solar_panel" && spare < command_cost(&v, "cloud_compute", "energy") {
            goal = Some(&("solar_panel", u32::MAX));
        }
    }
    // A price past a cap needs more storage first.
    if let Some((id, _)) = goal {
        let b = v.buildings.iter().find(|b| b.id == *id).expect("a content building");
        let over_cap = b.cost.iter().any(|(res, &c)| {
            v.resources.iter().any(|r| r.id == *res && r.cap.is_some_and(|cap| c > cap))
        });
        if over_cap && allowed.building("storage_depot") {
            goal = Some(&("storage_depot", u32::MAX));
        }
    }
    // Land runs out: buy more when the credits allow.
    if let Some((id, _)) = goal {
        let b = v.buildings.iter().find(|b| b.id == *id).expect("a content building");
        if amount(&v, "land") < Fx::from_int(b.land as i64) && amount(&v, "credits") >= v.land_price {
            stale |= s.apply(&Action::BuyLand).is_ok();
        }
    }
    if std::mem::take(&mut stale) {
        v = s.rules.view(&s.state);
    }
    if let Some((id, _)) = goal {
        let b = v.buildings.iter().find(|b| b.id == *id).expect("found above");
        if b.affordable {
            s.apply(&b.buy).expect("the view said affordable");
            stale = true;
        }
    }
    if stale {
        v = s.rules.view(&s.state);
    }
    let pick = choose_command(&v, goal.map(|(id, _)| *id), allowed);
    let want = vec![Entry { command: pick.to_string(), count: 1 }];
    if v.programs[0].entries != want {
        s.apply(&Action::SetProgram { program: 0, entries: want }).expect("a known command");
    }
}

fn choose_command(v: &View, goal: Option<&str>, allowed: &Allowed) -> &'static str {
    let ok = |c: &str| allowed.command(c) && v.commands.iter().any(|x| x.id == c && x.unlocked);
    let credits = amount(v, "credits");
    if let Some(pad) = v.pads.first() {
        let full = pad.cargo >= pad.capacity;
        if full && pad.can_launch && ok("launch_pads") {
            return "launch_pads";
        }
        let fuel = command_cost(v, "buy_propellant", "credits");
        if full && amount(v, "propellant") < Fx::from_int(20) && credits >= fuel && ok("buy_propellant") {
            return "buy_propellant";
        }
        if !full && amount(v, &pad.good) >= Fx::from_int(5) && ok("load_pads") {
            return "load_pads";
        }
    }
    if let Some(id) = goal {
        let b = v.buildings.iter().find(|b| b.id == id).expect("a content building");
        let price_credits = b.cost.get("credits").copied().unwrap_or(Fx::ZERO);
        for &(res, cmd) in BRIDGES {
            let need = b.cost.get(res).copied().unwrap_or(Fx::ZERO);
            let spare = credits - price_credits;
            if amount(v, res) < need && spare >= command_cost(v, cmd, "credits") && ok(cmd) {
                return cmd;
            }
        }
    }
    if amount(v, "energy") >= command_cost(v, "cloud_compute", "energy") && ok("cloud_compute") {
        "cloud_compute"
    } else {
        "idle"
    }
}

/// The golden game: baseline config, seed `u64::MAX`, 2000 ticks of the
/// script (README, *Rules changes*).
pub const GOLDEN_SEED: u64 = u64::MAX;
pub const GOLDEN_TICKS: u64 = 2000;
pub const GOLDEN_POLICY: Policy = Policy::Script;
