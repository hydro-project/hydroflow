 

#![no_main]


use libfuzzer_sys::fuzz_target; 

use lattices::algebra::associativity_single; 
use lattices::algebra::linearity_single;
use lattices::algebra::commutativity_single; 
use lattices::algebra::distributive_single;


 
fuzz_target!(|data: &[u8]| {
    // Define your function f here inside the fuzz target
    let f = |x: u8, y: u8| x.wrapping_add(y); // Use wrapping addition
    let q = |x: u8| x; // Identity function
    let g = |x: u8, y: u8| x.wrapping_add(y); // Use wrapping addition


    if data.len() < 3 {
        return; // Ensure there's enough data for the fuzz target
    }

    let a = data[0];
    let b = data[1];
    let c = data[2];

    // Running all property tests
    println!("Running associativity fuzz test");
    let associativity_result = associativity_single(a, b, c, f);
    println!("Associativity test result: {}", associativity_result);

    println!("Running commutativity fuzz test");
    let commutativity_result = commutativity_single(a, b, f);
    println!("Commutativity test result: {}", commutativity_result);
 
    println!("Running linearity fuzz test");
    let linearity_result = linearity_single(a, b, f, q, g);
    println!("Linearity test result: {}", linearity_result);

    println!("Running distributivity fuzz test");
    let distributivity_result = distributive_single(a, b, c, f, g);
    println!("Distributivity test result: {}", distributivity_result);
});
