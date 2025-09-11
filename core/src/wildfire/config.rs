use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;
use std::collections::HashMap;

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

use crate::config::Configuration;

#[derive(Debug, Deserialize, Clone)]
pub struct WildfireConfiguration {
    /// number of environments
    pub num_envs: usize,
    /// grid dimensions (rows, cols)
    pub grid: (u8, u8),

    pub max_agents: usize,
    /// maximum number of fires allowed
    pub max_fires: usize,

    /// maximum number of agents allowed in each space (indexed by space: y * grid.1 + x)
    pub max_agents_per_space: Vec<usize>,
    /// maximum number of fires allowed in each space (indexed by space: y * grid.1 + x)
    pub max_fires_per_space: Vec<usize>,

    /// initial agents: (count, y, x, power, suppressant, capacity, equipment)
    pub initial_agents: Vec<(usize, u8, u8, u8, u8, u8, u8)>,
    /// initial fires: (count, y, x, size, intensity)
    pub initial_fires: Vec<(usize, u8, u8, u16, u8)>,

    pub initial_fuel: Vec<u8>,
}

impl Configuration for WildfireConfiguration {
    fn validate(&self) -> Result<()> {
        let num_spaces = self.grid.0 as usize * self.grid.1 as usize;
        if self.max_agents_per_space.len() != num_spaces {
            return Err(eyre!(
                "max_agents_per_space length ({}) does not match number of spaces ({})",
                self.max_agents_per_space.len(),
                num_spaces
            ));
        }
        if self.max_fires_per_space.len() != num_spaces {
            return Err(eyre!(
                "max_fires_per_space length ({}) does not match number of spaces ({})",
                self.max_fires_per_space.len(),
                num_spaces
            ));
        }

        let total_agents: usize = self.initial_agents.iter().map(|(count, ..)| *count).sum();
        if total_agents > self.max_agents {
            return Err(eyre!(
                "Total initial agents ({}) exceeds max_agents ({})",
                total_agents,
                self.max_agents
            ));
        }

        let total_fires: usize = self.initial_fires.iter().map(|(count, ..)| *count).sum();
        if total_fires > self.max_fires {
            return Err(eyre!(
                "Total initial fires ({}) exceeds max_fires ({})",
                total_fires,
                self.max_fires
            ));
        }

        for (space_idx, (&max_agents_space, &max_fires_space)) in self
            .max_agents_per_space
            .iter()
            .zip(self.max_fires_per_space.iter())
            .enumerate()
        {
            if max_agents_space > self.max_agents {
                return Err(eyre!(
                    "max_agents_per_space[{}] ({}) exceeds global max_agents ({})",
                    space_idx,
                    max_agents_space,
                    self.max_agents
                ));
            }
            if max_fires_space > self.max_fires {
                return Err(eyre!(
                    "max_fires_per_space[{}] ({}) exceeds global max_fires ({})",
                    space_idx,
                    max_fires_space,
                    self.max_fires
                ));
            }
        }

        let mut agents_per_space = vec![0usize; num_spaces];
        for (count, y, x, ..) in &self.initial_agents {
            if *y >= self.grid.0 || *x >= self.grid.1 {
                return Err(eyre!(
                    "Initial agent position ({}, {}) is out of grid bounds ({}, {})",
                    y,
                    x,
                    self.grid.0,
                    self.grid.1
                ));
            }

            let idx = (*y as usize) * self.grid.1 as usize + (*x as usize);
            if idx < num_spaces {
                agents_per_space[idx] += *count;
            }
        }
        for (idx, &agents) in agents_per_space.iter().enumerate() {
            if agents > self.max_agents_per_space[idx] {
                return Err(eyre!(
                    "Initial agents ({}) in space {} exceed max_agents_per_space ({})",
                    agents,
                    idx,
                    self.max_agents_per_space[idx]
                ));
            }
        }

        let mut fires_per_space = vec![0usize; num_spaces];
        for (count, y, x, ..) in &self.initial_fires {
            if *y >= self.grid.0 || *x >= self.grid.1 {
                return Err(eyre!(
                    "Initial fire position ({}, {}) is out of grid bounds ({}, {})",
                    y,
                    x,
                    self.grid.0,
                    self.grid.1
                ));
            }
            let idx = (*y as usize) * self.grid.1 as usize + (*x as usize);
            if idx < num_spaces {
                fires_per_space[idx] += *count;
            }
        }
        for (idx, &fires) in fires_per_space.iter().enumerate() {
            if fires > self.max_fires_per_space[idx] {
                return Err(eyre!(
                    "Initial fires ({}) in space {} exceed max_fires_per_space ({})",
                    fires,
                    idx,
                    self.max_fires_per_space[idx]
                ));
            }
        }

        Ok(())
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for WildfireConfiguration {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let num_envs = u.int_in_range(1..=2048_usize)?;

        let grid = (u.int_in_range(1..=u8::MAX)?, u.int_in_range(1..=u8::MAX)?);
        let max_agents = u.int_in_range(1..=u8::MAX as usize)?;
        let max_fires = u.int_in_range(1..=u8::MAX as usize)?;

        let num_spaces = grid.0 as usize * grid.1 as usize;

        let mut max_agents_per_space = Vec::with_capacity(num_spaces);
        for _ in 0..num_spaces {
            max_agents_per_space.push(u.int_in_range(1..=u8::MAX as usize)?);
        }
        let mut max_fires_per_space = Vec::with_capacity(num_spaces);
        for _ in 0..num_spaces {
            max_fires_per_space.push(u.int_in_range(1..=u8::MAX as usize)?);
        }

        let num_agents = u.int_in_range(0..=max_agents)?;
        let mut agent_map: HashMap<(u8, u8, u8, u8, u8, u8), usize> = HashMap::new();
        for _ in 0..num_agents {
            let key = (
                u.int_in_range(0..=grid.0 - 1)?, // y
                u.int_in_range(0..=grid.1 - 1)?, // x
                u.int_in_range(0..=10)?,         // power
                u.int_in_range(0..=10)?,         // suppressant
                u.int_in_range(0..=10)?,         // capacity
                u.int_in_range(0..=10)?,         // equipment
            );
            *agent_map.entry(key).or_insert(0) += 1;
        }
        let initial_agents = agent_map
            .into_iter()
            .map(|((y, x, power, suppressant, capacity, equipment), count)| {
                (count, y, x, power, suppressant, capacity, equipment)
            })
            .collect();

        let num_fires = u.int_in_range(0..=max_fires)?;
        let mut fire_map: HashMap<(u8, u8, u16, u8), usize> = HashMap::new();
        for _ in 0..num_fires {
            let key = (
                u.int_in_range(0..=grid.0 - 1)?, // y
                u.int_in_range(0..=grid.1 - 1)?, // x
                u.int_in_range(0..=u16::MAX)?,   // size
                u.int_in_range(0..=u8::MAX)?,    // intensity
            );
            *fire_map.entry(key).or_insert(0) += 1;
        }
        let initial_fires: Vec<(usize, u8, u8, u16, u8)> = fire_map
            .into_iter()
            .map(|((y, x, size, intensity), count)| (count, y, x, size, intensity))
            .collect();

        let grid_len = grid.0 as usize * grid.1 as usize;
        let total_fires = initial_fires.iter().map(|(count, ..)| count).sum::<usize>();
        let mut initial_fuel: Vec<u8> = {
            let mut fuel = Vec::with_capacity(grid_len);
            for _ in 0..grid_len {
                fuel.push(u.int_in_range(0..=u8::MAX)?);
            }
            fuel
        };

        let mut fuel_sum: usize = initial_fuel.iter().map(|&f| f as usize).sum();
        while fuel_sum < total_fires {
            let idx = u.int_in_range(0..=grid_len - 1)?;
            if initial_fuel[idx] < u8::MAX {
                initial_fuel[idx] += 1;
                fuel_sum += 1;
            }
        }
        Ok(WildfireConfiguration {
            num_envs,
            grid,
            max_agents,
            max_fires,
            max_agents_per_space,
            max_fires_per_space,
            initial_agents,
            initial_fires,
            initial_fuel,
        })
    }
}
