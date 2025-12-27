#!/usr/bin/env bash
#set -euo pipefail # we need to check it

echo "now calling process"

PATH="" "$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)" 2>stderr

# make sure it's not return code 0
if [ $? -eq 0 ]; then
    echo "Expected non-zero exit code"
    exit 1
fi

# make sure expected_panic.txt contents are in stderr
EXPECTED_PANIC_FILE="expected_panic.txt"
EXPECTED_STRING = "$(cat "$EXPECTED_PANIC_FILE")"
if ! grep "$EXPECTED_STRING" stderr; then
    echo "Expected panic message not found in stderr"
    exit 1
fi
