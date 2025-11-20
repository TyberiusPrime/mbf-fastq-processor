---
weight: 50
---

# TagOtherFileByName

Mark reads based on wether names are present in another file.

```toml
[[step]]
    action = "TagOtherFileByName"
    segment = "read1" # which segment's name are we using
    out_label = "present_in_other"
    filename = "names.fastq" # Can read fastq (also compressed), or SAM/BAM, or fasta files
    false_positive_rate = 0.01 # false positive rate (0..1)
    seed = 42 # seed for randomness
    include_mapped = true # in case of BAM/SAM, whether to include aligned reads
    include_unmapped = true # in case of BAM/SAM, whether to include unaligned reads
    fastq_readname_end_char = " " # (optional) char (byte value) at which to cut input fastq read names before comparing. If not set, no cutting is done.
    reference_readname_end_char = "/" # (optional) char (byte value) at which to cut reference read names before storing them.

```

This step marks reads by comparing their names against names from another file.

`fastq_readname_end_chars` trims the incoming fastq read names before lookup, while `reference_readname_end_chars` controls how names from the reference file are stored. Leave either field unset to keep names unchanged.

With false_positive_rate > 0, uses a cuckoo filter, otherwise an exact hash set.

Please note our [remarks about cuckoo filters]({{< relref "docs/faq/_index.md" >}}#cuckoo-filtering).
