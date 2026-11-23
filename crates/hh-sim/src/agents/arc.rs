//! The later acts (draft, 042) for the author: just enough play to get
//! through them, not to play them well. It engages only once the swarm is
//! revealed (the Deep Space Telescope, 1.9) and the calendar runs, so Act 1
//! play, the ladder and the balance reads never meet it. Player strategy: its
//! numbers are knobs' kind, not rules (docs/rulings/sim.md).

use hh_core::{Action, Fx};

use super::Player;

/// Later-act buildings and how many of each it keeps (strategy).
const BUILD: &[(&str, u32)] = &[("experiment_lab", 4), ("tachyon_condenser", 3), ("neodymium_mine", 3), ("dyson_foundry", 4)];

/// One look's later-act moves; nothing before the reveal.
pub fn act(p: &mut Player) {
    if !p.view().saga.revealed {
        return;
    }
    if p.view().saga.victory {
        // Done: idle the later acts' buildings (the labs eat the science
        // Act 1's research needs), so play goes on as Act 1's.
        let idle: Vec<String> = p
            .view()
            .buildings
            .iter()
            .filter(|b| b.active > 0 && BUILD.iter().any(|(id, _)| *id == b.id))
            .map(|b| b.id.to_string())
            .collect();
        for id in idle {
            let _ = p.apply(&Action::SetActive { building: id, active: 0 });
        }
        return;
    }
    let techs: Vec<String> =
        p.view().saga.techs.iter().filter(|t| t.blocker.is_none()).map(|t| t.id.to_string()).collect();
    for t in techs {
        let _ = p.apply(&Action::BuyEarthTech { tech: t });
    }
    for &(id, keep) in BUILD {
        for _ in 0..2 {
            let v = p.view();
            let Some(b) = v.buildings.iter().find(|b| b.id == id) else { break };
            if b.owned >= keep {
                break;
            }
            if p.apply(&Action::Buy { building: id.to_string() }).is_err() {
                // Short of land is the usual reason; one more try with land.
                if p.apply(&Action::BuyLand).is_err() || p.apply(&Action::Buy { building: id.to_string() }).is_err() {
                    break;
                }
            }
        }
    }
    let v = p.view();
    if v.saga.vault_open && v.saga.tachyon_here > Fx::ZERO {
        let _ = p.apply(&Action::Deposit { resource: "tachyon".into() });
    }
    let v = p.view();
    if v.quest.as_ref().is_some_and(|q| q.kind == "super_timeline") && v.saga.super_blocker.is_none() {
        let _ = p.apply(&Action::BeginSuperTimeline);
    }
    fund_dyson(p);
    let v = p.view();
    if v.saga.reset_plan_open && v.saga.reset_plan.is_empty() && !v.library.is_empty() {
        let _ = p.apply(&Action::SetResetPlan { program: 0, loadout: Some(0) });
    }
}

/// Funds the Dyson Sphere at the most a day on each of its costs.
fn fund_dyson(p: &mut Player) {
    let v = p.view();
    let Some(d) = v.projects.iter().find(|d| d.id == "dyson_sphere" && d.unlocked && !d.completed) else { return };
    let want: Vec<(String, Fx)> = d
        .costs
        .keys()
        .map(|res| (res.clone(), d.max_rates.get(res).copied().unwrap_or(Fx::ZERO)))
        .filter(|(res, max)| d.rates.get(res).is_none_or(|r| r < max))
        .collect();
    for (res, rate) in want {
        let _ = p.apply(&Action::SetProjectRate { project: "dyson_sphere".into(), resource: res, rate });
    }
}
