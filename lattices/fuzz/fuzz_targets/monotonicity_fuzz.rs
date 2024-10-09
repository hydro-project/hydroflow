#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::is_monotonic_single;


// Define the fuzz target for monotonicity property
pub fn fuzz_target(data: &[u8], f: fn(u8, u8) -> u8) {
    // Check if there is enough data for the test (at least 2 values are needed)
    if data.len() < 2 {
        println!("Not enough data for monotonicity test.");
        return;
    }
    let a = data[0];
    let b = data[1]; 

    let result = is_monotonic_single(a, b, f);

    println!("Monotonicity test result: {}", result);
}
 