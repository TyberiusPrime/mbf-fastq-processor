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

echo "Verifying output.json was created..."

# Verify that output.json was created
if [[ ! -f "output.json" ]]; then
    echo "ERROR: output.json was not created" >&2
    exit 1
fi

if [[ ! -s "output.json" ]]; then
    echo "ERROR: output.json is empty" >&2
    exit 1
fi

echo "Verifying top.json constraint..."

# Verify that top.json exists in the script directory
top_json_file="$SCRIPT_DIR/top.json"
if [[ ! -f "$top_json_file" ]]; then
    echo "ERROR: top.json not found in test directory: $top_json_file" >&2
    exit 1
fi

# Read the maximum allowed value from top.json
max_value=$(cat "$top_json_file" | tr -d '[:space:]')
if ! [[ "$max_value" =~ ^[0-9]+$ ]]; then
    echo "ERROR: top.json should contain only a number, got: '$max_value'" >&2
    exit 1
fi

echo "Maximum allowed _InternalReadCount: $max_value"

# Extract the actual _InternalReadCount from the "top" section of output.json
actual_value=$(jq -r '.top._InternalReadCount' output.json)

if [[ "$actual_value" == "null" ]]; then
    echo "ERROR: Could not find top._InternalReadCount in output.json" >&2
    echo "Available keys in output.json:"
    jq -r 'keys[]' output.json
    if jq -e '.top' output.json >/dev/null; then
        echo "Keys in .top:"
        jq -r '.top | keys[]' output.json
    fi
    exit 1
fi

if ! [[ "$actual_value" =~ ^[0-9]+$ ]]; then
    echo "ERROR: top._InternalReadCount should be a number, got: '$actual_value'" >&2
    exit 1
fi

echo "Actual _InternalReadCount: $actual_value"

# Verify that actual ≤ max
if [[ "$actual_value" -gt "$max_value" ]]; then
    echo "ERROR: _InternalReadCount $actual_value exceeds maximum $max_value" >&2
    exit 1
fi

echo "✓ _InternalReadCount constraint satisfied: $actual_value ≤ $max_value"

# Check for unexpected files (basic check, not as comprehensive as other tests)
echo "Checking for basic file expectations..."
expected_files=("input.toml" "output.json")
for file in "${expected_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "ERROR: Expected file $file not found" >&2
        exit 1
    fi
done

# top.json and test.sh are in the original test directory, not copied to temp

echo "Test completed successfully!"