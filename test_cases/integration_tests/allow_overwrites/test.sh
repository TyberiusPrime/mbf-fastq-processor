#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

if [ ! -e "input_read1.fq" ]; then
    echo "input_read1.fq is missing" >&2
    exit 1
fi

# First, run without --allow-overwrites and verify it succeeds
echo "Testing normal operation..."
if ! "$PROCESSOR_CMD" process input.toml; then
    echo "Initial processing failed" >&2
    exit 1
fi

# Verify the output file was created
if [ ! -e "output_read1.fq" ]; then
    echo "Expected output file 'output_read1.fq' was not created" >&2
    exit 1
fi

# Verify no .incompleted file exists
if [ -e "output_read1.fq.incompleted" ]; then
    echo "Unexpected .incompleted file found after successful run" >&2
    exit 1
fi

echo "Testing overwrite protection (should fail)..."
# Try to run again without --allow-overwrites - this should fail
if "$PROCESSOR_CMD" process input.toml 2>/dev/null; then
    echo "Second run should have failed due to existing output file" >&2
    exit 1
fi

# Verify the original output file still exists and wasn't corrupted
if [ ! -e "output_read1.fq" ]; then
    echo "Original output file was removed during failed overwrite attempt" >&2
    exit 1
fi

# Verify no .incompleted file was created during the failed attempt
if [ -e "output_read1.fq.incompleted" ]; then
    echo "Unexpected .incompleted file found after failed overwrite attempt" >&2
    exit 1
fi

# Store the expected correct content from the first run
expected_content=$(cat "output_read1.fq")

echo "Modifying output file to verify --allow-overwrite actually overwrites..."
# Modify the output file to something different
echo "CORRUPTED FILE CONTENT" > "output_read1.fq"

# Verify the file was actually changed
modified_content=$(cat "output_read1.fq")
if [ "$expected_content" = "$modified_content" ]; then
    echo "Failed to modify output file for overwrite test" >&2
    exit 1
fi

echo "Testing --allow-overwrite (should restore correct content)..."
# Now try with --allow-overwrite - this should succeed and restore correct content
if ! "$PROCESSOR_CMD" process input.toml . --allow-overwrite; then
    echo "Third run with --allow-overwrite failed" >&2
    exit 1
fi

# Verify the output file still exists
if [ ! -e "output_read1.fq" ]; then
    echo "Output file missing after successful overwrite" >&2
    exit 1
fi

# Verify no .incompleted file exists
if [ -e "output_read1.fq.incompleted" ]; then
    echo "Unexpected .incompleted file found after successful overwrite" >&2
    exit 1
fi

# Get the content after overwrite
restored_content=$(cat "output_read1.fq")

# Verify the content was restored to the original correct content
if [ "$expected_content" != "$restored_content" ]; then
    echo "Content was not properly restored after --allow-overwrite" >&2
    echo "Expected length: ${#expected_content}, Restored length: ${#restored_content}" >&2
    exit 1
fi

# Verify it's different from the corrupted content we inserted
if [ "$modified_content" = "$restored_content" ]; then
    echo "File was not actually overwritten - still contains corrupted content" >&2
    exit 1
fi

echo "Verified file was actually overwritten: corrupted content replaced with correct content"

# Clean up output files since we handle verification in this script
rm -f output_read1.fq output_read1.fq.incompleted

echo "All tests passed successfully!"