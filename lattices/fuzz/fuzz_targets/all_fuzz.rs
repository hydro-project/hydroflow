

// #[macro_use]
// extern crate afl;
// extern crate url;

// mod associativity_fuzz;
// // mod commutativity_fuzz;
// // mod idempotence_fuzz;
// // mod linearity_fuzz;
// // mod distributivity_fuzz;

// fn main() {
//     // Define your function f here
//     let f: fn(u8, u8) -> u8 = |x, y| x.wrapping_add(y); // Example: addition

//     // Define q as a transformation function
//     let q: fn(u8) -> u8 = |x| x; // Example: identity function

//     // Example g function definition
//     let g: fn(u8, u8) -> u8 = |x, y| x + y; // Example g function

//     fuzz!(|data: &[u8]| {
//         if data.len() < 2 {
//             return; // Ensure there's enough data for the fuzz target
//         }

//         println!("Running associativity fuzz test");
//         associativity_fuzz::fuzz_target(data);

//         // println!("Running commutativity fuzz test");
//         // commutativity_fuzz::fuzz_target(data, f);

//         // println!("Running idempotency fuzz test");
//         // idempotence_fuzz::fuzz_target(data, f);

//         // println!("Running linearity fuzz test");
//         // linearity_fuzz::fuzz_target(data, f, q, g);

//         // println!("Running distributivity fuzz test");
//         // distributivity_fuzz::fuzz_target(data, f, g);
//     });
// }


#![no_main]


use libfuzzer_sys::fuzz_target; 

use lattices::algebra::associativity_single; 
use lattices::algebra::linearity_single;
use lattices::algebra::commutativity_single; 
use lattices::algebra::distributive_single;




// mod associativity_fuzz;
// mod commutativity_fuzz;
// mod idempotence_fuzz;
// mod linearity_fuzz;
// mod distributivity_fuzz;
// Import the necessary fuzzing macro
// Import the necessary fuzzing macro

// Ensure your function definitions are included
// fn associativity_single(...), fn commutativity_single(...), etc.

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
