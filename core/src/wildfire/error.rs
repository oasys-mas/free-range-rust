use thiserror::Error;

#[derive(Debug, Error)]
pub enum WildfireError {
    #[error("Agent index out of bounds: {0}")]
    AgentIndexOutOfBounds(usize),
    #[error("Fire index out of bounds: {0}")]
    FireIndexOutOfBounds(usize),
    #[error("Agent capacity exceeded: attempted {attempted}, max {max}")]
    AgentCapacityExceeded { attempted: usize, max: usize },
    #[error("Fire capacity exceeded: attempted {attempted}, max {max}")]
    FireCapacityExceeded { attempted: usize, max: usize },
    #[error("Invalid wildfire operation: {0}")]
    InvalidWildfireOperation(String),
    #[error(transparent)]
    Core(#[from] crate::error::CoreError),
}
