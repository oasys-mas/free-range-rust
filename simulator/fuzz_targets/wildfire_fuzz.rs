#![no_main]

use bumpalo::Bump;
use core::env::SimulatedEnvironment;
use core::wildfire::WildfireEnvironment;
use core::wildfire::config::WildfireConfiguration;
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
