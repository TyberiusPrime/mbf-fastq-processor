#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

mkfifo input_read1.fq

nohup bash -c 'cat input_read1_source.fq > input_read1.fq' >/dev/null 2>&1 &
input_pid=$!
echo "$input_pid" > ignore_input_writer.pid