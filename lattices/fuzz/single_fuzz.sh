#!/bin/bash

# Run the fuzzer and dump the times into an output file
cargo fuzz run fuzz_target_2 > output.txt

# Start the process of opening and reading the file
filename="output.txt"

# Check if the file exists
if [ ! -f "$filename" ]; then
    echo "File '$filename' not found."
    exit 1
fi

# Read the first line
read -r first_line < "$filename"
echo "First line: $first_line"

# Read the last line
last_line=$(tail -n 1 "$filename")
echo "Last line: $last_line"

# Extract time from first line
first_time=$(echo "$first_line" | sed -E 's/.*T([0-9:.]+)Z/\1/')
echo "First time: $first_time"

# Extract time from last line
last_time=$(echo "$last_line" | sed -E 's/.*T([0-9:.]+)Z/\1/')
echo "Last time: $last_time"

# Split the first_time into different portions
IFS=":. " read -r first_hours first_minutes first_seconds first_milliseconds <<< "$first_time"

echo "Hours: $first_hours"
echo "Minutes: $first_minutes"
echo "Seconds: $first_seconds"
echo "Milliseconds: $first_milliseconds"

# Split the last_time into different portions
IFS=":. " read -r last_hours last_minutes last_seconds last_milliseconds <<< "$last_time"

echo "Hours: $last_hours"
echo "Minutes: $last_minutes"
echo "Seconds: $last_seconds"
echo "Milliseconds: $last_milliseconds"

hours_diff=$((last_hours - first_hours))
minutes_diff=$((last_minutes - first_minutes))
seconds_diff=$((last_seconds - first_seconds))
milliseconds_diff=$((last_milliseconds - first_milliseconds))

echo "$milliseconds_diff" >> values.csv
