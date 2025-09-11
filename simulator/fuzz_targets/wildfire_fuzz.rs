#![no_main]

use bumpalo::Bump;
use free_range_rust::env::SimulatedEnvironment;
use free_range_rust::wildfire::WildfireEnvironment;
use free_range_rust::wildfire::config::WildfireConfiguration;
use libfuzzer_sys::fuzz_target;

use libfuzzer_sys::Corpus;

fuzz_target!(|input: (WildfireConfiguration, u64)| -> Corpus {
    let (config, seed) = input;

    let arena = Bump::new();
    let mut env = match WildfireEnvironment::new(config, &arena) {
        Ok(env) => env,
        Err(_) => return Corpus::Reject,
    };

    env.reset_seeded(seed).unwrap();

    Corpus::Keep
});
