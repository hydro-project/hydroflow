#![no_main]
use lattices::algebra::commutativity_single;
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
        let result = commutativity_single(input.i1.clone(), input.i2.clone(), FUNCTIONS.f);
        let log_file = if result {
            format!("fuzz_results/commutativity_PASS_{}.log", std::any::type_name::<utils::InputType>())
        } else {
            format!("fuzz_results/commutativity_FAIL_{}.log", std::any::type_name::<utils::InputType>())
        };
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .expect("Unable to open file");
        writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
    }
});
