# AssignByHalves


Assign sequences to references using a cell ranger inspired 'halves' algorithm to 
correct mismatching sequences.

```toml
[[step]]
    action = "AssignByHalves"
    in_label = "mytag" # a sequence region
    out_label = "assigned_tag"
    barcodes = "mybarcodelist" # the [barcodes] section to use as references

[barcodes.mybarcodelist]
    "AAAA" = "label_ignored" # only read when demultiplexing 
```


Similar [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}}) 
in `output = "barcode"` mode, this assigns sequences (e.g. gene probes) to labels
from a barcode table.

Perfectly matching sequences are assigned trivially.

For the others a cell ranger probe set assignment inspired algorithm is used.

The halves of the barcode are lookup up separately with a maximum hamming
distance of 1, yielding candidate matches.

If there is exactly one match, the other half is compared to the candidates,
calculating a mismatch score `matches - mismatches` (= `len - 2 * mismatches`)
for just the half, and the complete sequence. If the half-side score is above
0, and the total score is >= 30, the assignment is considered.

Only if the other half doesn't disagree at hamming distance of 1, 
an assignment is made.

If no assignment was made, the process is repeated with the halves reversed.



