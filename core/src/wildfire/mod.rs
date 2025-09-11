pub mod config;
pub mod error;
pub mod state;
pub mod transitions;

use bumpalo::Bump;
use color_eyre::Result;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::any::Any;
use std::collections::HashMap;
use std::iter::{repeat_n, repeat_with};
use uuid::Uuid;

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
        let agents: Vec<(Uuid, u8, u8, u8, u8, u8, u8)> = self
            .config
            .initial_agents
            .iter()
            .flat_map(|(count, y, x, power, suppressant, capacity, equipment)| {
                repeat_with(|| {
                    (
                        Uuid::new_v4(),
                        *y,
                        *x,
                        *power,
                        *suppressant,
                        *capacity,
                        *equipment,
                    )
                })
                .take(*count)
            })
            .collect();
        for env_idx in 0..self.config.num_envs {
            self.state.agent.add_agents(env_idx, &agents)?;
        }

        let fires: Vec<(u8, u8, u16, u8)> = self
            .config
            .initial_fires
            .iter()
            .flat_map(|(count, y, x, size, intensity)| {
                repeat_n((*y, *x, *size, *intensity), *count)
            })
            .collect();
        for env_idx in 0..self.config.num_envs {
            self.state.env.add_fires(env_idx, &fires)?;
        }

        let grid_len = self.config.grid.0 as usize * self.config.grid.1 as usize;
        for env_idx in 0..self.config.num_envs {
            let start = env_idx * grid_len;
            let end = start + grid_len;
            self.state.env.fuel[start..end].copy_from_slice(&self.config.initial_fuel);
        }

        Ok(())
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

    fn observe(&self, _agent: &str) -> Result<()> {
        todo!()
    }

    fn action_space(&self, _agent: &str) -> &dyn Space {
        todo!()
    }

    fn observation_space(&self, _agent: &str) -> &dyn Space {
        todo!()
    }
}

impl LoggableEnvironment for WildfireEnvironment<'_> {}
