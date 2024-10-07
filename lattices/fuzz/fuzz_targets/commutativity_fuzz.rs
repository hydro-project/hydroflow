#![no_main]
extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::commutativity_single;

pub fn fuzz_target(data: &[u8], f: fn(u8, u8) -> u8) {
    if data.len() < 2 {
        return;
    }

    let a = data[0];
    let b = data[1];

    let result = commutativity_single(a, b, f);
    println!("Commutativity test result: {}", result);
}
