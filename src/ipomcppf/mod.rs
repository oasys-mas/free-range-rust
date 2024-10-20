pub mod nodes;
pub mod particle_filter;

//use std::cell::RefCell;
//use std::rc::Rc;
//use std::any::Any;
//
//use crate::ipomcppf::nodes::{AsNode, BeliefNode};
//
//pub struct Configuration {
//    pub ucb_c: f64,
//    pub initial_trajectories: i16,
//    pub trajectories: i16,
//    pub num_particles: i16,
//    pub horizon: i16,
//    pub gamma: f64,
//    pub q_sensitivity: f64,
//}
//
//struct Planner {
//    has_initially_explored: bool,
//
//    config: Configuration,
//    tree: Rc<RefCell<BeliefNode>>,
//    last_action: Vec<i16>,
//}
//
//impl Planner {
//    pub fn search(&mut self) {
//        let trajectories = match self.has_initially_explored {
//            true => self.config.trajectories,
//            false => {
//                self.has_initially_explored = true;
//                self.config.initial_trajectories
//            }
//        };
//
//        for i in 0..trajectories {
//            if i % (trajectories / 10) == 0 {}
//            self.simulate(self.tree.clone(), 0);
//        }
//    }
//
//    fn simulate(&self, node: Rc<RefCell<BeliefNode>>, horizon: i16) {
//
//    }
//
//    fn rollout(&self, node: Rc<RefCell<BeliefNode>>, horizon: i16) {}
//
//    fn act(&self) -> Vec<i16> {
//        vec![]
//    }
//
//    fn observe(&self, observation: i64) {
//        //let tree_borrow = self.tree.borrow().get_child(observation);
//    }
//}
