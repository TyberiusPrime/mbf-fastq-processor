---
weight: 1000
---

# Sequencing Adapters

Paste this `count_oligos` block into a `Report` step to identify which adapter is present in your data.
See [Cookbook 10: Adapter Identification]({{< relref "docs/how-to/cookbooks/10-adapter-identification.md" >}}) for a complete example.

`count_oligos` performs exact, full-sequence matching — no mismatches, no IUPAC wildcards.

```toml
[[step]] 
    action = 'Report'
    # ...
    count_oligos = {
        # Illumina TruSeq / standard adapters
        # https://support-docs.illumina.com/SHARE/AdapterSequences/adapter-sequences.htm
        "Illumina Nextera/AmpliSeq"    = "CTGTCTCTTATACACATCT",
        "Illumina TruSeq R1/miRNA"     = "AGATCGGAAGAGCACACGTCTGAACTCCAGTCA",
        "Illumina TruSeq R2"           = "AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT",
        "Illumina Small RNA R2"        = "GATCGTCGGACTGTAGAACTCTGAACGTGTAGA",
        "Illumina Single End Adapter 1"= "GATCGGAAGAGCTCGTATGCCGTCTTCTGCTTG",
        "Illumina Paired End Adapter 2"= "GATCGGAAGAGCGGTTCAGCAGGAATGCCGAG",

        # MGI/BGI adapters
        # http://seqanswers.com/forums/showthread.php?t=87647 (2nd post)
        "BGI Forward"                  = "AAGTCGGAGGCCAAGCGGTCTTAGGAAGACAA",
        "BGI Reverse"                  = "AAGTCGGATCGTAGCCATGTCGTTCTGTGAGCCAAGGAGTTG",

        # QIASeq
        "QIASeq miRNA"                 = "AACTGTAGGCACCATCAAT",

        # ABI SOLiD
        "ABI SOLiD3 Adapter A"         = "CTGCCCCGGGTTCCTCATTCTCTCAGCAGCATG",
        "ABI SOLiD3 Adapter B"         = "CCACTACGCCTCCGCTTTCCTCTCTATGGGCAGTCGGTGAT",

        # Clontech
        "Clontech Universal Primer Mix"= "CTAATACGACTCACTATAGGGCAAGCAGTGGTATCAACGCAGAGT",
    }
```

More adapters may be found in the [fastp source](https://github.com/OpenGene/fastp/blob/master/src/knownadapters.h)

