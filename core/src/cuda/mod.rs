pub struct CudaBackend;

impl CudaBackend {
    pub fn new() -> Self {
        Self
    }
}

use crate::backend::{AgentActions, WildfireBackend};
use crate::wildfire::{WildfireBatch, WildfireConfig};

impl WildfireBackend for CudaBackend {
    fn step_batch(
        &mut self,
        _batch: &mut WildfireBatch,
        _actions: &[AgentActions],
        _config: &WildfireConfig,
    ) {
        // CUDA-accelerated step logic will go here
    }
}
