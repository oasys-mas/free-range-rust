pub mod config;
pub mod state;
pub mod transitions;

use bumpalo::Bump;
use color_eyre::Result;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::any::Any;
use std::collections::HashMap;

use crate::config::Configuration;
use crate::env::{LoggableEnvironment, SimulatedEnvironment};
use crate::spaces::{Sample, Space};
use crate::state::State;
use crate::wildfire::config::WildfireConfiguration;
use crate::wildfire::state::WildfireState;
use crate::wildfire::transitions::WildfireTransition;

#[allow(dead_code)]
pub struct WildfireEnvironment<'a> {
    arena: &'a Bump,
    rng: StdRng,

    config: WildfireConfiguration,

    state: WildfireState<'a>,
    transitions: Vec<Box<WildfireTransition<'a>>>,

    outputs: HashMap<String, Box<dyn Any>>,
}

impl<'a> SimulatedEnvironment<'a> for WildfireEnvironment<'a> {
    type State = WildfireState<'a>;
    type Config = WildfireConfiguration;

    fn new(config: WildfireConfiguration, arena: &'a Bump) -> Result<Self> {
        config.validate()?;

        let state = WildfireState::new(&config, arena);
        Ok(WildfireEnvironment {
            arena,
            rng: StdRng::from_entropy(),

            config,

            state,
            transitions: vec![],

            outputs: HashMap::new(),
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.state.clear();

        todo!();
    }

    fn reset_seeded(&mut self, seed: u64) -> Result<()> {
        self.rng = StdRng::seed_from_u64(seed);

        self.reset()
    }

    fn step(&mut self, _actions: &HashMap<String, Vec<Sample>>) -> Result<()> {
        todo!()
    }

    fn state(&self) -> &WildfireState<'a> {
        &self.state
    }

    fn action_space(&self, _agent: &str) -> &dyn Space {
        todo!()
    }

    fn observation_space(&self, _agent: &str) -> &dyn Space {
        todo!()
    }
}

impl LoggableEnvironment for WildfireEnvironment<'_> {}
