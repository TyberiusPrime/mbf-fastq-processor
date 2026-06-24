#!/usr/bin/env python3

data=$(<output_run.progress)

# First progress line: reports the running total of the first block (a value
# below the first 100_000 boundary), so we just require *some* initial line
# before the 100_000 report rather than a synthetic "0".
pos0=$(echo "$data" | grep -n "Processed Total:" | head -1 | cut -d: -f1)
pos_2=$(echo "$data" | grep -P -n "Processed Total: +1[0-9]{2}_[0-9]{3}" | cut -d: -f1)
pos_3=$(echo "$data" | grep -P -n "Processed Total: +2[0-9]{2}_[0-9]{3}" | cut -d: -f1)
pos_4=$(echo "$data" | grep -P -n "Processed Total: +3[0-9]{2}_[0-9]{3}" | cut -d: -f1)

if [ -z "$pos0" ] || [ "$pos0" -eq 0 ]; then
    echo "Error: no initial 'Processed Total:' line found"
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


