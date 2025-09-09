---
weight: 50
---

# TagOtherFileByName

Mark reads based on wether names are present in another file.

```toml
[[step]]
    action = "TagOtherFileByName"
    target = "read1" # which name are we using
    label = "present_in_other"
    filename = "names.fastq" # Can read fastq (also compressed), or sam/bam files
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
    ignore_unaligned = false # in case of BAM/SAM, whether to ignore unaligned reads
    readname_end_chars = " /" # (optional) chars at which to cut read names before comparing. If not set, no cutting is done.

```

This step makrs reads by comparing their names against names from another file.

With false_positive_rate > 0, uses a cuckoo filter, otherwise an exact hash set.
