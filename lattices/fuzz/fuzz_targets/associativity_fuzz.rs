#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::associativity_single;

pub fn fuzz_target(data: &[u8], f: fn(u8, u8) -> u8) {
    if data.len() < 3 {
        println!("Not enough data for associativity test.");
        return;
    }

    let a = data[0];
    let b = data[1];
    let c = data[2];

    let result = associativity_single(a, b, c, f);
    println!("Associativity test result: {}", result);
}
