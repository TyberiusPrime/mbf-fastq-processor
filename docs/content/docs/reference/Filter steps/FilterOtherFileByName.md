---
weight: 50
---

# FilterOtherFileByName

Filter reads based on names present in another file.

```toml
[[step]]
    action = "FilterOtherFileByName"
    filename = "names.fastq" # Can read fastq (also compressed), or sam/bam files
    keep_or_remove = "Keep" # or "Remove"
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
```

This transformation filters reads by comparing their names against names from another file. With false_positive_rate > 0, uses a cuckoo filter, otherwise an exact hash set.