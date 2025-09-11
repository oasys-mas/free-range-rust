use crate::transition::Transition;
use crate::wildfire::state::WildfireState;

pub type WildfireTransition<'a> = dyn Transition<WildfireState<'a>>;
