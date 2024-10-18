#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::linearity_single;
use libfuzzer_sys::fuzz_target;


#[macro_use]
extern crate lattices_fuzz;

type InputType = u8;

create_fuzz_functions!(InputType, FUNCTIONS);

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
