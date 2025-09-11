use std::collections::HashMap;

#[derive(Clone)]
pub struct Fire {
    pub x: usize,
    pub y: usize,
    pub power: f32,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct Agent {
    pub x: usize,
    pub y: usize,
    pub suppressant: f32,
    pub equipment: f32,
}

#[derive(Clone)]
pub struct WildfireState {
    pub fires: HashMap<(usize, usize), Vec<Fire>>,
    pub agents: HashMap<(usize, usize), Vec<Agent>>,
    // Add other dense fields (fuel, terrain, etc.) as needed
}

pub struct WildfireBatch {
    pub envs: Vec<WildfireState>,
}

impl WildfireBatch {
    pub fn new() -> Self {
        stub!()
    }
    pub fn reset(&mut self) {
        stub!()
    }
    pub fn step(&mut self) {
        stub!()
    }
}
