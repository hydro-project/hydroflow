#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::is_monotonic_single;
use lattices_fuzz::algebra_functions::FuzzFunctions;
use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;

type InputType = u8;
static FUNCTIONS: Lazy<FuzzFunctions<InputType>> = Lazy::new(|| FuzzFunctions::new(
    |a: u8, b: u8| a ^ b,
    Some(|a: u8| a),
    Some(|a: u8, b: u8| a.wrapping_mul(b)),
));

fuzz_target!(|data: &[u8]| {
    // Check if there is enough data for the test (at least 2 values are needed)
    if data.len() < 2 {
        println!("Not enough data for monotonicity test.");
        return;
    }
    let a = data[0];
    let b = data[1];

    let result = is_monotonic_single(a, b, FUNCTIONS.g.unwrap());

    println!("Monotonicity test result: {}", result);
});
