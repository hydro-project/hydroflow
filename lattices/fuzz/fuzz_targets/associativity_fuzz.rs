#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::associativity_single;
use once_cell::sync::Lazy;
use lattices_fuzz::algebra_functions::FuzzFunctions;

type InputType = u8;
static FUNCTIONS: Lazy<FuzzFunctions<InputType>> = Lazy::new(|| FuzzFunctions::new(
    |a: u8, b: u8| a ^ b,
    Some(|a: u8| a),
    Some(|a: u8, b: u8| a.wrapping_mul(b)),
));

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        println!("Not enough data for associativity test.");
        return;
    }

    let a = data[0];
    let b = data[1];
    let c = data[2];

    let result = associativity_single(a, b, c, FUNCTIONS.f);
    println!("Associativity test result: {}", result);
});