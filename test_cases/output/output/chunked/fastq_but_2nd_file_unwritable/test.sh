#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

if [ ! -e "input_read1.fq" ]; then
    echo "input_read1.fq is missing" >&2
    exit 1
fi

INCOMPLETE_FILENAME="output.incompleted"

echo "Testing with allow_overwrite operation..."
if "$PROCESSOR_CMD" process config.toml --allow-overwrite 2>stderr; then
    echo "Initial processing suceeded!" >&2
    exit 1
fi

cat stderr

if ! grep "Permission denied" stderr; then
  echo "Expected permission denied error message in stderr. Was $(cat stderr)" >&2
    exit 1
fi

if [ ! -e $INCOMPLETE_FILENAME ]; then
    echo "Expected incomplete output file $INCOMPLETE_FILENAME is missing" >&2
    exit 1
fi

if [ ! -e "output_read1.0.fq" ]; then
    echo "expected first output filename missing" >&2
    exit 1
fi
if [ ! -e "output_read1.1.fq" ]; then
    echo "expected first output filename missing" >&2
    exit 1
fi

if [ -e "output_read1.3.fq" ]; then
    echo "Unexpected third output filename" >&2
    exit 1
fi

