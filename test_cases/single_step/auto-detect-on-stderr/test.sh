SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
if ! "$PROCESSOR_CMD" process >stdout; then
    echo "ERROR: Processor failed" >&2
    exit 1
fi


# Compare with expected output
expected_file="$SCRIPT_DIR/stdout"
if [[ ! -f "$expected_file" ]]; then
    echo "ERROR: Expected output file $expected_file not found" >&2
    exit 1
fi

# Detailed comparison
if ! diff -u "$expected_file" "stdout"; then
    echo "ERROR: Output does not match expected results" >&2
    echo "Expected file: $expected_file"
    echo "Actual file: stdout"
    echo "Expected content:"
    head -10 "$expected_file"
    echo "Actual content:"
    head -10 "stdout"
    exit 1
fi
