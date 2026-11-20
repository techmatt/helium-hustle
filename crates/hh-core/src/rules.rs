//! The engine: a config compiled into its modules' params. Time advances only
//! through [`Rules::advance`] (N ticks, 1 tick = 1 in-game day); player actions
//! apply between ticks through [`Rules::apply`].

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::action::Action;
use crate::config::Config;
use crate::content::content;
use crate::modules::{self, *};
use crate::rng::Stream;
use crate::scenario::Scenario;
use crate::state::{Career, GameState, Run};
use crate::view::View;

#[derive(Clone)]
pub struct Rules {
    /// Shared with every game it starts and every copy of it (`Session::fork`, the
    /// oracle's rollouts), so neither copies the config.
    pub config: Arc<Config>,
    pub config_hash: String,
    /// Every module's params (shared by copies, as the config).
    pub(crate) p: Arc<modules::Params>,
}

impl Rules {
    pub fn new(config: &Config) -> Result<Rules, String> {
        config.check()?;
        let p = modules::Params::new(|id| config.module(id).cloned())?;
        Ok(Rules {
            config: Arc::new(config.clone()),
            config_hash: config.hash(),
            p: Arc::new(p),
        })
    }

    /// The rules a state plays under (its embedded config).
    pub fn for_state(g: &GameState) -> Result<Rules, String> {
        Rules::new(&g.config)
    }

    /// A new game: run 1 of a fresh career, at tick 0.
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
            config: Arc::clone(&self.config),
            tick: 0,
            career: Career { run_number: 1, ..Career::default() },
            run: Run::default(),
            rng,
            cheated: false,
        };
        career::new_run(self, &mut g);
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
            self.tick(g);
        }
    }

    /// One day, in this order (docs/rules/README.md, *The day*).
    fn tick(&self, g: &mut GameState) {
        g.tick += 1;
        economy::fold_flow_average(self, g);
        g.run.today = Default::default();
        boredom::grow(self, g);
        programs::expire_overclocks(g);
        demand::update(self, g);
        // Programs are paid before buildings that take upkeep, from the stock
        // and what the upkeep-free buildings made today.
        let free_energy = economy::produce_free(self, g);
        programs::run(self, g);
        economy::resolve(self, g, free_energy);
        projects::drain(g);
        shipments::tick(g);
        demand::bursts(self, g);
        economy::clamp(self, g);
        programs::note_day_net(g);
        events::check(self, g);
        achievements::check(g);
        ideology::track(self, g);
        g.run.day += 1;
        if boredom::at_cap(self, g) {
            career::retire(self, g, true);
        }
        events::trim_activity(self, g);
    }

    /// Applies a player action between ticks. An `Err` leaves `g` untouched.
    pub fn apply(&self, g: &mut GameState, a: &Action) -> Result<(), String> {
        let mut h = g.clone();
        match a {
            Action::Buy { building } => economy::buy(self, &mut h, building),
            Action::Sell { building } => economy::sell(&mut h, building),
            Action::SetActive { building, active } => economy::set_active(&mut h, building, *active),
            Action::BuyLand => economy::buy_land(self, &mut h),
            Action::SetProgram { program, entries } => programs::set_program(self, &mut h, *program, entries),
            Action::SetProcessors { program, processors } => {
                programs::set_processors(&mut h, *program, *processors)
            }
            Action::SetPadGood { pad, good } => shipments::set_pad_good(self, &mut h, *pad, good),
            Action::SetPadPaused { pad, paused } => shipments::set_pad_paused(&mut h, *pad, *paused),
            Action::Launch { pad } => shipments::launch_action(self, &mut h, *pad),
            Action::SetLoadingPriority { goods } => shipments::set_loading_priority(self, &mut h, goods),
            Action::Research { research } => research::buy(self, &mut h, research),
            Action::SetProjectRate { project, resource, rate } => {
                projects::set_rate(self, &mut h, project, resource, *rate)
            }
            Action::Retire => career::retire_action(self, &mut h),
            Action::Cheat { cheat } => crate::cheat::apply(self, &mut h, cheat),
        }?;
        events::trim_activity(self, &mut h);
        *g = h;
        Ok(())
    }

    /// Whether `a` would apply now (tests and debugging: it applies to a copy; the
    /// view uses each module's blocker functions instead).
    pub fn is_legal(&self, g: &GameState, a: &Action) -> bool {
        self.apply(&mut g.clone(), a).is_ok()
    }

    pub fn view(&self, g: &GameState) -> View {
        crate::view::build(self, g)
    }

    /// The state one day on, for the view's expected change per day.
    pub(crate) fn peek(&self, g: &GameState) -> GameState {
        let mut h = g.clone();
        self.tick(&mut h);
        h
    }

    /// A state fits these rules: its config is this one and it holds every
    /// content resource and declared RNG stream.
    pub fn check_state(&self, g: &GameState) -> Result<(), String> {
        if *g.config != *self.config {
            return Err("state: its config differs from the rules'".into());
        }
        for r in &content().resources {
            if !g.run.resources.contains_key(&r.id) {
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
