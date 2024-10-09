#![no_main]
extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;
use lattices::algebra::linearity_single;  

// Define the fuzz target
pub fn fuzz_target(
    data: &[u8],
    f: fn(u8, u8) -> u8,       
    q: fn(u8) -> u8,  
    g: fn(u8, u8) -> u8,     
) {
    if data.len() < 3 {
        println!("Not enough data for linearity test.");
        return;
    }

    let a = data[0];
    let b = data[1];

    let result = linearity_single(a, b, f, q, g); 
    println!("Linearity test result: {}", result);
}