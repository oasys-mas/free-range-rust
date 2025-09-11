// core/src/env.rs

use serde_json::Value;

/// General trait for all batched environments (wildfire, cybersecurity, rideshare, etc.)
pub trait Environment {
    fn reset(&mut self, seed: Option<&[u64]>, options: Option<&Value>) {
        stub!()
    }
    fn reset_batch(
        &mut self,
        batch_indices: &[usize],
        seed: Option<&[u64]>,
        options: Option<&Value>,
    ) {
        stub!()
    }
    fn step(&mut self) -> (Vec<Value>, Vec<bool>, Vec<Value>) {
        stub!()
    }
    fn update_actions(&mut self) {
        stub!()
    }
    fn update_observations(&mut self) {
        stub!()
    }
    fn action_space(&self, agent: &str) -> Value {
        stub!()
    }
    fn observation_space(&self, agent: &str) -> Value {
        stub!()
    }
}
