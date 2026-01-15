#!/usr/bin/env bash
#set -euo pipefail # we need to check it


# Remove all paths containing 'rapidgzip' from PATH
export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v 'rapidgzip' | tr '\n' ':' | sed 's/:$//')
$PROCESSOR_CMD process "$CONFIG_FILE" >stdout 2>stderr


# make sure it's not return code 0
if [ $? -eq 0 ]; then
    echo "Expected non-zero exit code, but got zero"
    echo "ran $PROCESSOR_CMD process $CONFIG_FILE"
    echo "stdout from run was"
    cat stdout
    echo "stderr from run was"
    cat stderr

    cat $CONFIG_FILE
    ls 
    cat output_read1.fq
    exit 1
fi

# make sure expected_panic.txt contents are in stderr
EXPECTED_STRING="Make sure you have a rapidgzip binary on your path."

stderr=$(cat stderr)
if ! grep -q "$EXPECTED_STRING" stderr; then
    echo "Expected panic message not found in stderr"
    echo "stderr: $stderr"
    exit 1
fi
