#![no_main]

use lattices::algebra::idempotency_single;
use libfuzzer_sys::{arbitrary::Unstructured, fuzz_target};
use lattices_fuzz::utils;
use std::fs::OpenOptions;
use std::io::Write;

#[macro_use]
extern crate lattices_fuzz;

create_fuzz_functions!(utils::InputType, FUNCTIONS);

fuzz_target!(|data: &[u8]| {
    let mut us = Unstructured::new(data);
    if let Ok(input) = us.arbitrary::<utils::TestingInput>() {
        let result = idempotency_single(input.i1.clone(), FUNCTIONS.f);
        // println!("Idempotency test result: {}", result);

        let log_file = if result {
            format!("fuzz_results/idempotency_PASS_{}.log", std::any::type_name::<utils::InputType>())
        } else {
            format!("fuzz_results/idempotency_FAIL_{}.log", std::any::type_name::<utils::InputType>())
        };
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open file");
        writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
    }
});
