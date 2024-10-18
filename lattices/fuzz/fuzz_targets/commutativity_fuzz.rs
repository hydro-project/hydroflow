#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::commutativity_single;
use libfuzzer_sys::fuzz_target;


#[macro_use]
extern crate lattices_fuzz;

type InputType = u8;

create_fuzz_functions!(InputType, FUNCTIONS);

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
