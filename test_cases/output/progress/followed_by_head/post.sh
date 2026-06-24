#!/usr/bin/env python3

data=$(<output_run.progress)

if grep -q "337_903 molecules" output_run.progress; then
    echo "Error: '337_903 molecules' was found in file"
    cat output_run.progress
    exit 1
fi

 #verify that 10_000, 20_000 or similar low multple of 10k was found
 if ! grep -q -P "to process [12345][0-9]_[0-9]{3} molecules" output_run.progress; then
    echo "Error: Expected 'Processed Total: x0_000' not found"
    cat output_run.progress
    exit 1
fi

echo "All checks passed successfully"

