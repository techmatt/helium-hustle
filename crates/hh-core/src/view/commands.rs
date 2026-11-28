//! The Commands list: each command's verb, flows, boredom cost and description with its numbers filled in.

use std::borrow::Cow;

use super::CommandView;
use crate::content::{content, CommandEffect};
use crate::modules::programs as program_rules;
use crate::modules::*;
use crate::rules::Rules;
use crate::state::GameState;

/// Every command, in content order.
pub(super) fn commands(r: &Rules, g: &GameState) -> Vec<CommandView> {
    content()
        .commands
        .iter()
        .map(|cmd| {
            let (costs, produces) = program_rules::flows(r, g, cmd);
            let verb = match &cmd.effect {
                Some(CommandEffect::Promote { .. }) => Some("Promote"),
                Some(CommandEffect::Fund { .. }) => Some("Fund"),
                Some(CommandEffect::Overclock { .. }) => Some("Overclock"),
                None if cmd.costs.contains_key(economy::CREDITS) && cmd.produces.len() == 1 => Some("Buy"),
                _ => None,
            };
            let target = verb.and_then(|vb| cmd.name.strip_prefix(vb)).map(str::trim);
            CommandView {
                verb,
                target,
                id: &cmd.id,
                name: &cmd.name,
                description: command_description(cmd),
                unlocked: program_rules::unlocked(g, cmd),
                visible: program_rules::visible(g, cmd),
                costs,
                produces,
                boredom: program_rules::boredom_effect(r, g, cmd),
                ideology: cmd.ideology.as_deref(),
            }
        })
        .collect()
}

/// A command's description with an overclock's `{days}` filled in: its fixed
/// duration ("4 days", "1 day").
fn command_description(cmd: &'static crate::content::Command) -> Cow<'static, str> {
    match &cmd.effect {
        Some(CommandEffect::Overclock { duration, .. }) => {
            let n = program_rules::overclock_days(*duration);
            let days = if n == 1 { "1 day".to_string() } else { format!("{n} days") };
            Cow::Owned(cmd.description.replace("{days}", &days))
        }
        _ => Cow::Borrowed(&cmd.description),
    }
}
