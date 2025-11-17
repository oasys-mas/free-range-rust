use bumpalo::Bump;
use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use free_range_rust::env::SimulatedEnvironment;
use free_range_rust::wildfire::WildfireEnvironment;
use free_range_rust::wildfire::config::WildfireConfiguration;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "simulator")]
#[command(about = "Simulation CLI suite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// run wildfire simulation
    Wildfire {
        /// path to environment config file (JSON)
        #[arg(short, long)]
        config: String,
        /// seed for randomization
        #[arg(short, long)]
        seed: Option<u64>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Wildfire { config, seed } => {
            let data = fs::read_to_string(Path::new(&config))?;
            let config: WildfireConfiguration = serde_json::from_str(&data)?;

            let arena = Bump::new();
            let mut env =
                WildfireEnvironment::new(config, &arena).expect("unable to initialize environment");

            if let Some(seed) = seed {
                env.reset_seeded(seed)?;
            } else {
                env.reset()?;
            }

            println!("simulation complete");
        }
    }

    Ok(())
}
