use color_eyre::Result;
use std::any::Any;
use std::collections::HashMap;

use crate::spaces::Sample;
use crate::state::State;

type EnvironmentOutput = HashMap<String, Box<dyn Any>>;

pub trait Transition<S: for<'a> State<'a>> {
    fn apply(
        &self,
        state: &mut S,
        actions: &HashMap<String, Vec<Sample>>,
        outputs: &EnvironmentOutput,
    ) -> Result<EnvironmentOutput>;
}
