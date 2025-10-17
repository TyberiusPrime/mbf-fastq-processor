#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

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

# verify that they're still named pipes
if [[ ! -p "input_read1.fq" ]]; then
	echo "ignore_input_writer.pid is not a named pipe" >&2
	exit 1
fi


await_pid ignore_input_writer.pid

# if ! diff -u "${SCRIPT_DIR}/expected_output_read1.fq" ignore_output_read1.fq; then
#     echo "Output captured from named pipe did not match expected output" >&2
#     exit 1
# fi

rm -f ignore_input_writer.pid 
