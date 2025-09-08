---
weight: 50
---

# FilterOtherFileBySequence

Filter reads based on sequences present in another file.

```toml
[[step]]
    action = "FilterOtherFileBySequence"
    filename = "sequences.fastq" # fastq (also compressed), or sam/bam files
    keep_or_remove = "Keep" # or "Remove"
    target = "Read1" # Read1|Read2|Index1|Index2
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
```

This transformation filters reads by comparing their sequences against sequences from another file.