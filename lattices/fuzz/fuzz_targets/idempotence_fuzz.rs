#![no_main]
extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::idempotency_single;

pub fn fuzz_target(data: &[u8], f: fn(u8, u8) -> u8) {
    // Check if there is enough data for the test
    if data.len() < 1 {
        println!("Not enough data for idempotency test.");
        return;
    }

    // Extract the value for testing
    let a = data[0]; // Using the first byte as input

    // Call the idempotency check with the function f
    let result = idempotency_single(a, f); // Make sure to use the correct function signature
    println!("Idempotency test result: {}", result);
}



 
 