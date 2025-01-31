use simple_logger::SimpleLogger;
use std::time::Instant;

use free_range_rust::Space;

fn main() {
    let _ = SimpleLogger::new().without_timestamps().init().unwrap();

    let discrete_spaces = vec![
        Space::Discrete { n: 1, start: 0 },
        Space::Discrete { n: 1, start: 0 },
        Space::Discrete { n: 1, start: -1 },
        Space::Discrete { n: 1, start: -2 },
        Space::Discrete { n: 1, start: -3 },
    ];

    // Create the `OneOf` space, which contains the Discrete spaces
    let one_of_space = Space::OneOf { spaces: discrete_spaces };

    // Create the `Vector` space, which is a repetition of the `OneOf` space 10 times
    let vector_space = Space::Vector {
        spaces: vec![one_of_space; 1000], // Repeat `OneOf` 10 times
    };

    let start_time = Instant::now();

    for _ in 0..1_000_000 {
        vector_space.enumerate_nested();
    }

    let duration = start_time.elapsed();

    println!("Time elapsed in enumerate_oneof: {:?}", duration);

    //BeliefNode::create_action_nodes(&head, action_space.clone(), vec![]);

    //let head_borrow = head.borrow();
    //head_borrow.show();
    //head_borrow.show_with_depth(4);

    //let pf = &head_borrow.data.as_belief().unwrap().particle_filter;
    //println!("{}", pf);
}
