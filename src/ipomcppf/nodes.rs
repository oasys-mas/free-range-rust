use debug_tree::{default_tree, TreeConfig, TreeSymbols};
use log::debug;
use rand::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::rc::{Rc, Weak};

use crate::bindings::spaces::Space;
use crate::ipomcppf::particle_filter::ParticleFilter;

pub type Link<T> = Rc<RefCell<T>>;
pub type WeakLink<T> = Weak<RefCell<T>>;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum KeyMap {
    Belief(i64),
    Action(Vec<i16>),
}

pub trait NodeData: Debug + Display {
    fn as_belief(&self) -> Option<&BeliefNode> {
        None
    }

    fn as_action(&self) -> Option<&ActionNode> {
        None
    }
}

pub struct Node {
    pub data: Box<dyn NodeData>,
    pub parent: Option<WeakLink<Node>>,
    pub children: HashMap<KeyMap, Link<Node>>,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Node {
    pub fn new(data: Box<dyn NodeData>) -> Link<Self> {
        Rc::new(RefCell::new(Node {
            data,
            parent: None,
            children: HashMap::new(),
        }))
    }

    pub fn new_with_parent(data: Box<dyn NodeData>, key: KeyMap, parent: &Link<Node>) -> Link<Self> {
        let new_node = Self::new(data);
        new_node.borrow_mut().parent = Some(Rc::downgrade(parent));

        parent.borrow_mut().children.insert(key.clone(), new_node.clone());

        new_node
    }

    pub fn show(&self) {
        let default_tree = default_tree();
        default_tree.set_config_override(
            TreeConfig::new()
                .indent(4)
                .symbols(TreeSymbols::with_rounded().leaf("> ")),
        );

        self.show_helper(0, None);
        debug!("\n{}", default_tree.string());
    }

    pub fn show_with_depth(&self, target: i16) {
        let default_tree = default_tree();
        default_tree.set_config_override(
            TreeConfig::new()
                .indent(4)
                .symbols(TreeSymbols::with_rounded().leaf("> ")),
        );

        self.show_helper(0, Some(target));
        debug!("\n{}", default_tree.string());
    }

    pub fn to_string(&self) -> String {
        let default_tree = default_tree();
        default_tree.set_config_override(
            TreeConfig::new()
                .indent(4)
                .symbols(TreeSymbols::with_rounded().leaf("> ")),
        );

        self.show_helper(0, None);
        default_tree.string()
    }

    pub fn to_string_with_depth(&self, target: i16) -> String {
        let default_tree = default_tree();
        default_tree.set_config_override(
            TreeConfig::new()
                .indent(4)
                .symbols(TreeSymbols::with_rounded().leaf("> ")),
        );

        self.show_helper(0, Some(target));
        default_tree.string()
    }

    fn show_helper(&self, depth: i16, target: Option<i16>) {
        let target_depth = target.unwrap_or(i16::MAX);
        if depth > target_depth {
            return;
        }

        debug_tree::add_branch!("{}", self);

        let mut sorted_keys: Vec<_> = self.children.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(child) = self.children.get(key) {
                child.borrow().show_helper(depth + 1, target);
            }
        }

        if depth == 0 {
            let default_tree = default_tree();
            default_tree.set_config_override(
                TreeConfig::new()
                    .indent(4)
                    .symbols(TreeSymbols::with_rounded().leaf("> ")),
            );
        }
    }

    pub fn argmax_ucb(&self, ucb_c: f64) -> Link<Node> {
        let belief_node = match self.data.as_belief() {
            Some(belief) => belief,
            None => panic!("Cannot call argmax_ucb on a non-belief node."),
        };

        if belief_node.visits <= 0 {
            panic!("Cannot call argmax_ucb because this node has never been visited before.");
        }

        let mut max_ucb_value = f64::MIN;
        let mut best_child = None;

        for child in self.children.values() {
            let child_borrow = child.borrow();
            let action_node = child_borrow.data.as_action().expect("Failed to get action node.");

            // If the child has never been visited, it has an infinite UCB value.
            if action_node.visits == 0 {
                return child.clone();
            }

            let visit_factor = (f64::ln(action_node.visits as f64) / action_node.visits as f64).sqrt();
            let ucb_value = action_node.q_value + ucb_c * visit_factor;

            if ucb_value > max_ucb_value {
                max_ucb_value = ucb_value;
                best_child = Some(child.clone());
            }
        }

        best_child.expect("Failed to find the best child.")
    }

    pub fn argmax_q(&self, q_sensitivity: f64) -> Link<Node> {
        if self.data.as_belief().is_none() {
            panic!("Cannot call argmax_ucb on a non-belief node.");
        }

        let max_q_value = self.children.values().fold(f64::MIN, |max, child| {
            let q_value = child
                .borrow()
                .data
                .as_action()
                .expect("Failed to get action node.")
                .q_value;

            max.max(q_value)
        });

        let threshold = max_q_value - q_sensitivity;

        let mut rng = thread_rng();
        let best_child = self
            .children
            .values()
            .filter(|child| {
                child
                    .borrow()
                    .data
                    .as_action()
                    .map_or(false, |action_node| action_node.q_value >= threshold)
            })
            .choose(&mut rng)
            .expect("Failed to choose a random child.")
            .clone();

        best_child
    }
}

#[derive(Debug)]
pub struct BeliefNode {
    pub observation: i64,
    pub visits: i16,
    pub particle_filter: ParticleFilter<i64>,
}

impl NodeData for BeliefNode {
    fn as_belief(&self) -> Option<&BeliefNode> {
        Some(self)
    }
}

impl Display for BeliefNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "b [{}]: {{ visits={} }}", self.observation, self.visits)
    }
}

impl BeliefNode {
    pub fn new(observation: i64) -> Link<Node> {
        Node::new(Box::new(BeliefNode {
            observation,
            visits: 0,
            particle_filter: ParticleFilter::new(50, vec![0, 1], vec![0.5, 0.5]),
        }))
    }

    pub fn new_with_parent(observation: i64, parent: &Link<Node>) -> Link<Node> {
        Node::new_with_parent(
            Box::new(BeliefNode {
                observation,
                visits: 0,
                particle_filter: ParticleFilter::new(50, vec![0, 1], vec![0.5, 0.5]),
            }),
            KeyMap::Belief(observation),
            parent,
        )
    }

    pub fn create_action_nodes(parent: &Link<Node>, action_space: Space, partial_space: Vec<i16>) {
        match action_space {
            Space::Discrete { n, start, .. } => {
                for i in start..start + n {
                    let new_space = {
                        let mut new_partial_space = partial_space.clone();
                        new_partial_space.push(i as i16);
                        new_partial_space
                    };
                    ActionNode::new_with_parent(new_space, parent);
                }
            }
            Space::OneOf { spaces, .. } => {
                for (index, space) in spaces.into_iter().enumerate() {
                    let new_partial_space = {
                        let mut new_space = partial_space.clone();
                        new_space.push(index as i16);
                        new_space
                    };
                    Self::create_action_nodes(&parent, space, new_partial_space);
                }
            }
            Space::Vector { spaces, .. } => {
                for (index, space) in spaces.into_iter().enumerate() {
                    let new_partial_space = {
                        let mut new_space = partial_space.clone();
                        new_space.push(index as i16);
                        new_space
                    };
                    Self::create_action_nodes(&parent, space, new_partial_space);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ActionNode {
    pub action: Vec<i16>,
    pub visits: i16,
    pub q_value: f64,
}

impl NodeData for ActionNode {
    fn as_action(&self) -> Option<&ActionNode> {
        Some(self)
    }
}

impl ActionNode {
    pub fn new(action: Vec<i16>) -> Link<Node> {
        Node::new(Box::new(ActionNode {
            action,
            visits: 0,
            q_value: 0.0,
        }))
    }

    pub fn new_with_parent(action: Vec<i16>, parent: &Link<Node>) -> Link<Node> {
        Node::new_with_parent(
            Box::new(ActionNode {
                action: action.clone(),
                visits: 0,
                q_value: 0.0,
            }),
            KeyMap::Action(action),
            parent,
        )
    }
}

impl Display for ActionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "a {:?}: {{ visits={}, q_value={} }}",
            self.action, self.visits, self.q_value
        )
    }
}
