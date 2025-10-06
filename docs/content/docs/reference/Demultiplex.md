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
AAAAAA_CCCCCC = "sample-1" # output files will be named prefix.barcode_prefix.infix.suffix
                           # e.g. output_sample-1_1.fq.gz
                           # e.g. output_sample-1_report.fq.gz
```

Demultiplex is a 'magic' transformation that 'forks' the output.

Transformations downstream are duplicated per barcode,
so you can for example filter to the head reads in each barcode,
and get reports for both, all reads and each separate barcode.

Note that this does not append the barcodes to the name,
(use ExtractToName for that) nor does it remove the sequence from the reads
(combine with CutStart/CutEnd).

Query barcodes may use IUPAC codes. 
Matching a IUPAC code does not count as a (hamming) mismatch.

You can define multiple barcodes to go into the same output file.

Can be used only once in a configuration.

## Hamming Distance matching
Correcting a tag for hamming distance is a separate step. See [HammingCorrect]({{< relref "docs/reference/tag-steps/HammingCorrect.md" >}}).
