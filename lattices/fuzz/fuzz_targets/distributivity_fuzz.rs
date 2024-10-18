#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::distributive_single;
use libfuzzer_sys::fuzz_target;


#[macro_use]
extern crate lattices_fuzz;

type InputType = u8;

create_fuzz_functions!(InputType, FUNCTIONS);

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
