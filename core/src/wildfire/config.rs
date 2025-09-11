pub struct WildfireConfig {
    pub grid_size: (usize, usize),
}

impl WildfireConfig {
    fn new(grid_size: (usize, usize)) -> Self {
        WildfireConfig { grid_size }
    }
}
