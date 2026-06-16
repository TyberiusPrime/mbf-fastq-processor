#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"
: "${CONFIG_FILE:?CONFIG_FILE must be set by the test harness}"

#echo "Starting output named pipe test in $SCRIPT_DIR"
#ls -la


touch 'output_read1.fq'
chmod 000 output_read1.fq
if ! $PROCESSOR_CMD process --allow-overwrite 1>stdout 2>stderr; then
    echo "process call failed as expected"
else 
    echo "process succeded?"
    exit 1
fi
echo "done"

should="Could not open file for output:"
if [[ ! -f stderr ]]; then
    echo "Expected error message in stderr, but stderr is empty"
    exit 1
fi
# check if the string should is in stderr
if ! grep -c "$should" stderr; then
    echo "Expected error message not found in stderr"
    echo 'actual' "$(cat stderr)"
    exit 1
else
    echo "Expected error message found in stderr"
fi

echo "All tests passed successfully!"
