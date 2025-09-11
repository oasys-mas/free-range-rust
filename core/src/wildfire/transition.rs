use crate::wildfire::state::WildfireState;
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

#[derive(Clone)]
pub struct Sample {
    pub action_type: String,
    pub data: i32,
}

pub trait Transition {
    fn apply(
        &self,
        state: WildfireState,
        actions: &HashMap<String, Vec<Sample>>,
        outputs: &HashMap<String, Box<dyn Any>>,
    ) -> Result<(WildfireState, HashMap<String, Box<dyn Any>>), Error>;
}

pub trait Environment {
    fn step(&mut self, actions: &HashMap<String, Vec<Sample>>) -> Result<(), Error>;
}

pub struct WildfireEnvironment {
    pub state: WildfireState,
    pub transitions: Vec<Box<dyn Transition>>,
    pub outputs: HashMap<String, Box<dyn Any>>,
}

impl WildfireEnvironment {
    pub fn new(state: WildfireState, transitions: Vec<Box<dyn Transition>>) -> Self {
        WildfireEnvironment {
            state,
            transitions,
            outputs: HashMap::new(),
        }
    }
}

impl Environment for WildfireEnvironment {
    fn step(&mut self, actions: &HashMap<String, Vec<Sample>>) -> Result<(), Error> {
        let mut state = self.state.clone();
        let mut all_outputs: HashMap<String, Box<dyn Any>> = HashMap::new();

        for transition in &self.transitions {
            match transition.apply(state, actions, &all_outputs) {
                Ok((new_state, outputs)) => {
                    state = new_state;
                    all_outputs.extend(outputs);
                }
                Err(e) => return Err(e),
            }
        }

        self.state = state;
        self.outputs = all_outputs;
        Ok(())
    }
}

pub struct ExampleTransition;

impl Transition for ExampleTransition {
    fn apply(
        &self,
        mut state: WildfireState,
        actions: &HashMap<String, Vec<Sample>>,
        outputs: &HashMap<String, Box<dyn Any>>,
    ) -> Result<(WildfireState, HashMap<String, Box<dyn Any>>), Error> {
        // Example: increment all agent suppressant by 1 if "refill" action present
        if let Some(samples) = actions.get("refill") {
            for _sample in samples {
                for s in state.agent.suppressant.iter_mut() {
                    *s += 1;
                }
            }
        }

        let mut new_outputs = HashMap::new();
        new_outputs.insert(
            "refilled".to_string(),
            Box::new(state.agent.suppressant.clone()),
        );
        Ok((state, new_outputs))
    }
}
