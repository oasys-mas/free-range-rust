#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use color_eyre::eyre::Result;

use crate::config::Configuration;

#[derive(Debug)]
pub struct WildfireConfiguration {
    pub num_envs: usize,

    pub grid: (u8, u8),

    pub max_fires: usize,
    pub max_agents: usize,
}

impl Configuration for WildfireConfiguration {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for WildfireConfiguration {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let num_envs = u.int_in_range(1..=2048 as usize)?;

        let grid = (u.int_in_range(1..=u8::MAX)?, u.int_in_range(1..=u8::MAX)?);

        let max_fires = u.int_in_range(1..=u8::MAX as usize)?;
        let max_agents = u.int_in_range(1..=u8::MAX as usize)?;

        Ok(WildfireConfiguration {
            num_envs,
            grid,
            max_fires,
            max_agents,
        })
    }
}
