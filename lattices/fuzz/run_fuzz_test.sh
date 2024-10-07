#!/bin/bash

# Check if the input file is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input_file>"
    exit 1
fi

INPUT_FILE=$1

# Check if the input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Input file '$INPUT_FILE' not found!"
    exit 1
fi

# Navigate to the project directory (replace with your actual path)
cd /Users/admin/research/hydroflow/lattices/fuzz || {
    echo "Failed to navigate to project directory."
    exit 1
}

# Build the project
echo "Building the project..."
cargo build

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Run the project with the input file
echo "Running fuzz tests with input file '$INPUT_FILE'..."
cargo run -- "$INPUT_FILE"

if [ $? -ne 0 ]; then
    echo "Fuzz testing failed!"
    exit 1
fi

echo "Fuzz testing completed successfully."
