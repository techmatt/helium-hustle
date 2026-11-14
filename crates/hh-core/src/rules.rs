//! The engine: a config compiled into its modules. Time advances only through
//! [`Rules::advance`] (N ticks, 1 tick = 1 in-game day); player actions apply
//! between ticks through [`Rules::apply`].

use std::collections::BTreeMap;

use crate::action::Action;
use crate::config::Config;
use crate::content::content;
use crate::fx::Fx;
use crate::modules::{self, Module, REGISTRY};
use crate::rng::Stream;
use crate::scenario::Scenario;
use crate::state::GameState;
use crate::view::{ResourceView, View};

pub struct Rules {
    pub config: Config,
    pub config_hash: String,
    /// `(module id, module)` in registry order.
    modules: Vec<(&'static str, Box<dyn Module>)>,
}

impl Rules {
    pub fn new(config: &Config) -> Result<Rules, String> {
        config.check()?;
        let modules = REGISTRY
            .iter()
            .filter_map(|d| config.module(d.id).map(|mc| (d.id, modules::build(d.id, mc))))
            .collect();
        Ok(Rules {
            config: config.clone(),
            config_hash: config.hash(),
            modules,
        })
    }

    /// The rules a state plays under (its embedded config).
    pub fn for_state(g: &GameState) -> Result<Rules, String> {
        Rules::new(&g.config)
    }

    /// A new game at tick 0.
    pub fn new_game(&self, seed: u64) -> GameState {
        let mut rng = BTreeMap::new();
        for (id, mc) in &self.config.modules {
            for &p in modules::rng_purposes(id, &mc.variant) {
                let fresh = rng.insert(p.to_string(), Stream::new(seed, p)).is_none();
                assert!(fresh, "rng purpose {p:?} declared twice");
            }
        }
        let mut g = GameState {
            seed,
            config: self.config.clone(),
            tick: 0,
            resources: content()
                .resources
                .iter()
                .map(|r| (r.id.clone(), Fx::ZERO))
                .collect(),
            buildings: BTreeMap::new(),
            rng,
        };
        for (_, m) in &self.modules {
            m.init(&mut g);
        }
        g
    }

    /// A new game, then a scenario's starting state over it.
    pub fn start(&self, seed: u64, scenario: Option<&Scenario>) -> GameState {
        let mut g = self.new_game(seed);
        if let Some(s) = scenario {
            s.apply_to(&mut g);
        }
        g
    }

    /// Runs `n` ticks.
    pub fn advance(&self, g: &mut GameState, n: u64) {
        for _ in 0..n {
            g.tick += 1;
            for (_, m) in &self.modules {
                m.tick(g);
            }
        }
    }

    /// Applies a player action between ticks. An `Err` leaves `g` untouched.
    pub fn apply(&self, g: &mut GameState, a: &Action) -> Result<(), String> {
        for (_, m) in &self.modules {
            if let Some(r) = m.apply(g, a) {
                return r;
            }
        }
        Err(format!("no module in this config takes {a:?}"))
    }

    /// Whether `a` would apply now.
    pub fn is_legal(&self, g: &GameState, a: &Action) -> bool {
        self.apply(&mut g.clone(), a).is_ok()
    }

    pub fn view(&self, g: &GameState) -> View {
        let mut v = View {
            tick: g.tick,
            resources: content()
                .resources
                .iter()
                .map(|r| ResourceView {
                    id: r.id.clone(),
                    name: r.name.clone(),
                    amount: g.amount(&r.id),
                    cap: None,
                    per_day: Fx::ZERO,
                })
                .collect(),
            buildings: Vec::new(),
        };
        for (_, m) in &self.modules {
            m.view(g, &mut v);
        }
        v
    }

    /// A state fits these rules: its config is this one and it holds every
    /// content resource and declared RNG stream.
    pub fn check_state(&self, g: &GameState) -> Result<(), String> {
        if g.config != self.config {
            return Err("state: its config differs from the rules'".into());
        }
        for r in &content().resources {
            if !g.resources.contains_key(&r.id) {
                return Err(format!("state: missing resource {:?}", r.id));
            }
        }
        for (id, mc) in &self.config.modules {
            for p in modules::rng_purposes(id, &mc.variant) {
                if !g.rng.contains_key(*p) {
                    return Err(format!("state: missing rng stream {p:?}"));
                }
            }
        }
        Ok(())
    }
}
