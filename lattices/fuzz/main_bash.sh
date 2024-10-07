#!/bin/bash

# Number of times to run the script
num_times=25

# Output file
output_file="fuzz_results.txt"

# Loop to run the script multiple times
for ((i = 1; i <= num_times; i++)); do
    ./single_fuzz.sh >> "$output_file"
done
