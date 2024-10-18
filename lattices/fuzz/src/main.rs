 use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let fuzz_targets = vec![
        // "commutativity_fuzz",
        // "associativity_fuzz",
        // "linearity_fuzz",
        // "monotonicity_fuzz",
        "distributivity_fuzz",
    ];

    for fuzz_target in fuzz_targets {
        println!("Running fuzz target: {}", fuzz_target);

        let status = Command::new("cargo")
            .arg("fuzz")
            .arg("run")
            .arg(fuzz_target)
            .arg("--")
            .arg("-max_total_time=1") // Run each fuzz target for 5 seconds
            .current_dir(&manifest_dir)  
            .status()
            .expect("Failed to execute fuzz target");

        if !status.success() {
            eprintln!("Fuzz target failed: {}", fuzz_target);
            std::process::exit(1);
        }
    }

    println!("All fuzz targets ran successfully.");
}