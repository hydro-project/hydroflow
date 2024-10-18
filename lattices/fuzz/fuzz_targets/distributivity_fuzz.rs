#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::commutativity_single;
use libfuzzer_sys::fuzz_target;

#[macro_use]
extern crate lattices_fuzz;

type InputType = u16;

create_fuzz_functions!(InputType, FUNCTIONS);

fuzz_target!(|data: &[u8]| {
    let required_bytes = std::mem::size_of::<InputType>();

    if data.len() < required_bytes * 2 {
        println!("Not enough data for commutativity test.");
        return;
    }
    let a = InputType::from_le_bytes(data[0..required_bytes].try_into().expect("slice with incorrect length"));
    let b = InputType::from_le_bytes(data[required_bytes..required_bytes * 2].try_into().expect("slice with incorrect length"));

    let result = commutativity_single(a, b, FUNCTIONS.f);

    println!("Commutativity test result: {}", result);
});