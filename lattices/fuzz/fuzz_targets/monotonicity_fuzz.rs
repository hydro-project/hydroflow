#![no_main]

use lattices::algebra::is_monotonic_single;
use libfuzzer_sys::{arbitrary::Unstructured, fuzz_target};
use lattices_fuzz::utils_integrated;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;

#[macro_use]
extern crate lattices_fuzz;

create_fuzz_functions!(utils_integrated::InputType, FUNCTIONS);

fuzz_target!(|data: &[u8]| {
    let mut us = Unstructured::new(data);
    if let Ok(input) = us.arbitrary::<utils_integrated::TestingInput>() {
        let result = panic::catch_unwind(|| {
            is_monotonic_single(input.i1.clone(), input.i2.clone(), FUNCTIONS.q.unwrap_or(utils_integrated::default_q))
        });

        match result {
            Ok(result) => {
                println!("Monotonicity test result: {}", result);
                let log_file = if result {
                    format!("fuzz_results/monotonicity_PASS_{}.log", std::any::type_name::<utils_integrated::InputType>())
                } else {
                    format!("fuzz_results/monotonicity_FAIL_{}.log", std::any::type_name::<utils_integrated::InputType>())
                };
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Unable to open file");
                writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
            }
            Err(_) => {
                let log_file = format!("fuzz_results/monotonicity_PANIC_{}.log", std::any::type_name::<utils_integrated::InputType>());
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Unable to open file");
                writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
                println!("A panic occurred during the monotonicity test.");
            }
        }
    }
});
