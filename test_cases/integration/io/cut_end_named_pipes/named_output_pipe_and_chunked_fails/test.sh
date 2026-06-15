#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"
: "${CONFIG_FILE:?CONFIG_FILE must be set by the test harness}"

cleanup() {
    rm -f output_read1.fq
}
trap cleanup EXIT

mkfifo output_read1.fq

stderr_output=$("$PROCESSOR_CMD" process "$CONFIG_FILE" "$(pwd)" 2>&1 >/dev/null) && {
    echo "ERROR: expected processor to fail with named-pipe + chunked error, but it succeeded" >&2
    exit 1
}

expected="Chunked output is not supported when writing to named pipes"
if ! echo "$stderr_output" | grep -qF "$expected"; then
    echo "ERROR: stderr did not contain expected error text" >&2
    echo "Expected to find: $expected" >&2
    echo "Actual stderr: $stderr_output" >&2
    exit 1
fi
