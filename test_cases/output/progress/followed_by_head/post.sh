#!/usr/bin/env python3

data=$(<output_run.progress)

pos0=$(echo "$data" | grep -n "Processed Total: 0" | cut -d: -f1)

if ! grep -q "10_000 molecules" output_run.progress; then
    echo "Error: '10_000 molecules' not found in file"
    exit 1
fi

echo "All checks passed successfully"

