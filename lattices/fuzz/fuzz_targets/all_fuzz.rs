mod associativity_fuzz;
mod commutativity_fuzz;
mod idempotence_fuzz;
mod linearity_fuzz;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: run_all <input file>");
        return;
    }
    let input = std::fs::read(&args[1]).expect("Failed to read input file");

    
    // Define your function f here
    let f: fn(u8, u8) -> u8 = |x, y| x.wrapping_add(y);  // Example: addition

    // Define q as a transformation function (not wrapped in Option)
    let q: fn(u8) -> f64 = |x| x as f64; // Example: identity function

    // Example g function definition
    let g: fn(f64, f64) -> f64 = |x, y| x + y; // Define g as needed 
    
    // Run associativity, commutativity, and idempotency tests
    println!("Running associativity fuzz test");
    associativity_fuzz::fuzz_target(&input, f);

    println!("Running commutativity fuzz test");
    commutativity_fuzz::fuzz_target(&input, f);

    println!("Running idempotency fuzz test");
    idempotence_fuzz::fuzz_target(&input, f);

    // Run linearity fuzz test if q is provided

    println!("Running linearity fuzz test");
    linearity_fuzz::fuzz_target(&input, f, q, g); // Pass g here

}
