//! The Story panel: the active quest, side quests, acts, completed quests and this run's event texts.

use super::{keep, ActView, ProgressView, QuestView, SideQuestView, StoryView, SubObjectiveView, Text};
use crate::content::{content, Condition, Event, Trigger};
use crate::fx::Fx;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// The highest boredom phase whose event fired this run: Story lists only its text.
pub(super) fn latest_phase(g: &GameState) -> Option<u32> {
    content()
        .events
        .iter()
        .filter(|e| !e.silent && g.run.events.contains_key(&e.id))
        .filter_map(|e| match e.trigger {
            Trigger::BoredomPhase { phase } => Some(phase),
            _ => None,
        })
        .max()
}

/// The quest in progress, with its sub-objectives.
pub(super) fn quest(r: &Rules, g: &GameState) -> Option<QuestView> {
    let c = content();
    events::active_quest(g).map(|q| QuestView {
        id: &q.id,
        kind: events::kind(&q.condition),
        title: &q.title,
        goal: &q.goal,
        reward: &q.reward,
        hints: &q.hints,
        body: &q.body,
        story: &q.story,
        progress: progress(events::progress(r, g, &q.id, &q.condition)),
        sub_objectives: match &q.condition {
            Condition::AllOf { sub_objectives } => sub_objectives
                .iter()
                .map(|s| SubObjectiveView {
                    id: &s.id,
                    label: &s.label,
                    done: g.career.completed_sub_objectives.contains(&events::sub_key(&q.id, &s.id)),
                    progress: progress(events::progress(r, g, &q.id, &s.condition)),
                })
                .collect(),
            _ => Vec::new(),
        },
        beat: q.beat.as_deref(),
        act: c.act_of(&q.id),
        more: c.events.iter().filter(|e| e.is_quest()).skip_while(|e| e.id != q.id).nth(1).is_some(),
        picture: q.picture.as_deref(),
    })
}

/// The side quests started, done or not, in content order.
pub(super) fn side_quests(r: &Rules, g: &GameState) -> Vec<SideQuestView> {
    content()
        .events
        .iter()
        .filter(|e| events::side_started(g, e))
        .map(|e| {
            let done = g.career.completed_quests.contains(&e.id);
            SideQuestView {
                id: &e.id,
                beat: e.beat.as_deref(),
                title: &e.title,
                goal: &e.goal,
                reward: &e.reward,
                body: &e.body,
                parent: match &e.trigger {
                    Trigger::QuestComplete { quest } => Some(quest.as_str()),
                    Trigger::Flag { beside, .. } => Some(beside.as_str()),
                    _ => None,
                },
                done,
                progress: if done { None } else { progress(events::progress(r, g, &e.id, &e.condition)) },
                picture: e.picture.as_deref(),
                keep: keep::side_quest(g, e),
            }
        })
        .collect()
}

/// The acts whose opening quest is open or done.
pub(super) fn acts(g: &GameState) -> Vec<ActView> {
    content()
        .acts
        .iter()
        .filter(|a| events::quest_open_or_done(g, &a.opens_with))
        .map(|a| ActView { act: a.act, name: &a.name })
        .collect()
}

/// The quests completed in the career, by id in content order.
pub(super) fn completed_quests(g: &GameState) -> Vec<Text> {
    content()
        .events
        .iter()
        .filter(|e| g.career.completed_quests.contains(&e.id))
        .map(|e| e.id.as_str())
        .collect()
}

/// The completed quests' texts, in content order.
pub(super) fn story_list(g: &GameState) -> Vec<StoryView> {
    content().events.iter().filter(|e| g.career.completed_quests.contains(&e.id)).map(story).collect()
}

fn progress(p: Option<(Fx, Fx)>) -> Option<ProgressView> {
    p.map(|(now, target)| ProgressView { now, target })
}

fn story(e: &'static Event) -> StoryView {
    StoryView {
        id: &e.id,
        title: &e.title,
        summary: &e.summary,
        goal: &e.goal,
        reward: &e.reward,
        effect: &e.effect,
        body: &e.body,
        rate: None,
        chain: e.is_quest(),
        story: &e.story,
        payoff: &e.payoff,
        beat: e.beat.as_deref(),
        act: content().act_of(&e.id),
        picture: e.picture.as_deref(),
    }
}

/// An event's text with the number its `{rate}` takes: for a boredom-phase event, that
/// phase's boredom per day under this run's multiplier (display only; the UI formats it).
fn event_story(r: &Rules, g: &GameState, e: &'static Event) -> StoryView {
    let mut s = story(e);
    if let Trigger::BoredomPhase { phase } = e.trigger {
        let rates = &r.p.boredom.phase_rates;
        let i = (phase.max(1) as usize - 1).min(rates.len() - 1);
        s.rate = Some(rates[i] * boredom::multiplier(r, g));
    }
    s
}

/// The one-off events fired this run, oldest first by the day each fired (content order
/// within a day), but of the boredom-phase events only the latest phase's (a `once`
/// story event stays).
pub(super) fn event_texts(r: &Rules, g: &GameState, latest_phase: Option<u32>) -> Vec<StoryView> {
    let mut fired: Vec<(u64, &'static Event)> = content()
        .events
        .iter()
        .filter(|e| !e.is_quest() && !e.silent)
        .filter_map(|e| g.run.events.get(&e.id).map(|&day| (day, e)))
        .filter(|(_, e)| match e.trigger {
            Trigger::BoredomPhase { phase } if !e.once => Some(phase) == latest_phase,
            _ => true,
        })
        .collect();
    // Stable: same-day events keep content order.
    fired.sort_by_key(|&(day, _)| day);
    fired.into_iter().map(|(_, e)| event_story(r, g, e)).collect()
}
