use uuid::Uuid;

pub trait IndexView<'a> {
    type View;

    fn index_view(&'a self, idx: usize) -> Self::View;

    fn index_views(&'a self, indices: &[usize]) -> Vec<Self::View> {
        indices.iter().map(|&i| self.index_view(i)).collect()
    }
}

pub struct WildfireState {
    pub grid: (u8, u8),

    pub num_envs: usize,

    pub env: EnvState,
    pub agent: AgentState,
}

pub struct WildfireStateView<'a> {
    pub grid: (u8, u8),

    pub num_envs: usize,

    pub env: EnvStateView<'a>,
    pub agent: AgentStateView<'a>,
}

impl WildfireState {
    pub fn new(grid: (u8, u8), num_envs: usize) -> Self {
        WildfireState {
            grid,
            num_envs,
            env: EnvState::new(num_envs),
            agent: AgentState::new(num_envs),
        }
    }
}

impl<'a> IndexView<'a> for WildfireState {
    type View = WildfireStateView<'a>;
    fn index_view(&'a self, idx: usize) -> Self::View {
        WildfireStateView {
            grid: self.grid,
            num_envs: 1,
            env: self.env.index_view(idx),
            agent: self.agent.index_view(idx),
        }
    }
}

impl<'a> From<&'a WildfireState> for WildfireStateView<'a> {
    fn from(state: &'a WildfireState) -> Self {
        WildfireStateView {
            grid: state.grid,
            num_envs: state.num_envs,
            env: EnvStateView::from(&state.env),
            agent: AgentStateView::from(&state.agent),
        }
    }
}

pub struct EnvState {
    pub offsets: Box<[(u16, u16)]>,

    pub size: Vec<u16>,
    pub intensity: Vec<u8>,
    pub fuel: Vec<u8>,
}

pub struct EnvStateView<'a> {
    pub size: &'a [u16],
    pub intensity: &'a [u8],
    pub fuel: &'a [u8],
}

impl EnvState {
    pub fn new(num_envs: usize) -> Self {
        EnvState {
            offsets: vec![(0, 0); num_envs].into_boxed_slice(),
            size: Vec::new(),
            intensity: Vec::new(),
            fuel: Vec::new(),
        }
    }
}

impl<'a> IndexView<'a> for EnvState {
    type View = EnvStateView<'a>;

    fn index_view(&'a self, env_idx: usize) -> Self::View {
        let (start, end) = self.offsets[env_idx];

        EnvStateView {
            size: &self.size[start as usize..end as usize],
            intensity: &self.intensity[start as usize..end as usize],
            fuel: &self.fuel[start as usize..end as usize],
        }
    }
}

impl<'a> From<&'a EnvState> for EnvStateView<'a> {
    fn from(state: &'a EnvState) -> Self {
        EnvStateView {
            size: &state.size,
            intensity: &state.intensity,
            fuel: &state.fuel,
        }
    }
}

pub struct AgentState {
    pub offsets: Vec<(u16, u16)>,

    pub name: Vec<Uuid>,

    pub y: Vec<u8>,
    pub x: Vec<u8>,

    pub power: Vec<u8>,
    pub suppressant: Vec<u8>,
    pub capacity: Vec<u8>,
    pub equipment: Vec<u8>,
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

impl AgentState {
    pub fn new(num_envs: usize) -> Self {
        AgentState {
            offsets: vec![(0, 0); num_envs],

            name: Vec::new(),

            y: Vec::new(),
            x: Vec::new(),

            power: Vec::new(),
            suppressant: Vec::new(),
            capacity: Vec::new(),
            equipment: Vec::new(),
        }
    }
}

impl<'a> IndexView<'a> for AgentState {
    type View = AgentStateView<'a>;
    fn index_view(&'a self, idx: usize) -> Self::View {
        AgentStateView {
            name: &self.name[idx..idx + 1],

            y: &self.y[idx..idx + 1],
            x: &self.x[idx..idx + 1],

            power: &self.power[idx..idx + 1],
            suppressant: &self.suppressant[idx..idx + 1],
            capacity: &self.capacity[idx..idx + 1],
            equipment: &self.equipment[idx..idx + 1],
        }
    }
}

impl<'a> From<&'a AgentState> for AgentStateView<'a> {
    fn from(state: &'a AgentState) -> Self {
        AgentStateView {
            name: &state.name,

            y: &state.y,
            x: &state.x,

            power: &state.power,
            suppressant: &state.suppressant,
            capacity: &state.capacity,
            equipment: &state.equipment,
        }
    }
}
