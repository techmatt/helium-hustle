//! Game records: a game is seed + config (+ the scenario it started from) + a
//! list of actions stamped with their tick. [`Record::replay`] rebuilds the
//! identical state. [`Session`] plays a game and keeps its record as it goes.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::config::Config;
use crate::hash::as_string;
use crate::rules::Rules;
use crate::scenario::Scenario;
use crate::state::GameState;
use crate::RULES_REVISION;

pub const RECORD_FORMAT: &str = "hh-record/1";

/// An action and the tick it was applied at (after that many ticks ran).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamped {
    pub tick: u64,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub format: String,
    pub rules_revision: u32,
    pub config: Config,
    #[serde(with = "as_string")]
    pub seed: u64,
    /// The scenario laid over the new game, in full (a later edit to the
    /// embedded file can't change an old record).
    pub scenario: Option<Scenario>,
    pub actions: Vec<Stamped>,
    /// The tick the game was left at.
    pub ticks: u64,
    /// The final state's fingerprint, if known.
    pub fingerprint: Option<String>,
}

impl Record {
    pub fn from_json(json: &str) -> Result<Record, String> {
        let r: Record = serde_json::from_str(json).map_err(|e| format!("bad record: {e}"))?;
        if r.format != RECORD_FORMAT {
            return Err(format!("record format {:?}, expected {RECORD_FORMAT:?}", r.format));
        }
        Ok(r)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("record serializes")
    }

    /// Rebuilds the game. Returns the final state and warnings (another rules
    /// revision, a fingerprint that differs); errors on an action that no
    /// longer applies or a stamp out of order.
    pub fn replay(&self) -> Result<(GameState, Vec<String>), String> {
        let rules = Rules::new(&self.config)?;
        if let Some(s) = &self.scenario {
            s.check()?;
        }
        let mut g = rules.start(self.seed, self.scenario.as_ref());
        for (i, s) in self.actions.iter().enumerate() {
            if s.tick < g.tick {
                return Err(format!("action {i}: tick {} is before tick {}", s.tick, g.tick));
            }
            let n = s.tick - g.tick;
            rules.advance(&mut g, n);
            rules
                .apply(&mut g, &s.action)
                .map_err(|e| format!("action {i} at tick {}: {e}", s.tick))?;
        }
        if self.ticks < g.tick {
            return Err(format!("ticks {} is before the last action's tick {}", self.ticks, g.tick));
        }
        let n = self.ticks - g.tick;
        rules.advance(&mut g, n);
        let mut warnings = Vec::new();
        if self.rules_revision != RULES_REVISION {
            warnings.push(format!(
                "recorded under rules revision {}, replayed under {RULES_REVISION}",
                self.rules_revision
            ));
        }
        if let Some(f) = &self.fingerprint {
            let now = g.fingerprint();
            if *f != now {
                warnings.push(format!("final fingerprint {now}, recorded {f}"));
            }
        }
        Ok((g, warnings))
    }
}

/// A game in play with its record.
pub struct Session {
    pub rules: Rules,
    pub state: GameState,
    actions: Vec<Stamped>,
    scenario: Option<Scenario>,
}

impl Session {
    pub fn new(config: &Config, seed: u64, scenario: Option<&Scenario>) -> Result<Session, String> {
        let rules = Rules::new(config)?;
        let state = rules.start(seed, scenario);
        Ok(Session {
            rules,
            state,
            actions: Vec::new(),
            scenario: scenario.cloned(),
        })
    }

    pub fn advance(&mut self, n: u64) {
        self.rules.advance(&mut self.state, n);
    }

    /// Applies and records an action; an illegal one is neither.
    pub fn apply(&mut self, a: &Action) -> Result<(), String> {
        self.rules.apply(&mut self.state, a)?;
        self.actions.push(Stamped {
            tick: self.state.tick,
            action: a.clone(),
        });
        Ok(())
    }

    pub fn record(&self) -> Record {
        Record {
            format: RECORD_FORMAT.to_string(),
            rules_revision: RULES_REVISION,
            config: self.rules.config.clone(),
            seed: self.state.seed,
            scenario: self.scenario.clone(),
            actions: self.actions.clone(),
            ticks: self.state.tick,
            fingerprint: Some(self.state.fingerprint()),
        }
    }
}
