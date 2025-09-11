use color_eyre::Result;

pub trait Configuration {
    fn validate(&self) -> Result<()>;
}
