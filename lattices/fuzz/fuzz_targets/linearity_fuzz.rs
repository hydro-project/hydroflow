#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::linearity_single;
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
        println!("Not enough data for linearity test.");
        return;
    }

    let a = data[0];
    let b = data[1];

    if let (Some(g), Some(q)) = (FUNCTIONS.g, FUNCTIONS.q) {
        let result = linearity_single(a, b, FUNCTIONS.f, g, q);  
        println!("Linearity test result: {}", result);
    } else {
        println!("Skipping linearity test because g or q is not available.");
    }
});

