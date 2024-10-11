#![no_main]

use lattices::algebra::{
    associativity_single, commutativity_single, distributive_single, linearity_single,
};
use libfuzzer_sys::fuzz_target;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

fuzz_target!(|data: &[u8]| {

    let crash_dir = "./artifacts/all_fuzz/crashes/";

    // Create the directory if it doesn't exist
    create_dir_all(crash_dir).expect("Failed to create crash log directory");

    // Create a unique filename using the current timestamp
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let log_file_path = format!("{}crashes_{}.txt", crash_dir, timestamp);

    // Open the crash log file 
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&log_file_path)
        .expect("Failed to open crash log file");

    if data.len() < 3 {
        writeln!(file, "Insufficient data length: expected at least 3 bytes").unwrap();
        return; // Ensure there's enough data for the fuzz target
    }

    let a = data[0];
    let b = data[1];
    let c = data[2];

    // Define the functions for testing
    let f = |x: u8, y: u8| x ^ y;              // XOR operation
    let g = |x: u8, y: u8| x.wrapping_mul(y);  // Multiplication modulo 256
    let q = |x: u8| x;                         // Identity function


    let associativity_result = associativity_single(a, b, c, f);
    if !associativity_result {
        writeln!(file, "Associativity test failed with inputs: a={}, b={}, c={}", a, b, c).unwrap();
    }

    let commutativity_result = commutativity_single(a, b, f);
    if !commutativity_result {
        writeln!(file, "Commutativity test failed with inputs: a={}, b={}", a, b).unwrap();
    }

    let linearity_result = linearity_single(a, b, f, q, g);
    if !linearity_result {
        writeln!(file, "Linearity test failed with inputs: a={}, b={}", a, b).unwrap();
    }

    let distributivity_result = distributive_single(a, b, c, f, g);
    if !distributivity_result {
        writeln!(file, "Distributivity test failed with inputs: a={}, b={}, c={}", a, b, c).unwrap();
    }
});
