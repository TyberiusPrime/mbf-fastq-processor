---
title: UMI Extract and Trim
---
Useful for example for Lexogen Quantseq 3' mRNA-Seq Library Prep Kit FWD for Illumina.

```toml
[input]
    read1 = "myfastq.fq"

# Place  the UMI after the name, separated with '_'
[[step]]
    action = "ExtractToName"
    regions = [
        {segment= "Read1", start = 0, length = 8},
    ]

# Remove the UMI from the read.
[[step]]
    action = "CutStart"
    segment = "read1"
    n = 8

[output]
    prefix = "output"
    format = "Gzip"

```