#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

mkfifo output_read1.fq

# nohup bash -c 'cat input_read1_source.fq > input_read1.fq' >/dev/null 2>&1 &
# input_pid=$!
# echo "$input_pid" > ignore_input_writer.pid

nohup bash -c 'cat output_read1.fq > output_read1_after_cat.fq' >/dev/null 2>&1 &
output_pid=$!
echo "$output_pid" > ignore_output_reader.pid