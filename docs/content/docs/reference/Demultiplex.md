---
weight: 10
---
## Demultiplexed output


[Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}) is a magic transformation that forks the output.

You receive one set of output files per barcode (combination) defined.

Transformations downstream are (virtually) duplicated,
so you can for example filter to the head reads in each barcode,
and get reports for both: all reads and each separate barcode.

Demultiplexing can be done on barcodes, or on boolean tags.

### Based on barcodes

```toml
[[step]]
    action = "Demultiplex"
    label = "mytag"
    barcodes = "mybarcodes"
    output_unmatched  = true # if set, write reads not matching any barcode
                             #  to a file like ouput_prefix_no-barcode_1.fq

[barcodes.mybarcodes] # can be before and after.
# separate multiple regions with a _
# a Mapping of barcode -> output name.
AAAAAA_CCCCCC = "sample-1" # output files are named prefix{ix_separator}barcode_prefix{ix_separator}segment.suffix
                           # with the separator defaulting to '_', e.g. output_sample-1_1.fq.gz
                           # or output_sample-1_report.fq.gz
```

### Based on boolean tags

```toml
[[step]]
    segment = "read1"
    action = "TagOtherFileByName"
    label = "a_bool_tag"
    filename = "path/to/boolean_tags.tsv"
    false_positive_rate = 0

[[step]]
    action = "Demultiplex"
    label = "a_bool_tag"
```


Note that this does not 
extract the barcodes from the read 
(use an extract step, such as [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}})).

Nor does it append the barcodes to the read name,
(use [StoreTagInComment]({{< relref "docs/reference/tag-steps/using/StoreTagInComment.md" >}}) for that) or remove the sequence from the reads
(combine with [CutStart]({{< relref "docs/reference/modification-steps/CutStart.md" >}})
/ [CutEnd]({{< relref "docs/reference/modification-steps/CutEnd.md" >}}) or perhaps 
[TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}).


Notes: 
- Query barcodes may use IUPAC codes. 
- IUPAC barcodes must be non-overlapping ( and this is enforced).
- Within one demultiplex step barcode must be of equal length.
- You can define multiple barcodes to go into the same output file.
- Multiple demultiplex steps per configuration are valid - you'll
  receive their product in terms of output files.
- A demultiplex step matching zero barcodes (across all reads) will issue an error.

## Hamming Distance matching
Correcting a tag for hamming distance is a separate step. See [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}}).
