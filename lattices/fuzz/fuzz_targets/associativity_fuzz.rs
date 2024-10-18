#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::associativity_single;
use libfuzzer_sys::fuzz_target;
use lattices_fuzz::utils; 

#[macro_use]
extern crate lattices_fuzz;

create_fuzz_functions!(utils::InputType, FUNCTIONS);

fuzz_target!(|data: &[u8]| {
    let required_bytes = std::mem::size_of::<utils::InputType>();

    if data.len() < required_bytes * 3 {
        println!("Not enough data for associativity test.");
        return;
    }

    let a = utils::InputType::from_le_bytes(data[0..required_bytes].try_into().expect("slice with incorrect length"));
    let b = utils::InputType::from_le_bytes(data[required_bytes..required_bytes * 2].try_into().expect("slice with incorrect length"));
    let c = utils::InputType::from_le_bytes(data[required_bytes * 2..required_bytes * 3].try_into().expect("slice with incorrect length"));


    let result = associativity_single(a, b, c, FUNCTIONS.f);
    println!("Associativity test result: {}", result);
});