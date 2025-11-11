---
weight: 50
---

# TagOtherFileBySequence

Marks reads based on wether sequences are present in another file.

```toml
[[step]]
    action = "TagOtherFileBySequence"
    out_label = "present_in_other_file"
    filename = "names.fastq" # Can read fastq (also compressed), or SAM/BAM, or fasta files
    segment = "read1" # Any of your input segments
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
    ignore_unaligned = false # in case of BAM/SAM, whether to ignore unaligned reads. Mapped reads are always considered
```

This step annotates reads by comparing their sequences against sequences from another file.

Please note our [remarks about cuckoo filters]({{< relref "docs/faq/_index.md" >}}#cuckoo-filtering).
