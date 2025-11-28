#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"
: "${CONFIG_FILE:?CONFIG_FILE must be set by the test harness}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

# Function to wait for a background process to complete
await_pid() {
    local pid_file=$1
    if [[ -f $pid_file ]]; then
        local pid
        pid=$(cat "$pid_file")
        if [[ -n $pid ]]; then
            while kill -0 "$pid" 2>/dev/null; do
                sleep 0.1
            done
        fi
    fi
}

# Function to clean up background processes and named pipes
cleanup() {
    # Kill background processes if they're still running
    if [[ -f ignore_output_reader.pid ]]; then
        local pid
        pid=$(cat ignore_output_reader.pid 2>/dev/null || echo "")
        if [[ -n $pid ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
        rm -f ignore_output_reader.pid
    fi
    
    # Remove named pipes if they exist
    rm -f output_read1.fq
}

# Set up cleanup trap
trap cleanup EXIT

echo "Setting up output named pipe..."

# Create output named pipe (input is a regular file)
mkfifo output_read1.fq

# Verify that input file exists
if [[ ! -f input_read1.fq ]]; then
    echo "ERROR: input_read1.fq not found" >&2
    exit 1
fi

echo "Starting background process to capture output pipe..."

# Start background process to read data from the output pipe
nohup bash -c 'cat output_read1.fq > output_read1_after_cat.fq' >/dev/null 2>&1 &
output_pid=$!
echo "$output_pid" > ignore_output_reader.pid

echo "Verifying output named pipe is created..."

# Verify that named pipe was created successfully
if [[ ! -p "output_read1.fq" ]]; then
    echo "ERROR: output_read1.fq is not a named pipe" >&2
    exit 1
fi

echo "Running processor with output named pipe..."

# Run the processor - this should read from a regular file and write to the output pipe
if ! "$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)"; then
    echo "ERROR: Processor failed" >&2
    exit 1
fi

echo "Waiting for output background process to complete..."

# Wait for the background process to complete
await_pid ignore_output_reader.pid

echo "Verifying output was captured..."

# Verify that output was captured from the pipe
if [[ ! -f "output_read1_after_cat.fq" ]]; then
    echo "ERROR: output_read1_after_cat.fq was not created" >&2
    exit 1
fi

if [[ ! -s "output_read1_after_cat.fq" ]]; then
    echo "ERROR: output_read1_after_cat.fq is empty" >&2
    exit 1
fi

echo "Comparing output with expected results..."

# Compare with expected output
expected_file="$SCRIPT_DIR/output_read1_after_cat.fq"
if [[ ! -f "$expected_file" ]]; then
    echo "ERROR: Expected output file $expected_file not found" >&2
    exit 1
fi

# Detailed comparison
if ! diff -u "$expected_file" "output_read1_after_cat.fq"; then
    echo "ERROR: Output does not match expected results" >&2
    echo "Expected file: $expected_file"
    echo "Actual file: output_read1_after_cat.fq"
    echo "Expected content (first 10 lines):"
    head -10 "$expected_file"
    echo "Actual content (first 10 lines):"
    head -10 "output_read1_after_cat.fq"
    exit 1
fi

# Check for unexpected files (excluding our known files and temporary files)
echo "Checking for unexpected files..."
unexpected_files=()
for file in *; do
    case "$file" in
        input.toml|input_read1.fq|output_read1_after_cat.fq|skip_windows|test.sh|prep.sh|post.sh|actual)
            # Expected files - skip
            ;;
        ignore_*.pid|output_read1.fq)
            # Temporary files we create - skip
            ;;
        *)
            if [[ -f "$file" ]]; then
                unexpected_files+=("$file")
            fi
            ;;
    esac
done

if [[ ${#unexpected_files[@]} -gt 0 ]]; then
    echo "ERROR: Unexpected files found: ${unexpected_files[*]}" >&2
    exit 1
fi

# Check that no expected files are missing
echo "Checking for missing expected files..."
missing_files=()

# Check for expected output files
if [[ ! -f "output_read1_after_cat.fq" ]]; then
    missing_files+=("output_read1_after_cat.fq")
fi

# The processor should not create output_read1.fq as a regular file since it's a named pipe
# but let's verify the named pipe exists during processing (it should be cleaned up after)

if [[ ${#missing_files[@]} -gt 0 ]]; then
    echo "ERROR: Missing expected files: ${missing_files[*]}" >&2
    exit 1
fi

echo "All checks passed! Output named pipe test completed successfully."

# Clean up temporary files created during the test
rm -f ignore_output_reader.pid
rm -f output_read1.fq

echo "Test completed successfully!"