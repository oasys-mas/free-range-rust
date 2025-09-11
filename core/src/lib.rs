pub mod backend;
pub mod cuda;
pub mod env;
pub mod logging;
pub mod macros;
pub mod simd;
pub mod wildfire;

pub fn hello() -> &'static str {
    "Hello from core!"
}
