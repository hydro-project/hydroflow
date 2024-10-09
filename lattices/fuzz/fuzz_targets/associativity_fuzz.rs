// #![no_main]

// extern crate libfuzzer_sys;
// use libfuzzer_sys::fuzz_target;
// use lattices::algebra::associativity_single;

// // Define your function f outside
// fn wrapping_add(x: u8, y: u8) -> u8 {
//     x.wrapping_add(y)
// }

// fuzz_target!(|data: &[u8]| {
//     if data.len() < 3 {
//         return; // Ensure there's enough data for the test
//     }

//     let a = data[0];
//     let b = data[1];
//     let c = data[2];

//     // Now you can use your wrapping_add function
//     let result = associativity_single(a, b, c, wrapping_add);
//     println!("Associativity test result: {}", result);
// });
#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::associativity_single;

pub fn fuzz_target(a: u8, b: u8, c: u8, f: fn(u8, u8) -> u8) {
    let result = associativity_single(a, b, c, f);
    println!("Associativity test result: {}", result);
}
