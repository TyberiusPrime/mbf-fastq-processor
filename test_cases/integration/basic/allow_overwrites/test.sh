#!/usr/bin/env bash
set -euo pipefail 2>/dev/null || set -eu

: "${PROCESSOR_CMD:?PROCESSOR_CMD must be set by the test harness}"

if [ ! -e "input_read1.fq" ]; then
    echo "input_read1.fq is missing" >&2
    exit 1
fi

INCOMPLETE_FILENAME="output.incompleted"
OUTPUT_FQ="output_read1.fq"

# First, run without --allow-overwrites and verify it succeeds
echo "Testing normal operation..."
if ! "$PROCESSOR_CMD" process input.toml; then
    echo "Initial processing failed" >&2
    exit 1
fi

# Verify the output file was created
if [ ! -e $OUTPUT_FQ ]; then
    echo "Expected output file $OUTPUT_FQ was not created" >&2
    exit 1
fi

# Verify no .incompleted file exists
if [ -e $INCOMPLETE_FILENAME ]; then
    echo "Unexpected $INCOMPLETE_FILENAME file found after successful run" >&2
    exit 1
fi
#
# Store the expected correct content from the first run
expected_content=$(cat $OUTPUT_FQ)

echo "sentinel" > $OUTPUT_FQ

echo "Testing overwrite protection (should fail)..."
# Try to run again without --allow-overwrites - this should fail
if "$PROCESSOR_CMD" process input.toml 2>/dev/null; then
    echo "Second run should have failed due to existing output file" >&2
    exit 1
fi

# Verify the original output file still exists and wasn't corrupted
if [ ! -e $OUTPUT_FQ ]; then
    echo "Original output file was removed during failed overwrite attempt" >&2
    exit 1
fi

actual_content=$(cat $OUTPUT_FQ)
if [ "$actual_content" != "sentinel" ]; then
    echo "Output file content was altered during failed overwrite attempt" >&2
    exit 1
fi

# Verify no .incompleted file was created during the failed attempt
if [ -e $INCOMPLETE_FILENAME ]; then
    echo "Unexpected $INCOMPLETE_FILENAME file found after failed overwrite attempt" >&2
    exit 1
fi


echo "Modifying output file to verify --allow-overwrite actually overwrites..."
# Modify the output file to something different
echo "CORRUPTED FILE CONTENT" > $OUTPUT_FQ

# Verify the file was actually changed
modified_content=$(cat $OUTPUT_FQ)
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
if [ ! -e $OUTPUT_FQ ]; then
    echo "Output file missing after successful overwrite" >&2
    exit 1
fi

# Verify no .incompleted file exists
if [ -e $INCOMPLETE_FILENAME ]; then
    echo "Unexpected $INCOMPLETE_FILENAME file found after successful overwrite" >&2
    exit 1
fi

# Get the content after overwrite
restored_content=$(cat $OUTPUT_FQ)

# Verify the content was restored to the original correct content
if [ "$expected_content" != "$restored_content" ]; then
    echo "Content was not properly restored after --allow-overwrite" >&2
    echo "Expected length: ${#expected_content}, Restored length: ${#restored_content}" >&2
    exit 1
fi

echo "Verified file was actually overwritten: corrupted content replaced with correct content"

# now make it fail, but not with 'already exist'
rm output*


orig_input_toml=$(cat input.toml)

echo <<'EOF'
[[step]]
  action = "_InduceFailure"
  msg = "Testing..."
EOF; >>input.toml

if "$PROCESSOR_CMD" process input.toml 2>/dev/null; then
    echo "This run should have failed after output file creation" >&2
    exit 1
fi
#
# verify the output files do exist (albeit empty)
if -! [ -e $OUTPUT_FQ ]; then
    echo "Expected output file $OUTPUT_FQ was not created before failure" >&2
    exit 1
fi

if [ ! -e $INCOMPLETE_FILENAME ]; then
    echo "Expected $INCOMPLETE_FILENAME file was not created after failure" >&2
    exit 1
fi

# restore original input.toml
echo "$orig_input_toml" > input.toml
#run now succeeds, even without --allow-overwrites because .incompleted
if ! "$PROCESSOR_CMD" process input.toml then
    echo "Run with --allow-overwrite failed after induced failure" >&2
    exit 1
fi
# and the output file has the original conten
final_content=$(cat $OUTPUT_FQ)
if [ "$expected_content" != "$final_content" ]; then
    echo "Content was not properly restored after handling induced failure" >&2
    exit 1
fi






echo "All tests passed successfully!"
