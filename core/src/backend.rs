use crate::wildfire::{WildfireBatch, WildfireConfig};

pub trait WildfireBackend {
    fn step_batch(
        &mut self,
        batch: &mut WildfireBatch,
        actions: &[AgentActions],
        config: &WildfireConfig,
    ) {
        stub!()
    }
    // ... other backend-specific methods
}

pub enum Backend {
    Cpu(super::simd::CpuBackend),
    Cuda(super::cuda::CudaBackend),
}

impl WildfireBackend for Backend {
    fn step_batch(
        &mut self,
        batch: &mut WildfireBatch,
        actions: &[AgentActions],
        config: &WildfireConfig,
    ) {
        stub!()
    }
}

// Placeholder for AgentActions
pub struct AgentActions;
