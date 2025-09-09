---
weight: 50
---

# TagOtherFileBySequence

Marks reads based on wether sequences are present in another file.

```toml
[[step]]
    action = "TagOtherFileBySequence"
    label = "present_in_other_file"
    filename = "sequences.fastq" # fastq (also compressed), or sam/bam files
    target = "Read1" # Read1|Read2|Index1|Index2
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
```

This ste annotates reads by comparing their sequences against sequences from another file.
