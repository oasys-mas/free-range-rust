use simple_logger::SimpleLogger;
use std::time::Instant;

use free_range_rust::spaces::{Discrete, OneOf, VectorSpace};
use free_range_rust::Space;
use std::sync::Arc;

fn main() {
    SimpleLogger::new().without_timestamps().init().expect("failed to initialize logging");

    let discrete_spaces: Vec<Arc<dyn Space>> = vec![
        Arc::new(Discrete { n: 1, start: 0 }),
        Arc::new(Discrete { n: 1, start: 0 }),
        Arc::new(Discrete { n: 1, start: -1 }),
        Arc::new(Discrete { n: 1, start: -2 }),
        Arc::new(Discrete { n: 1, start: -3 }),
    ];

    let one_of_space: Arc<dyn Space> = Arc::new(OneOf { spaces: discrete_spaces });

    let vector_space: Arc<dyn Space> = Arc::new(VectorSpace { spaces: vec![one_of_space.clone(); 1000] });

    let start_time = Instant::now();

    for _ in 0..1_000_000 {
        vector_space.enumerate();
    }

    let duration = start_time.elapsed();
    println!("Time elapsed in enumerate_oneof: {duration:?}");
}
