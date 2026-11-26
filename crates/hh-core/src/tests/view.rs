//! The view: its legality flags, pads, nav tiles, the story timeline and event texts.

use super::support::{buy, own};
use super::{baseline, f, rules, set};
use crate::driver::{play, Policy};
use crate::{Action, Fx, GameState, Session};

#[test]
fn the_views_can_apply_flags_agree_with_is_legal() {
    // The view answers "would it apply" with blocker functions, not by playing the
    // action on a copy; they must say what `apply` says.
    let mut s = Session::new(&baseline(), 7, None).unwrap();
    let mut checked = [0u32; 2];
    for _ in 0..60 {
        play(&mut s, 25, Policy::Script);
        let (r, g) = (&s.rules, &s.state);
        let v = r.view(g);
        assert_eq!(v.can_buy_land, r.is_legal(g, &Action::BuyLand), "land at tick {}", g.tick);
        assert_eq!(v.can_retire, r.is_legal(g, &Action::Retire), "retire at tick {}", g.tick);
        for b in &v.buildings {
            assert_eq!(b.affordable, r.is_legal(g, &buy(b.id)), "{} at tick {}", b.id, g.tick);
            checked[0] += b.affordable as u32;
        }
        for x in &v.research {
            let legal = r.is_legal(g, &Action::Research { research: x.id.into() });
            assert_eq!(x.affordable, legal, "{} at tick {}", x.id, g.tick);
        }
        checked[1] += v.can_buy_land as u32;
    }
    assert!(checked[0] > 0 && checked[1] > 0, "the game never made a buy or Buy Land legal: {checked:?}");
}

#[test]
fn the_view_shows_pad_readiness_and_the_rate_a_cap_throws_away() {
    let r = rules();
    let mut g = r.new_game(1);
    own(&mut g, "launch_pad", 1);
    own(&mut g, "solar_panel", 3);
    // Full but short of fuel: not fueled, not ready.
    g.run.pads[0].good = Some("he3".into());
    g.run.pads[0].cargo = r.p.shipments.pad_cargo_capacity;
    set(&mut g, "propellant", "19.4");
    let p = &r.view(&g).pads[0];
    assert!(!p.fueled && !p.ready && !p.can_launch);
    // Fuel enough: ready; part loaded, it can launch but isn't ready.
    set(&mut g, "propellant", "20");
    let p = &r.view(&g).pads[0];
    assert!(p.fueled && p.ready);
    g.run.pads[0].cargo = f("5");
    let p = &r.view(&g).pads[0];
    assert!(p.fueled && p.can_launch && !p.ready);
    // Energy at its cap: the change is 0, the rate the uncapped surplus.
    set(&mut g, "energy", "0");
    let below = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    let cap = below.cap.unwrap();
    set(&mut g, "energy", &cap.to_string());
    let at = r.view(&g).resources.iter().find(|x| x.id == "energy").unwrap().clone();
    assert!(at.per_day > Fx::ZERO && at.overflow == at.per_day, "solar makes a surplus: {:?}", at.per_day);
    assert_eq!(at.change_per_day, Fx::ZERO);
    assert_eq!(below.change_per_day, below.per_day, "below its cap nothing is cut");
    assert!(below.overflow == Fx::ZERO && below.unmet == Fx::ZERO);
}

#[test]
fn the_view_shows_nav_tiles_from_content_and_the_next_run() {
    let r = rules();
    let mut g = r.new_game(1);
    let shown = |g: &GameState| -> Vec<String> {
        r.view(g).nav.into_iter().filter(|t| t.visible).map(|t| t.id.to_string()).collect()
    };
    let start = shown(&g);
    assert!(start.contains(&"buildings".to_string()) && start.contains(&"story".to_string()));
    assert!(!start.contains(&"launch_pads".to_string()) && !start.contains(&"retirement".to_string()));
    g.career.owned_buildings.insert("launch_pad".into());
    g.run.enabled_panels.insert("retirement".into());
    let later = shown(&g);
    assert!(later.contains(&"launch_pads".to_string()) && later.contains(&"retirement".to_string()));
    // The next run's starting credits count this run's credits.
    g.run.stats.credits_earned = f("5000");
    let v = r.view(&g);
    assert_eq!(v.next_run.starting_credits, f("100"));
    assert_eq!(v.career.starting_credits, Fx::ZERO);
    assert!(v.last_run.is_none());
}

#[test]
fn story_timeline_view_has_acts_beats_and_started_side_quests() {
    let r = rules();
    let mut g = r.new_game(1);
    let v = r.view(&g);
    // A new game: Act 1 alone, 1.1 active in it with more to come, no side quest started.
    assert_eq!(v.acts.iter().map(|a| (a.act, a.name)).collect::<Vec<_>>(), vec![(1, "The Lunar Operation")]);
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.id, q.beat, q.act, q.more), ("qpower_boot_sequence", Some("1.1"), Some(1), true));
    assert!(v.side_quests.is_empty());
    // Through 1.4: no side quest yet; once speculators have come, Big Launch
    // (1.a) has started beside 1.4, not done, with its progress.
    let chain: Vec<_> = crate::content::content().events.iter().filter(|e| e.is_quest()).collect();
    let at = chain.iter().position(|e| e.id == "qtrade").unwrap();
    for e in &chain[..=at] {
        g.career.completed_quests.insert(e.id.clone());
    }
    assert!(r.view(&g).side_quests.is_empty());
    g.career.flags.insert("speculators_seen".into());
    let v = r.view(&g);
    let side: Vec<_> = v.side_quests.iter().map(|s| (s.id, s.parent, s.done)).collect();
    assert_eq!(side, vec![("qbig_launch", Some("qtrade"), false)]);
    assert!(v.side_quests[0].progress.is_some());
    assert!(v.story.iter().filter(|s| s.chain).all(|s| s.act == Some(1) && s.beat.is_some()));
    // The whole chain: every act reached, the last quest has none after it.
    for e in &chain {
        g.career.completed_quests.insert(e.id.clone());
    }
    g.career.completed_quests.remove(&chain[chain.len() - 1].id);
    let v = r.view(&g);
    assert_eq!(v.acts.len(), crate::content::content().acts.len());
    let q = v.quest.as_ref().unwrap();
    assert_eq!((q.act, q.more), (Some(4), false));
    assert_eq!(crate::content::content().act_of("qthe_vault"), Some(3));
    assert_eq!(crate::content::content().act_of("qfilling_vault"), Some(3));
}

#[test]
fn the_view_fills_every_text_placeholder() {
    // Every event fired: no `{name}` left in any text the view hands out.
    let r = rules();
    let mut g = r.new_game(1);
    for e in &crate::content::content().events {
        if e.is_quest() {
            g.career.completed_quests.insert(e.id.clone());
        } else {
            g.run.events.insert(e.id.clone(), 0);
        }
    }
    let v = r.view(&g);
    let placeholder = |t: &str| t.split('{').skip(1).any(|rest| rest.split('}').next().is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '_')));
    // `{rate}` is the UI's to fill (with Stats' formatting), from the view's `rate`.
    let unfilled = |s: &crate::view::StoryView| {
        placeholder(s.body) || placeholder(&s.effect.replace("{rate}", if s.rate.is_some() { "" } else { "{rate}" }))
    };
    for s in v.event_texts.iter().chain(&v.story) {
        assert!(!unfilled(s) && !placeholder(s.title) && !placeholder(s.summary) && !placeholder(s.goal) && !placeholder(s.reward), "{}: {}", s.id, s.body);
    }
    for c in &v.commands {
        assert!(!placeholder(&c.description), "{}: {}", c.id, c.description);
    }
    // Only the latest boredom phase is listed; each earlier one's text fills in when it is
    // the latest. Phase 2's rate is its baseline 0.2 per day (no modifiers yet).
    for later in ["boredom_phase_5", "boredom_phase_4", "boredom_phase_3"] {
        g.run.events.remove(later);
        let v = r.view(&g);
        for s in &v.event_texts {
            assert!(!unfilled(s), "{}: {}", s.id, s.body);
        }
    }
    let v = r.view(&g);
    let p2 = v.event_texts.iter().find(|s| s.id == "boredom_phase_2").unwrap();
    assert!(p2.effect.contains("{rate}") && !p2.body.contains("{rate}"), "{} / {}", p2.effect, p2.body);
    assert_eq!(p2.rate, Some(f("0.2")));
}

#[test]
fn story_lists_only_the_latest_boredom_phase() {
    let r = rules();
    let mut g = r.new_game(1);
    for id in ["propellant_discovery", "boredom_phase_2", "boredom_phase_3"] {
        g.run.events.insert(id.to_string(), 0);
    }
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    // The one-off beat stays; phase 3 supersedes phase 2 (still fired in the state).
    assert_eq!(ids, ["propellant_discovery", "boredom_phase_3"]);
}

#[test]
fn story_lists_events_by_the_day_they_fired() {
    // Content order is propellant_discovery before boredom_phase_2; fired the other way
    // round, Story lists them the other way round (oldest first: the UI shows newest first).
    let r = rules();
    let mut g = r.new_game(1);
    g.run.events.insert("boredom_phase_2".into(), 3);
    g.run.events.insert("propellant_discovery".into(), 40);
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["boredom_phase_2", "propellant_discovery"]);
    // A same-day pair keeps content order.
    g.run.events.insert("propellant_discovery".into(), 3);
    let v = r.view(&g);
    let ids: Vec<&str> = v.event_texts.iter().map(|s| s.id).collect();
    assert_eq!(ids, ["propellant_discovery", "boredom_phase_2"]);
}
