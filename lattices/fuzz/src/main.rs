
use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Output};


const ALGEBRAIC_PROPERTIES: [&'static str; 3] = ["associativity", "commutativity", "idempotence"]; 

fn generate_fuzz_functions(udf: &str) -> anyhow::Result<()> {

	let path = "lattices/fuzz/src/fuzz_functions_integrated.rs";
	let mut fuzz_functions_file = fs::File::create(path)?; 
	let prefix = fs::read_to_string("lattices/fuzz/fuzz_functions_prefix.rs")?;
	let postfix = fs::read_to_string("lattices/fuzz/fuzz_functions_postfix.rs")?;

	fuzz_functions_file.write_all(prefix.as_bytes())?;
    fuzz_functions_file.write_all(udf.as_bytes())?;  
    fuzz_functions_file.write_all(postfix.as_bytes())?; 

	Ok(())
}

fn generate_utils(datatype_signature: &str) -> anyhow::Result<()> { 
	let path = "lattices/fuzz/src/utils_integrated.rs";
	let mut utils_file = fs::File::create(path)?; 
	let prefix = fs::read_to_string("lattices/fuzz/utils_prefix.rs")?;
	let postfix = fs::read_to_string("lattices/fuzz/utils_postfix.rs")?;

	utils_file.write_all(prefix.as_bytes())?;
	utils_file.write_all(datatype_signature.as_bytes())?; 
    utils_file.write_all(postfix.as_bytes())?; 

	Ok(())
}

fn find_fuzz_target(property: &str) -> &str {
		match property {
		"associativity" => return "associativity_fuzz",
		"commutativity" => return "commutativity_fuzz",
		"idempotence" => return "idempotence_fuzz",
		"monotonicity" => return "monotonicity_fuzz",
		"distributivity" => return "distributivity_fuzz",
		"linearity" => return "linearity_fuzz",
		_ => return "Fuzz target not found",
	};
}

// fn process_fold(subtree: syn::Expr, datatype_signature: &str, property: &str) -> anyhow::Result<()> { 
fn process_fold(datatype_signature: &str, property: &str, udf: &str) -> Output { 
	// let token_stream = subtree.to_token_stream();
	// generate_fuzz_functions(token_stream.to_string())?;
	generate_fuzz_functions(udf); // replace the hardcoded function def token_stream.
	generate_utils(datatype_signature); 
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
	let fuzz_target = find_fuzz_target(&property);
	let output = Command::new("cargo")
		.arg("fuzz")
		.arg("run")
		.arg(fuzz_target)
		.arg("--")
		.arg("-max_total_time=1") // Run each fuzz target for 1 second
		.arg("--ignore-timeouts") // Ignore timeouts
		.arg("--ignore-ooms") // Ignore out of memory errors
		.current_dir(&manifest_dir)
		.output()
		.expect("Failed to execute fuzz target");

	if !output.status.success() {
		eprintln!("Fuzz target failed: {}", fuzz_target);

		let mut file = File::create(format!("{}_failure_output.txt", fuzz_target))
			.expect("Failed to create output file");

		file.write_all(&output.stdout).expect("Failed to write stdout to file");
		file.write_all(&output.stderr).expect("Failed to write stderr to file");

		// Remember that the property failed
		let log_path = format!("{}/fuzz_results/{}_FAIL_{}.log", manifest_dir, property, datatype_signature);
		File::create(log_path).expect("Failed to create log file");
	}
	output

}
 
fn main() -> anyhow::Result<()> {
	// deletes all the files in the fuzz_results folders from previous runs.
	let fuzz_results_path = format!("{}/fuzz_results", std::env::var("CARGO_MANIFEST_DIR")?);
	if std::path::Path::new(&fuzz_results_path).exists() {
		fs::remove_dir_all(&fuzz_results_path)?;
	}
	fs::create_dir(&fuzz_results_path)?;

	// only part to change is here! :) specify your datatype signature and udf.
	let datatype_signature = "u128"; 
	let udf = "| x , y | x . wrapping_add (y)"; 

	print!("{}", format!("HERE ARE THE TEST RESULTS for your udf {}! \n", udf));
	
	for property in ALGEBRAIC_PROPERTIES.iter() {
		process_fold(datatype_signature, property, udf);
	 
		let fail_path = format!("{}/fuzz_results/{}_FAIL_{}.log", std::env::var("CARGO_MANIFEST_DIR").unwrap(), property, datatype_signature);
		let panic_path = format!("{}/fuzz_results/{}_PANIC_{}.log", std::env::var("CARGO_MANIFEST_DIR").unwrap(), property, datatype_signature);
		// print!("{}", fail_path);
		if std::path::Path::new(&panic_path).exists() {
			println!("{} property panicked and failed", property);
		} else if std::path::Path::new(&fail_path).exists() {
			println!("{} property failed", property);
		} else {
			println!("{} property passed", property);
		}
	}
	Ok(()) 

	

}