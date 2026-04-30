#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

BAM="output_read1.bam"
BAI="output_read1.bam.bai"

# ── Baseline: clean run produces both merged .bam and .bam.bai ───────────────

echo "Testing baseline run..."
if ! "$PROCESSOR_CMD" process config.toml; then
    echo "Baseline run failed" >&2
    exit 1
fi

if [ ! -e "$BAM" ]; then
    echo "Expected $BAM was not created" >&2
    exit 1
fi
if [ ! -e "$BAI" ]; then
    echo "Expected $BAI was not created" >&2
    exit 1
fi

cp "$BAM" expected.bam
cp "$BAI" expected.bai

# ── Part A: pre-existing .bam blocks the run ─────────────────────────────────

echo "Testing overwrite protection on .bam..."
echo "sentinel" > "$BAM"

if "$PROCESSOR_CMD" process config.toml 2>/tmp/stderr_a.txt; then
    echo "Run should have failed due to existing $BAM" >&2
    exit 1
fi

if ! grep -q "output_read1.bam\" already exists" /tmp/stderr_a.txt; then
    echo "Expected error message about $BAM not found in stderr:" >&2
    cat /tmp/stderr_a.txt >&2
    exit 1
fi

actual=$(cat "$BAM")
if [ "$actual" != "sentinel" ]; then
    echo "$BAM was modified despite failure" >&2
    exit 1
fi

echo "Testing --allow-overwrite restores $BAM..."
if ! "$PROCESSOR_CMD" process config.toml . --allow-overwrite; then
    echo "Run with --allow-overwrite failed" >&2
    exit 1
fi

if ! cmp -s expected.bam "$BAM"; then
    echo "$BAM content differs from expected after overwrite" >&2
    exit 1
fi
if ! cmp -s expected.bai "$BAI"; then
    echo "$BAI content differs from expected after overwrite" >&2
    exit 1
fi

# ── Part B: pre-existing .bam.bai blocks the run ─────────────────────────────

echo "Testing overwrite protection on .bam.bai..."
rm "$BAM"
echo "sentinel" > "$BAI"

if "$PROCESSOR_CMD" process config.toml 2>/tmp/stderr_b.txt; then
    echo "Run should have failed due to existing $BAI" >&2
    exit 1
fi

if ! grep -q "output_read1.bam.bai\" already exists" /tmp/stderr_b.txt; then
    echo "Expected error message about $BAI not found in stderr:" >&2
    cat /tmp/stderr_b.txt >&2
    exit 1
fi

actual=$(cat "$BAI")
if [ "$actual" != "sentinel" ]; then
    echo "$BAI was modified despite failure" >&2
    exit 1
fi

echo "Testing --allow-overwrite restores $BAI..."
if ! "$PROCESSOR_CMD" process config.toml . --allow-overwrite; then
    echo "Run with --allow-overwrite failed" >&2
    exit 1
fi

if ! cmp -s expected.bam "$BAM"; then
    echo "$BAM content differs from expected after overwrite" >&2
    exit 1
fi
if ! cmp -s expected.bai "$BAI"; then
    echo "$BAI content differs from expected after overwrite" >&2
    exit 1
fi

echo "All tests passed!"
