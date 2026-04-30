#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"


touch output_tags.tsv
chmod -w output_tags.tsv

stderr_output=$("$PROCESSOR_CMD" process config.toml --allow-overwrite 2>&1 >/dev/null) && {
    echo "ERROR: expected command to fail, but it succeeded" >&2
    exit 1
}

expected='/output_tags.tsv": Permission denied (os error 13)'
if ! echo "$stderr_output" | grep -qF "$expected"; then
    echo "ERROR: stderr did not contain expected error text" >&2
    echo "Expected to find: $expected" >&2
    echo "Actual stderr: $stderr_output" >&2
    exit 1
fi
