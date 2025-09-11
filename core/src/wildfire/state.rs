use bumpalo::Bump;
use bumpalo::collections::Vec;
use color_eyre::eyre::{Result, eyre};
use uuid::Uuid;

use crate::state::{IndexView, State};
use crate::wildfire::config::WildfireConfiguration;

pub struct WildfireState<'a> {
    pub num_envs: usize,

    pub env: EnvState<'a>,
    pub agent: AgentState<'a>,
}

pub struct WildfireStateView<'a> {
    pub env: EnvStateView<'a>,
    pub agent: AgentStateView<'a>,
}

impl<'a> WildfireState<'a> {
    pub fn new(config: &WildfireConfiguration, arena: &'a Bump) -> Self {
        WildfireState {
            num_envs: config.num_envs,
            env: EnvState::new(arena, config.num_envs, config.max_fires, config.grid),
            agent: AgentState::new(arena, config.num_envs, config.max_agents),
        }
    }
}

impl<'a> State<'a> for WildfireState<'a> {
    type Config = WildfireConfiguration;

    fn initialize(_config: &Self::Config) -> Result<()> {
        todo!()
    }

    fn clear(&mut self) {
        self.env.clear();
        self.agent.clear();
    }
}

impl<'a> IndexView<'a> for WildfireState<'a> {
    type View = WildfireStateView<'a>;

    fn index_view(&'a self, idx: usize) -> Self::View {
        WildfireStateView {
            env: self.env.index_view(idx),
            agent: self.agent.index_view(idx),
        }
    }
}

impl<'a> From<&'a WildfireState<'a>> for WildfireStateView<'a> {
    fn from(state: &'a WildfireState) -> Self {
        WildfireStateView {
            env: EnvStateView::from(&state.env),
            agent: AgentStateView::from(&state.agent),
        }
    }
}

pub struct EnvState<'a> {
    pub max_fires: usize,
    pub offsets: &'a mut [(usize, usize)],

    pub size: &'a mut [u16],
    pub intensity: &'a mut [u8],
    pub fuel: &'a mut [u8],
}

pub struct EnvStateView<'a> {
    pub size: &'a [u16],
    pub intensity: &'a [u8],
    pub fuel: &'a [u8],
}

impl<'a> EnvState<'a> {
    pub fn clear(&mut self) {
        for (i, offset) in self.offsets.iter_mut().enumerate() {
            let start = i * self.max_fires;
            *offset = (start, start);
        }
    }

    pub fn add_fires(&mut self, env_idx: usize, fires: &[(u16, u8)]) -> Result<()> {
        for fire in fires {
            self.add_fire(env_idx, fire)?;
        }

        Ok(())
    }

    pub fn remove_fires(&mut self, env_idx: usize, indices: &[usize]) -> Result<()> {
        let mut sorted = indices.to_vec();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        for &idx in &sorted {
            self.remove_fire(env_idx, idx)?;
        }
        Ok(())
    }

    pub fn remove_fire(&mut self, env_idx: usize, remove_idx: usize) -> Result<()> {
        let (start, end) = self.offsets[env_idx];
        if remove_idx >= end - start {
            return Err(eyre!("Remove index out of bounds"));
        }

        let last_idx = end - 1;
        if remove_idx != last_idx {
            self.size.swap(start + remove_idx, last_idx);
            self.intensity.swap(start + remove_idx, last_idx);
            self.fuel.swap(start + remove_idx, last_idx);
        }

        self.offsets[env_idx].1 -= 1;
        Ok(())
    }

    pub fn add_fire(&mut self, env_idx: usize, fire: &(u16, u8)) -> Result<()> {
        let (start, end) = self.offsets[env_idx];
        if end >= start + self.max_fires {
            return Err(eyre!(
                "Maximum number of fires reached for this environment"
            ));
        }

        let (size, intensity) = *fire;
        self.size[end] = size;
        self.intensity[end] = intensity;

        self.offsets[env_idx].1 += 1;
        Ok(())
    }

    pub fn new(arena: &'a Bump, num_envs: usize, max_fires: usize, grid: (u8, u8)) -> Self {
        let mut offsets = Vec::with_capacity_in(num_envs, arena);
        for i in 0..num_envs {
            let start = i * max_fires;
            offsets.push((start, start));
        }
        let offsets = offsets.into_bump_slice_mut();

        let size = Vec::with_capacity_in(num_envs * max_fires, arena).into_bump_slice_mut();
        let intensity = Vec::with_capacity_in(num_envs * max_fires, arena).into_bump_slice_mut();

        let grid_len = grid.0 as usize * grid.1 as usize;
        let fuel = Vec::with_capacity_in(num_envs * grid_len, arena).into_bump_slice_mut();

        EnvState {
            offsets,
            max_fires,
            size,
            intensity,
            fuel,
        }
    }
}

impl<'a> IndexView<'a> for EnvState<'a> {
    type View = EnvStateView<'a>;

    fn index_view(&'a self, env_idx: usize) -> Self::View {
        let (start, end) = self.offsets[env_idx];
        EnvStateView {
            size: &self.size[start..end],
            intensity: &self.intensity[start..end],
            fuel: &self.fuel[start..end],
        }
    }
}

impl<'a> From<&'a EnvState<'a>> for EnvStateView<'a> {
    fn from(state: &'a EnvState) -> Self {
        EnvStateView {
            size: state.size,
            intensity: state.intensity,
            fuel: state.fuel,
        }
    }
}

pub struct AgentState<'a> {
    pub max_agents: usize,
    pub offsets: &'a mut [(usize, usize)],

    pub name: &'a mut [Uuid],

    pub y: &'a mut [u8],
    pub x: &'a mut [u8],

    pub power: &'a mut [u8],
    pub suppressant: &'a mut [u8],
    pub capacity: &'a mut [u8],
    pub equipment: &'a mut [u8],
}

pub struct AgentStateView<'a> {
    pub name: &'a [Uuid],

    pub y: &'a [u8],
    pub x: &'a [u8],

    pub power: &'a [u8],
    pub suppressant: &'a [u8],
    pub capacity: &'a [u8],
    pub equipment: &'a [u8],
}

impl<'a> AgentState<'a> {
    pub fn new(arena: &'a Bump, num_envs: usize, max_agents: usize) -> Self {
        let mut offsets = Vec::with_capacity_in(num_envs, arena);
        for i in 0..num_envs {
            let start = i * max_agents;
            offsets.push((start, start));
        }
        let offsets = offsets.into_bump_slice_mut();

        let name = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let y = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let x = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let power = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let suppressant = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let capacity = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();
        let equipment = Vec::with_capacity_in(num_envs * max_agents, arena).into_bump_slice_mut();

        AgentState {
            offsets,
            name,
            y,
            x,
            power,
            suppressant,
            capacity,
            equipment,
            max_agents,
        }
    }

    pub fn clear(&mut self) {
        for (i, offset) in self.offsets.iter_mut().enumerate() {
            let start = i * self.max_agents;
            *offset = (start, start);
        }
    }

    pub fn add_agents(
        &mut self,
        env_idx: usize,
        agents: &[(Uuid, u8, u8, u8, u8, u8, u8)],
    ) -> Result<()> {
        for agent in agents {
            self.add_agent(env_idx, agent)?;
        }

        Ok(())
    }

    pub fn add_agent(
        &mut self,
        env_idx: usize,
        agent: &(Uuid, u8, u8, u8, u8, u8, u8),
    ) -> Result<()> {
        let (start, end) = self.offsets[env_idx];
        if end >= start + self.max_agents {
            return Err(eyre!(
                "Maximum number of agents reached for this environment"
            ));
        }

        let (name, y, x, power, suppressant, capacity, equipment) = *agent;
        self.name[end] = name;
        self.y[end] = y;
        self.x[end] = x;
        self.power[end] = power;
        self.suppressant[end] = suppressant;
        self.capacity[end] = capacity;
        self.equipment[end] = equipment;

        self.offsets[env_idx].1 += 1;
        Ok(())
    }

    pub fn remove_agents(&mut self, env_idx: usize, indices: &[usize]) -> Result<()> {
        let mut sorted = indices.to_vec();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        for &idx in &sorted {
            self.remove_agent(env_idx, idx)?;
        }

        Ok(())
    }

    pub fn remove_agent(&mut self, env_idx: usize, remove_idx: usize) -> Result<()> {
        let (start, end) = self.offsets[env_idx];
        if remove_idx >= end - start {
            return Err(eyre!("Remove index out of bounds"));
        }

        let last_idx = end - 1;
        if remove_idx != last_idx {
            self.name.swap(start + remove_idx, last_idx);
            self.y.swap(start + remove_idx, last_idx);
            self.x.swap(start + remove_idx, last_idx);
            self.power.swap(start + remove_idx, last_idx);
            self.suppressant.swap(start + remove_idx, last_idx);
            self.capacity.swap(start + remove_idx, last_idx);
            self.equipment.swap(start + remove_idx, last_idx);
        }

        self.offsets[env_idx].1 -= 1;
        Ok(())
    }
}

impl<'a> IndexView<'a> for AgentState<'a> {
    type View = AgentStateView<'a>;

    fn index_view(&'a self, env_idx: usize) -> Self::View {
        let (start, end) = self.offsets[env_idx];
        AgentStateView {
            name: &self.name[start..end],
            y: &self.y[start..end],
            x: &self.x[start..end],
            power: &self.power[start..end],
            suppressant: &self.suppressant[start..end],
            capacity: &self.capacity[start..end],
            equipment: &self.equipment[start..end],
        }
    }
}

impl<'a> From<&'a AgentState<'a>> for AgentStateView<'a> {
    fn from(state: &'a AgentState) -> Self {
        AgentStateView {
            name: state.name,
            y: state.y,
            x: state.x,
            power: state.power,
            suppressant: state.suppressant,
            capacity: state.capacity,
            equipment: state.equipment,
        }
    }
}
