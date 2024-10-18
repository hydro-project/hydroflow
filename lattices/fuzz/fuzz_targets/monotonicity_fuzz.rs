#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::is_monotonic_single;
use libfuzzer_sys::fuzz_target;


#[macro_use]
extern crate lattices_fuzz;

type InputType = u8;

create_fuzz_functions!(InputType, FUNCTIONS);

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
