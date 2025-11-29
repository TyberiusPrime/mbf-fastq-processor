#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"
: "${CONFIG_FILE:?CONFIG_FILE must be set by the test harness}"

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

echo "Running processor..."

# Run the processor
if ! "$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)"; then
    echo "ERROR: Processor failed" >&2
    exit 1
fi

echo "Verifying .progress file output..."

# Verify that the expected .progress file exists
expected_file="$SCRIPT_DIR/output_progress.progress"
if [[ ! -f "$expected_file" ]]; then
    echo "ERROR: Expected .progress file not found: $expected_file" >&2
    exit 1
fi

# Verify that the actual .progress file was created
actual_file="output_progress.progress"
if [[ ! -f "$actual_file" ]]; then
    echo "ERROR: Actual .progress file not created: $actual_file" >&2
    exit 1
fi

if [[ ! -s "$actual_file" ]]; then
    echo "ERROR: Actual .progress file is empty: $actual_file" >&2
    exit 1
fi

echo "Processing and comparing .progress files..."

# Process both files: sort lines, normalize paths, replace numbers
process_progress_file() {
    local file=$1
    # Sort lines, normalize paths (absolute paths -> filename only), replace numbers with <number>
    cat "$file" | \
        sort | \
        sed -E 's|(/[^/[:space:]]+)+/([^/[:space:]]+\.fq)|\2|g' | \
        sed -E 's/[0-9]+/<number>/g'
}

expected_processed=$(process_progress_file "$expected_file")
actual_processed=$(process_progress_file "$actual_file")

# Compare processed content
if [[ "$expected_processed" != "$actual_processed" ]]; then
    echo "ERROR: .progress files do not match after processing" >&2
    echo "Expected (processed):" >&2
    echo "$expected_processed" >&2
    echo "" >&2
    echo "Actual (processed):" >&2
    echo "$actual_processed" >&2
    exit 1
fi

echo "✓ .progress file verification passed"

# Basic file check
expected_files=("input.toml" "output_progress.progress")
for file in "${expected_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "ERROR: Expected file $file not found" >&2
        exit 1
    fi
done

echo "Test completed successfully!"