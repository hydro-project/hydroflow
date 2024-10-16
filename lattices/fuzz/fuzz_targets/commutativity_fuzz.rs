#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::commutativity_single;
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
    if data.len() < 2 {
        println!("Not enough data for commutativity test.");
        return;
    }

    let a = data[0];
    let b = data[1];

    let result = commutativity_single(a, b, FUNCTIONS.f);

    println!("Commutativity test result: {}", result);
});
