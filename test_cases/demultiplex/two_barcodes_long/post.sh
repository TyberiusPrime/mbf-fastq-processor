#!/usr/bin/env bash
python3 << EOF
import gzip

barcodes_a = {"ct": "ct", "tt": "tt"}
barcodes_b = {"ag": "ag", "at": "at"}

lines = gzip.GzipFile("input_read1.fq.gz").read().decode().split("\n")
seqs = lines[1::4]
counts = {}
for seq in seqs:
    bc_a = seq[0:2].lower()
    bc_b = seq[4:6].lower()
    name_a = barcodes_a.get(bc_a, "no-barcode")
    name_b = barcodes_b.get(bc_b, "no-barcode")
    key = f"{name_a}_{name_b}"
    if key not in counts:
        counts[key] = 0
    counts[key] += 1

error = False

for key, count in counts.items():
    fn = f"output_{key}_read1.fq"
    lines = open(fn).read().split("\n")
    ll = (len(lines) - 1) / 4
    if ll != count:
        print("mismatch", key, count, fn, ll)
        error = True

if error:
    raise ValueError("Count mismatch from expected")

EOF

