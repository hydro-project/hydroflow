#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::distributive_single;
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
    if data.len() < 3 {
        println!("Not enough data for distributivity test.");  
        return;
    }

    let a = data[0];
    let b = data[1];
    let c = data[2];

    if let Some(q) = FUNCTIONS.q {
        let result = distributive_single(a, b, c, FUNCTIONS.f, q);
        println!("Distributivity test result: {}", result);
    } else {
        println!("Skipping distributivity test because g is not available.");
    }
});
