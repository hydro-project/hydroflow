#!/bin/bash

# Ensure the fuzzer is built
echo "Building the fuzzer..."
if ! cargo fuzz build; then
    echo "Fuzzer build failed!"
    exit 1
fi

# Run the fuzzer
echo "Starting fuzz testing..."
cargo fuzz run all_fuzz

# Check if fuzzing output exists (AFL usually saves crashes in `crashes/`)
output_dir="fuzz/artifacts/all_fuzz/crashes"
if [ -d "$output_dir" ]; then
    num_crashes=$(ls -1q "$output_dir" | wc -l)
    if [ "$num_crashes" -gt 0 ]; then
        echo "Fuzzing found $num_crashes crash(es). Check the $output_dir folder."
    else
        echo "No crashes found."
    fi
else
    echo "No crash output directory found. Fuzzing may not have finished yet."
fi
