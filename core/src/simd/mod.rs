pub struct CpuBackend;

impl CpuBackend {
    pub fn new() -> Self {
        Self
    }
}

use crate::backend::{AgentActions, WildfireBackend};
use crate::wildfire::{WildfireBatch, WildfireConfig};

impl WildfireBackend for CpuBackend {
    fn step_batch(
        &mut self,
        _batch: &mut WildfireBatch,
        _actions: &[AgentActions],
        _config: &WildfireConfig,
    ) {
        // SIMD-accelerated step logic will go here
    }
}
