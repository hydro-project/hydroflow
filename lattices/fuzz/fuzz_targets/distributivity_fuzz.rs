// #![no_main]
// extern crate libfuzzer_sys;
// use libfuzzer_sys::fuzz_target;
// use lattices::algebra::distributive_single;

// pub fn fuzz_target(data: &[u8], f: fn(u8, u8) -> u8, g: fn(u8, u8) -> u8) {

//     if data.len() < 3 {
//         println!("Not enough data for distributive test.");
//         return;
//     }

//     let a = data[0];
//     let b = data[1];
//     let c = data[2];

//     let result = distributive_single(a, b, c, f, g);
//     println!("Distributive test result: {}", result);
// }
#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::distributivity_single;

pub fn fuzz_target(a: u8, b: u8, f: fn(u8, u8) -> u8, g: fn(u8, u8) -> u8) {
    let result = distributivity_single(a, b, f, g);
    println!("Distributivity test result: {}", result);
}
