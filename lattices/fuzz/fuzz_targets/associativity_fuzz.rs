#![no_main]
use lattices::algebra::associativity_single;
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
            associativity_single(input.i1.clone(), input.i2.clone(), input.i3.clone(), FUNCTIONS.f)
        });

        match result {
            Ok(result) => {
                println!("Associativity test result: {}", result);
                let log_file = if result {
                    format!("fuzz_results/associativity_PASS_{}.log", std::any::type_name::<utils_integrated::InputType>())
                } else {
                    format!("fuzz_results/associativity_FAIL_{}.log", std::any::type_name::<utils_integrated::InputType>())
                };
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Unable to open file");
                writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
            }
            Err(_) => {
                let log_file = format!("fuzz_results/associativity_PANIC_{}.log", std::any::type_name::<utils_integrated::InputType>());
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Unable to open file");
                writeln!(file, "Input: {:?}", input).expect("Unable to write to file");
                println!("A panic occurred during the associativity test.");
            }
        }
    }
});
