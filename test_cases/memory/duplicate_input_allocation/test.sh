#!/usr/bin/env bash
set -euo pipefail

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

if [ ! -e "input_data.fq.zst" ]; then
    echo "input_data.fq.zst is missing" >&2
    exit 1
fi

run_case() {
    local config_file=$1
    local captured
    if ! captured=$(RUST_MEASURE_ALLOC=1 "$PROCESSOR_CMD" process "$config_file" 2>&1); then
        printf 'processing failed for %s\n%s\n' "$config_file" "$captured" >&2
        return 1
    fi
	# verify that return code was 0
	if [ $? -ne 0 ]; then
		printf 'processing failed for %s\n%s\n' "$config_file" "$captured" >&2
		return 1
	fi
    printf '%s\n' "$captured"
}

extract_metric() {
    local label=$1
    local payload=$2
    printf '%s' "$payload" | sed -n "s/.*${label}=\([0-9][0-9]*\).*/\\1/p"
}

single_output=$(run_case "input.toml")
duplicate_output=$(run_case "input_duplicate.toml")


echo $single_output
echo $duplicate_output

if [ -z "$single_output" ] || [ -z "$duplicate_output" ]; then
    echo "missing allocator output" >&2
    exit 1
fi

single_max=$(extract_metric "bytes_max" "$single_output")
duplicate_max=$(extract_metric "bytes_max" "$duplicate_output")
single_leak=$(extract_metric "bytes_current" "$single_output")
duplicate_leak=$(extract_metric "bytes_current" "$duplicate_output")

if [ -z "$single_max" ] || [ -z "$duplicate_max" ]; then
    echo "failed to parse max allocation metrics" >&2
    exit 1
fi

abs_diff=$(( duplicate_max - single_max ))
if [ "$abs_diff" -lt 0 ]; then
    abs_diff=$(( -abs_diff ))
fi

allowed_diff=$(( single_max / 10 ))
# if [ "$allowed_diff" -lt 1048576 ]; then
#     allowed_diff=1048576
# fi

if [ "$abs_diff" -gt "$allowed_diff" ]; then
    printf 'Duplicate input used too much memory: single=%s duplicate=%s diff=%s limit=%s\n' \
        "$single_max" "$duplicate_max" "$abs_diff" "$allowed_diff" >&2
    exit 1
fi

leak_threshold=1048576
if [ -n "$single_leak" ] && [ "$single_leak" -gt "$leak_threshold" ]; then
    printf 'Single input leaked memory: leak=%s threshold=%s\n' "$single_leak" "$leak_threshold" >&2
    exit 1
fi
if [ -n "$duplicate_leak" ] && [ "$duplicate_leak" -gt "$leak_threshold" ]; then
    printf 'Duplicate input leaked memory: leak=%s threshold=%s\n' "$duplicate_leak" "$leak_threshold" >&2
    exit 1
fi
