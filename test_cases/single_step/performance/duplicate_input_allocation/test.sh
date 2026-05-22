#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

if [ ! -e "input_data.fq.zst" ]; then
    echo "input_data.fq.zst is missing" >&2
    exit 1
fi

run_case() {
    local config_file=$1
    local output
    # RUST_MEASURE_ALLOC=1 causes fastqrab to emit peak_rss_kb=<n> on stderr
    # after processing; capture stderr, discard stdout
    if ! output=$( RUST_MEASURE_ALLOC=1 "$PROCESSOR_CMD" process "$config_file" 2>&1 1>/dev/null ); then
        printf 'processing failed for %s\n%s\n' "$config_file" "$output" >&2
        return 1
    fi
    printf '%s\n' "$output"
}

peak_rss_kb() {
    printf '%s' "$1" | sed -n 's/.*peak_rss_kb=\([0-9][0-9]*\).*/\1/p'
}

single_output=$(run_case "config.toml")
duplicate_output=$(run_case "input_duplicate.toml")

single_rss=$(peak_rss_kb "$single_output")
duplicate_rss=$(peak_rss_kb "$duplicate_output")

if [ -z "$single_rss" ] || [ -z "$duplicate_rss" ]; then
    printf 'failed to parse peak_rss_kb from fastqrab output\nsingle:\n%s\nduplicate:\n%s\n' \
        "$single_output" "$duplicate_output" >&2
    exit 1
fi

abs_diff=$(( duplicate_rss - single_rss ))
if [ "$abs_diff" -lt 0 ]; then
    abs_diff=$(( -abs_diff ))
fi

# 10 MB slack for RSS page-granularity noise; 24x duplicate file references
# should not cause proportional memory growth since duplicates are deduplicated
# before processing.
allowed_diff_kb=10240

if [ "$abs_diff" -gt "$allowed_diff_kb" ]; then
    printf 'Duplicate input used too much memory: single=%skB duplicate=%skB diff=%skB limit=%skB\n' \
        "$single_rss" "$duplicate_rss" "$abs_diff" "$allowed_diff_kb" >&2
    exit 1
fi
