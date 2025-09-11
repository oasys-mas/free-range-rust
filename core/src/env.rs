use bumpalo::Bump;
use color_eyre::Result;
use std::collections::HashMap;

use crate::config::Configuration;
use crate::spaces::Sample;
use crate::spaces::Space;
use crate::state::State;

pub trait SimulatedEnvironment<'a> {
    type State: State<'a>;
    type Config: Configuration;

    fn new(config: Self::Config, arena: &'a Bump) -> Result<Self>
    where
        Self: Sized;

    fn state(&self) -> &Self::State;

    fn step(&mut self, actions: &HashMap<String, Vec<Sample>>) -> Result<()>;

    fn reset(&mut self) -> Result<()>;
    fn reset_seeded(&mut self, seed: u64) -> Result<()>;

    fn observe(&self, agent: &str) -> Result<()>;

    fn action_space(&self, agent: &str) -> &dyn Space;

    fn observation_space(&self, agent: &str) -> &dyn Space;
}

pub trait LoggableEnvironment {}

pub trait Environment<'a>: SimulatedEnvironment<'a> + LoggableEnvironment {}

impl<'a, T> Environment<'a> for T where T: SimulatedEnvironment<'a> + LoggableEnvironment {}
