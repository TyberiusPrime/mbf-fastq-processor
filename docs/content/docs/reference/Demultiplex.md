---
weight: 10
---
## Demultiplexed output

```toml
[[step]]
    action = "Demultiplex"
    regions = [ # Where are the barcodes located?
        {source = "read1", start=0, length=6},
        {source = "read1", start=10, length=6},
    ]
    max_hamming_distance = 0 # if a barcode doesn't match, how many mismatches are allowed?
    output_unmatched  = true # if set, write reads not matching any barcode
                             #  to a file like ouput_prefix_no-barcode_1.fq

[step.barcodes] # with single square brackets!
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

Query barcodes may use IUPAC codes. Matching a IUPAC code does not count as a (hamming) mismatch.

Can be used only once in a configuration.

