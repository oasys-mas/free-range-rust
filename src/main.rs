use simple_logger::SimpleLogger;

use free_range_rust::Space;

fn main() {
    let _ = SimpleLogger::new().without_timestamps().init().unwrap();

    //let head = BeliefNode::new(0);
    let action_space = Space::new_one_of(vec![
        Space::new_discrete(3, 0),
        Space::new_discrete(3, 0),
        Space::new_one_of(vec![
            Space::new_discrete(2, 0),
            Space::new_one_of(vec![Space::new_discrete(5, 0)]),
        ]),
    ]);

    //BeliefNode::create_action_nodes(&head, action_space.clone(), vec![]);

    //let head_borrow = head.borrow();
    //head_borrow.show();
    //head_borrow.show_with_depth(4);

    //let pf = &head_borrow.data.as_belief().unwrap().particle_filter;
    //println!("{}", pf);
}
