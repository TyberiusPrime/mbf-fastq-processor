---
weight: 10
---
## Demultiplexed output

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

[Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}}) is a 'magic' transformation that 'forks' the output.

Transformations downstream are duplicated per barcode,
so you can for example filter to the head reads in each barcode,
and get reports for both, all reads and each separate barcode.

Note that this does not 
extract the barcodes form the read 
(use an extract step, such as [ExtractRegion]({{< relref "docs/reference/tag-steps/extract/ExtractRegion.md" >}})).

Nor does it append the barcodes to the read name,
(use [StoreTagInComment]({{< relref "docs/reference/tag-steps/using/StoreTagInComment.md" >}}) for that) or remove the sequence from the reads
(combine with [CutStart]({{< relref "docs/reference/modification-steps/CutStart.md" >}})
/ [CutEnd]({{< relref "docs/reference/modification-steps/CutEnd.md" >}}) or perhaps 
[TrimAtTag]({{< relref "docs/reference/modification-steps/TrimAtTag.md" >}}).


Notes: 
- Query barcodes may use IUPAC codes. 
- IUPAC barcodes must be non-overlapping .
- You can define multiple barcodes to go into the same output file.
- Can be used only once in a configuration.

## Hamming Distance matching
Correcting a tag for hamming distance is a separate step. See [HammingCorrect]({{< relref "docs/reference/tag-steps/using/HammingCorrect.md" >}}).
