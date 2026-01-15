#!/usr/bin/env python3

data=$(<output_run.progress)

pos0=$(echo "$data" | grep -n "Processed Total: 0" | cut -d: -f1)
pos_2=$(echo "$data" | grep -n "Processed Total: 100_000" | cut -d: -f1)
pos_3=$(echo "$data" | grep -n "Processed Total: 200_000" | cut -d: -f1)
pos_4=$(echo "$data" | grep -n "Processed Total: 300_000" | cut -d: -f1)

if [ -z "$pos0" ] || [ "$pos0" -eq 0 ]; then
    echo "Error: 'Processed Total: 0' not found"
    exit 1
fi

if [ -z "$pos_2" ] || [ "$pos_2" -eq 0 ]; then
    echo "Error: 'Processed Total: 100_000' not found"
    exit 1
fi

if [ -z "$pos_3" ] || [ "$pos_3" -eq 0 ]; then
    echo "Error: 'Processed Total: 200_000' not found"
    exit 1
fi

if [ -z "$pos_4" ] || [ "$pos_4" -eq 0 ]; then
    echo "Error: 'Processed Total: 300_000' not found"
    exit 1
fi

if [ "$pos0" -ge "$pos_2" ]; then
    echo "Error: pos0 is not less than pos_2"
    exit 1
fi

if [ "$pos_2" -ge "$pos_3" ]; then
    echo "Error: pos_2 is not less than pos_3"
    exit 1
fi

if [ "$pos_3" -ge "$pos_4" ]; then
    echo "Error: pos_3 is not less than pos_4"
    exit 1
fi

if ! grep -q "337_903 molecules" output_run.progress; then
    echo "Error: '337_903 molecules' not found in file"
    exit 1
fi

rm output_run.progress # we don't check it beyond this.
echo "All checks passed successfully"


