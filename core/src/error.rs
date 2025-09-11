use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("Capacity exceeded: attempted {attempted}, max {max}")]
    CapacityExceeded { attempted: usize, max: usize },
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
