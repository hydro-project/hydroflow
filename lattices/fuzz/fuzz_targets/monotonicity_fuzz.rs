#![no_main]

extern crate libfuzzer_sys;
use lattices::algebra::is_monotonic_single;
use libfuzzer_sys::fuzz_target;
use lattices_fuzz::utils; 

 
#[macro_use]
extern crate lattices_fuzz;
create_fuzz_functions!(utils::InputType, FUNCTIONS);


fuzz_target!(|data: &[u8]| {
    let required_bytes = std::mem::size_of::<utils::InputType>();

    if data.len() < required_bytes * 2 {
        println!("Not enough data for monotonicity test.");
        return;
    }
    let a = utils::InputType::from_le_bytes(data[0..required_bytes].try_into().expect("slice with incorrect length"));
    let b = utils::InputType::from_le_bytes(data[required_bytes..required_bytes * 2].try_into().expect("slice with incorrect length"));

    let result = is_monotonic_single(a, b, FUNCTIONS.q.unwrap_or(utils::default_q));

    println!("Monotonicity test result: {}", result);
}); 