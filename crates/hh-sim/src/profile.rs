//! Player profiles (`configs/sim/profiles.json`): attention (real seconds
//! between looks) and a speed policy (ticks per real second, and when it
//! changes). Together they turn ticks into the player's real time.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attention {
    pub seconds: u64,
    pub says: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Speed {
    Fixed { speed: u64 },
    Adaptive { busy: u64, idle: u64, says: String },
    Ramp { speeds: Vec<u64>, says: String },
}

impl Speed {
    /// The speed after a look: `acted` is whether the player took any action
    /// at it, `retirements` the career's so far.
    pub fn at(&self, acted: bool, retirements: u32) -> u64 {
        match self {
            Speed::Fixed { speed } => *speed,
            Speed::Adaptive { busy, idle, .. } => {
                if acted {
                    *busy
                } else {
                    *idle
                }
            }
            Speed::Ramp { speeds, .. } => speeds[(retirements as usize).min(speeds.len() - 1)],
        }
    }

    fn check(&self, name: &str) -> Result<(), String> {
        let all: Vec<u64> = match self {
            Speed::Fixed { speed } => vec![*speed],
            Speed::Adaptive { busy, idle, .. } => vec![*busy, *idle],
            Speed::Ramp { speeds, .. } => speeds.clone(),
        };
        if all.is_empty() || all.iter().any(|&s| !(1..=200).contains(&s)) {
            return Err(format!("speed {name}: speeds must be 1 to 200 ticks a second, at least one"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profiles {
    #[serde(rename = "_doc")]
    pub doc: String,
    pub attention: BTreeMap<String, Attention>,
    pub speeds: BTreeMap<String, Speed>,
}

impl Profiles {
    pub fn from_json(text: &str) -> Result<Profiles, String> {
        let p: Profiles = serde_json::from_str(text).map_err(|e| format!("profiles: {e}"))?;
        for (name, s) in &p.speeds {
            s.check(name)?;
        }
        if let Some((name, _)) = p.attention.iter().find(|(_, a)| a.seconds == 0) {
            return Err(format!("attention {name}: seconds must be at least 1"));
        }
        Ok(p)
    }

    pub fn attention(&self, name: &str) -> Result<&Attention, String> {
        self.attention
            .get(name)
            .ok_or_else(|| format!("unknown attention {name:?} (known: {})", keys(&self.attention)))
    }

    pub fn speed(&self, name: &str) -> Result<&Speed, String> {
        self.speeds
            .get(name)
            .ok_or_else(|| format!("unknown speed {name:?} (known: {})", keys(&self.speeds)))
    }
}

fn keys<T>(m: &BTreeMap<String, T>) -> String {
    m.keys().cloned().collect::<Vec<_>>().join(", ")
}
