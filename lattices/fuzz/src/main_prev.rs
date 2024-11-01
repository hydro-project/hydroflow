use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let fuzz_targets = vec![
        "commutativity_fuzz",
        "associativity_fuzz",
        "linearity_fuzz",
        "monotonicity_fuzz",
        "distributivity_fuzz",
        "idempotence_fuzz",
    ];

    for fuzz_target in fuzz_targets {
        println!("Running fuzz target: {}", fuzz_target);

        let output = Command::new("cargo")
            .arg("fuzz")
            .arg("run")
            .arg(fuzz_target)
            .arg("--")
            .arg("-max_total_time=1") // Run each fuzz target for 1 second
            .current_dir(&manifest_dir)
            .output()
            .expect("Failed to execute fuzz target");

        if !output.status.success() {
            eprintln!("Fuzz target failed: {}", fuzz_target);

            let mut file = File::create(format!("{}_failure_output.txt", fuzz_target))
                .expect("Failed to create output file");

            file.write_all(&output.stdout).expect("Failed to write stdout to file");
            file.write_all(&output.stderr).expect("Failed to write stderr to file");

            std::process::exit(1);
        }
    }

    println!("All fuzz targets ran successfully.");
}