//! The content check (`Content::check`): every id a row names is real, and each row
//! carries only what its kind may (docs/rules/README.md, *Content*).

use super::*;

impl Content {
    /// Whether every id an event condition names (sub-objectives included) is real.
    pub(super) fn condition_ok(&self, c: &Condition) -> bool {
        match c {
            Condition::BuildingOwned { building, .. } => self.building(building).is_some(),
            Condition::ResourceCumulative { resource, .. } => self.resource(resource).is_some(),
            Condition::ResearchCompleted { research } => self.research_item(research).is_some(),
            Condition::AllOf { sub_objectives } => sub_objectives.iter().all(|s| self.condition_ok(&s.condition)),
            Condition::ShipmentCompleted { .. }
            | Condition::Immediate
            | Condition::Never
            | Condition::IdeologyRankAny { .. }
            | Condition::ProjectCompletedAny { .. }
            | Condition::FlagSet { .. }
            | Condition::DaysSurvived { .. }
            | Condition::CreditsEarned { .. }
            | Condition::Retirements { .. }
            | Condition::LaunchCredits { .. }
            | Condition::TradeCredits { .. }
            | Condition::ResearchCount { .. }
            | Condition::IdeologyRankEach { .. }
            | Condition::QuietDays { .. }
            | Condition::EarthScience { .. }
            | Condition::TimelinesEnded { .. }
            | Condition::SwarmRepelled
            | Condition::SuperTimeline
            | Condition::Victory => true,
            Condition::EarthTech { tech } => self.earth_tech.iter().any(|t| &t.id == tech),
            Condition::VaultHolds { resource, .. }
            | Condition::LaunchCargo { resource, .. }
            | Condition::LaunchFull { resource } => self.resource(resource).is_some(),
            Condition::ProjectCompleted { project } | Condition::ProjectEnabled { project } => {
                self.project(project).is_some()
            }
            Condition::IdeologyRank { axis, .. } => AXES.contains(&axis.as_str()),
            Condition::EventNotFired { event } => self.event(event).is_some(),
            Condition::AdversaryPresent { adversary } => {
                crate::modules::attention::ADVERSARIES.contains(&adversary.as_str())
            }
            Condition::QuestActive { quest } => self.event(quest).is_some_and(|e| e.is_chain()),
        }
    }

    /// Ids unique across all content; every reference names something real.
    pub fn check(&self) -> Result<(), String> {
        let mut ids = BTreeSet::new();
        let all = self
            .resources
            .iter()
            .map(|r| &r.id)
            .chain(self.buildings.iter().map(|b| &b.id))
            .chain(self.commands.iter().map(|b| &b.id))
            .chain(self.research.iter().map(|b| &b.id))
            .chain(self.events.iter().map(|b| &b.id))
            .chain(self.projects.iter().map(|b| &b.id))
            .chain(self.achievements.iter().map(|b| &b.id))
            .chain(self.rivals.iter().map(|b| &b.id))
            .chain(self.earth_tech.iter().map(|b| &b.id));
        for id in all {
            if !ids.insert(id.as_str()) {
                return Err(format!("content: duplicate id {id:?}"));
            }
        }
        let mut errors = Vec::new();
        let mut need = |ok: bool, what: String| {
            if !ok {
                errors.push(what);
            }
        };
        let res = |r: &str| self.resource(r).is_some();
        let bld = |b: &str| self.building(b).is_some();
        let quest = |q: &str| self.event(q).is_some_and(|e| e.is_chain());
        let research = |r: &str| self.research_item(r).is_some();
        let modifier = |k: &str| MODIFIER_KEYS.contains(&k);
        let axis = |a: &str| AXES.contains(&a);
        let req = |r: &Requirement| r.ids_ok(self);
        for b in &self.buildings {
            let w = format!("building {:?}", b.id);
            for r in b.cost.keys().chain(b.produces.keys()).chain(b.upkeep.keys()) {
                need(res(r), format!("{w} names unknown resource {r:?}"));
            }
            for (r, _) in b.produces.iter().filter(|(r, _)| b.upkeep.contains_key(*r)) {
                need(false, format!("{w} both makes and uses {r:?}"));
            }
            need(req(&b.requires), format!("{w}: bad requires"));
            need(b.ideology.as_deref().is_none_or(axis), format!("{w}: unknown ideology"));
            need(b.nudge.is_none() || b.ideology.is_some(), format!("{w}: nudge without ideology"));
            need(b.output_modifiers.iter().all(|k| modifier(k)), format!("{w}: unknown modifier"));
            need(b.cost_scaling >= Fx::ONE, format!("{w}: cost_scaling below 1"));
            for e in &b.effects {
                if let BuildingEffect::Grant { resource: r, .. } | BuildingEffect::Store { resource: r, .. } = e {
                    need(res(r), format!("{w} effect names unknown resource {r:?}"));
                }
            }
        }
        for c in &self.commands {
            let w = format!("command {:?}", c.id);
            need(c.ideology.as_deref().is_none_or(axis), format!("{w}: unknown ideology"));
            need(c.nudge.is_none() || c.ideology.is_some(), format!("{w}: nudge without ideology"));
            for r in c.costs.keys().chain(c.produces.keys()) {
                need(res(r), format!("{w} names unknown resource {r:?}"));
            }
            need(req(&c.requires), format!("{w}: bad requires"));
            need(c.scale_modifier.as_deref().is_none_or(modifier), format!("{w}: unknown modifier"));
            match &c.effect {
                Some(CommandEffect::Promote { good }) => need(res(good), format!("{w}: unknown good")),
                Some(CommandEffect::Fund { axis: a }) => need(axis(a), format!("{w}: unknown axis")),
                Some(CommandEffect::Neutralize { adversary }) => need(
                    crate::modules::attention::ADVERSARIES.contains(&adversary.as_str()),
                    format!("{w}: unknown adversary {adversary:?}"),
                ),
                Some(CommandEffect::Overclock { target, .. }) => need(
                    self.buildings.iter().any(|b| b.category == *target),
                    format!("{w}: overclock target {target:?} is no building category"),
                ),
                _ => {}
            }
        }
        for r in &self.research {
            let w = format!("research {:?}", r.id);
            need(r.requires.as_deref().is_none_or(research), format!("{w}: bad requires"));
            need(r.visible_when.ids_ok(self), format!("{w}: bad visible_when"));
            need(r.reveal_when.ids_ok(self), format!("{w}: bad reveal_when"));
            if let Some(ResearchEffect::CommandCostOverride { command, resource, .. }) = &r.effect {
                need(self.command(command).is_some() && res(resource), format!("{w}: bad override"));
            }
            if let Some(ResearchEffect::Modifier { key, .. }) = &r.effect {
                need(modifier(key), format!("{w}: unknown modifier {key:?}"));
            }
        }
        for e in &self.events {
            let w = format!("event {:?}", e.id);
            match &e.trigger {
                Trigger::QuestComplete { quest: q } => need(quest(q), format!("{w}: trigger names unknown quest {q:?}")),
                Trigger::Flag { beside: q, .. } => need(quest(q), format!("{w}: trigger beside unknown quest {q:?}")),
                _ => {}
            }
            need(
                e.category == "story" || e.category == "side" || e.category == "ongoing",
                format!("{w}: bad category {:?}", e.category),
            );
            // Sub-objectives latch only on the chain's active quest.
            need(!(e.is_side() && matches!(e.condition, Condition::AllOf { .. })), format!("{w}: a side quest can't be all_of"));
            need(self.condition_ok(&e.condition), format!("{w}: condition names something unknown"));
            for u in &e.unlocks {
                need(u.ids_ok(self), format!("{w}: bad unlock"));
            }
            // A quest's reward line lists only what completing it grants, so it may have none.
            let mechanics_ok = if !e.is_one_off() {
                !e.goal.is_empty()
                    && e.reward != "None."
                    && e.summary.is_empty()
                    && e.effect.is_empty()
            } else {
                e.goal.is_empty() && e.reward.is_empty() && (e.silent || !e.summary.is_empty())
            };
            need(mechanics_ok, format!("{w}: a quest has a goal and maybe a reward, never \"None.\" (no summary or effect); a one-off event a summary"));
            let story_ok = e.payoff.is_empty() && (e.story.is_empty() || e.once) || e.is_chain();
            need(story_ok, format!("{w}: story and payoff are for chain quests (a story also for a once one-off)"));
            need(!e.once || e.is_one_off(), format!("{w}: once is for one-off events"));
            let modifier_ok = !e.is_one_off() || !e.unlocks.iter().any(|u| matches!(u, Unlock::Modifier { .. }));
            need(modifier_ok, format!("{w}: a modifier unlock is a quest's (read while completed)"));
            if let Some(p) = &e.picture {
                let slug = !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
                need(!e.is_one_off() && slug, format!("{w}: picture is a file stem, on quests"));
            }
            need(
                e.on_start
                    .iter()
                    .all(|u| matches!(u, Unlock::Grant { resource, amount } if res(resource) && *amount > Fx::ZERO))
                    && (e.on_start.is_empty() || e.is_chain()),
                format!("{w}: on_start takes grants, on chain quests"),
            );
            if let Some(goods) = &e.pad_goods {
                need(e.is_chain() && !goods.is_empty() && goods.iter().all(|g| res(g)), format!("{w}: bad pad_goods"));
            }
        }
        for p in &self.projects {
            let w = format!("project {:?}", p.id);
            need(p.costs.keys().all(|r| res(r)), format!("{w}: unknown cost resource"));
            need(p.min_days.is_none_or(|d| d >= 1), format!("{w}: min_days below 1"));
            need(p.repeat.as_ref().is_none_or(|x| x.cost_growth >= Fx::ONE), format!("{w}: cost_growth below 1"));
            need(
                self.condition_ok(&p.unlock) && !matches!(p.unlock, Condition::AllOf { .. }),
                format!("{w}: bad unlock"),
            );
            for reward in &p.rewards {
                match reward {
                    ProjectReward::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                    ProjectReward::StartingBuildings { buildings } => {
                        need(buildings.keys().all(|b| bld(b)), format!("{w}: unknown building"))
                    }
                    ProjectReward::BoredomModifiers { command_boredom, .. } => need(
                        command_boredom.keys().all(|c| self.command(c).is_some()),
                        format!("{w}: unknown command"),
                    ),
                    ProjectReward::StartingResearch { research } => need(
                        !research.is_empty() && research.iter().all(|x| self.research_item(x).is_some()),
                        format!("{w}: unknown research"),
                    ),
                    ProjectReward::ResearchDiscount { mult, category } => need(
                        *mult > Fx::ZERO
                            && *mult <= Fx::ONE
                            && category.as_ref().is_none_or(|c| self.research.iter().any(|x| x.category == *c)),
                        format!("{w}: research_discount mult outside (0, 1] or unknown category"),
                    ),
                    _ => {}
                }
            }
        }
        for a in &self.achievements {
            let w = format!("achievement {:?}", a.id);
            if let AchievementCondition::ProducedInADay { resource: r, .. }
            | AchievementCondition::Stockpile { resource: r, .. }
            | AchievementCondition::ConsumedInADay { resource: r, .. } = &a.condition
            {
                need(res(r), format!("{w}: unknown resource {r:?}"));
            }
            match &a.reward {
                AchievementReward::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                AchievementReward::BonusBuildings { building, .. } => {
                    need(bld(building), format!("{w}: unknown building"))
                }
            }
        }
        // Every flag a building or project waits on is set by something.
        let mut set: BTreeSet<&str> = self.commands.iter().filter_map(|c| c.sets_flag.as_deref()).collect();
        for p in &self.projects {
            for reward in &p.rewards {
                if let ProjectReward::Flag { flag, .. } = reward {
                    set.insert(flag);
                }
            }
        }
        for e in &self.events {
            for u in &e.unlocks {
                if let Unlock::SetFlag { flag } = u {
                    set.insert(flag);
                }
            }
        }
        // `if` opens on a flag content must set (programs.md, *Unlock*).
        let flag = crate::modules::programs::IF_FLAG;
        need(set.contains(flag), format!("programs: nothing sets flag {flag:?}"));
        // The engine sets this one itself (programs.md, *Unlock*).
        set.insert(crate::modules::programs::LOGIC_FLAG);
        for b in &self.buildings {
            if let Requirement::Flag { flag } = &b.requires {
                need(set.contains(flag.as_str()), format!("building {:?}: nothing sets flag {flag:?}", b.id));
            }
        }
        for c in &self.commands {
            if let Requirement::Flag { flag } = &c.requires {
                need(set.contains(flag.as_str()), format!("command {:?}: nothing sets flag {flag:?}", c.id));
            }
        }
        for p in &self.projects {
            if let Condition::FlagSet { flag } = &p.unlock {
                need(set.contains(flag.as_str()), format!("project {:?}: nothing sets flag {flag:?}", p.id));
            }
        }
        for r in &self.rivals {
            need(res(&r.good), format!("rival {:?}: unknown good", r.id));
            need(r.interval_min <= r.interval_max, format!("rival {:?}: interval", r.id));
        }
        // Nav tiles: unique ids, real buildings, and every panel an event
        // enables has the tile it shows.
        let mut tiles = BTreeSet::new();
        for t in &self.nav {
            need(tiles.insert(t.id.as_str()), format!("nav {:?}: duplicate id", t.id));
            if let NavCondition::BuildingKnown { building } = &t.visible {
                need(bld(building), format!("nav {:?}: unknown building", t.id));
            }
        }
        for e in &self.events {
            for u in &e.unlocks {
                if let Unlock::EnableNavPanel { panel } = u {
                    let shown = self
                        .nav
                        .iter()
                        .any(|t| matches!(&t.visible, NavCondition::Panel { panel: p } if p == panel));
                    need(shown, format!("event {:?}: no nav tile shows panel {panel:?}", e.id));
                }
            }
        }
        // Acts: numbered 1, 2, ... in order, the first opening with the first
        // chain quest, each opening with a chain quest later than the one before.
        {
            let chain: Vec<&str> = self.events.iter().filter(|e| e.is_chain()).map(|e| e.id.as_str()).collect();
            let mut last: Option<usize> = None;
            for (i, a) in self.acts.iter().enumerate() {
                let w = format!("act {}", a.act);
                need(a.act as usize == i + 1, format!("{w}: acts number 1, 2, ... in order"));
                let at = chain.iter().position(|q| *q == a.opens_with);
                need(at.is_some(), format!("{w}: opens_with is not a chain quest"));
                if let Some(at) = at {
                    need(i > 0 || at == 0, format!("{w}: the first act opens with the first chain quest"));
                    need(last.is_none_or(|l| at > l), format!("{w}: opens before the act before it"));
                    last = Some(at);
                }
            }
            need(!self.acts.is_empty(), "acts.json: no acts".to_string());
            // Messages: exactly the ids the view reads, once each.
            let ids: Vec<&str> = self.messages.iter().map(|m| m.id.as_str()).collect();
            let mut want = MESSAGES.to_vec();
            let mut got = ids.clone();
            want.sort_unstable();
            got.sort_unstable();
            need(got == want, format!("messages.json: ids {ids:?}, want each of {MESSAGES:?} once"));
            // Story draws each side quest off the chain quest that starts it
            // (or that a flag-triggered one hangs beside).
            for e in self.events.iter().filter(|e| e.is_side()) {
                let off_chain = match &e.trigger {
                    Trigger::QuestComplete { quest } | Trigger::Flag { beside: quest, .. } => chain.contains(&quest.as_str()),
                    _ => false,
                };
                need(off_chain, format!("side quest {:?}: its trigger must be a chain quest's completion or a flag beside one", e.id));
            }
        }
        // The ids hh-core names in code (docs/rules/README.md, *Content*).
        {
            use crate::modules::{career, economy as e, shipments as s};
            for r in [e::ENERGY, e::CREDITS, e::LAND, e::PROCESSORS, e::BOREDOM, e::SCIENCE, s::PROPELLANT] {
                need(res(r), format!("engine resource {r:?} missing from content"));
            }
            need(bld(s::LAUNCH_PAD), format!("engine building {:?} missing from content", s::LAUNCH_PAD));
            let retire = self
                .nav
                .iter()
                .any(|t| matches!(&t.visible, NavCondition::Panel { panel } if panel == career::RETIREMENT_PANEL));
            need(retire, format!("no nav tile shows the engine's panel {:?}", career::RETIREMENT_PANEL));
        }
        for t in &self.earth_tech {
            let w = format!("earth tech {:?}", t.id);
            need(["timeline", "next_timeline", "forever"].contains(&t.scope.as_str()), format!("{w}: bad scope"));
            need(t.requires.as_deref().is_none_or(quest), format!("{w}: unknown quest"));
            for e in &t.effects {
                match e {
                    TechEffect::Modifier { key, .. } => need(modifier(key), format!("{w}: unknown modifier")),
                    TechEffect::StartResources { resources } => {
                        need(resources.keys().all(|r| res(r)), format!("{w}: unknown resource"))
                    }
                    TechEffect::Unlock { feature } => need(
                        crate::modules::timeline::FEATURES.contains(&feature.as_str()),
                        format!("{w}: unknown feature"),
                    ),
                    TechEffect::SwarmDelay { .. } => {}
                }
            }
        }
        for r in [crate::modules::timeline::EARTH_SCIENCE, crate::modules::timeline::TACHYON] {
            need(res(r), format!("engine resource {r:?} missing from content"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("content: {}", errors.join("; ")))
        }
    }
}
