#!/bin/bash

echo "Building the fuzzer..."
if ! cargo fuzz build; then
    echo "Fuzzer build failed!"
    exit 1
fi

echo "Starting fuzz testing for 15 seconds..."
cargo fuzz run all_fuzz -- -max_total_time=3

output_dir="fuzz/artifacts/all_fuzz/crashes"
if [ -d "$output_dir" ]; then
    num_crashes=$(ls -1q "$output_dir" | wc -l)
    if [ "$num_crashes" -gt 0 ]; then
        echo "Fuzzing found $num_crashes crash(es). Check the $output_dir folder."
        # echo "Crash log example:"
        # head -n 20 "$output_dir/crash-0"
    else
        echo "No crashes found."
    fi
else
    echo "No crash output directory found. Fuzzing may not have finished yet."
fi
