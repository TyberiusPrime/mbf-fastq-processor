### TagDuplicatesTwoStep

```toml
[[step]]
    action = "TagDuplicatesTwoStep"
    label = "dups"

    # First region: exact matching using HashMap
    [step.first_region]
        segment = "read1"
        start = 0
        length = 8

    # Second regions: probabilistic matching using Cuckoo filter
    [[step.second_regions]]
        segment = "read1"
        start = 8
        length = 10

    false_positive_rate = 0.00001  # 0..1, applies to second step only
    seed = 59  # required when false_positive_rate > 0

[[step]]
    action = "FilterByBoolTag"
    label = "dups"
    keep_or_remove = "Remove"  # Keep|Remove
```

Two-step duplicate detection with hybrid exact/probabilistic matching:

1. **First step (exact)**: Uses a HashMap to exactly match the `first_region`. This creates separate bins for each unique first region sequence.

2. **Second step (probabilistic)**: Within each bin, uses a [Cuckoo filter](https://en.wikipedia.org/wiki/Cuckoo_filter) to detect duplicates based on `second_regions`.

This approach is particularly efficient when:
- You have a small barcode/UMI in the first region (exact matching)
- Followed by longer sequence content in the second region (probabilistic matching)

The memory requirement is automatically estimated based on 4^n where n is the total length of `second_regions`. This represents the theoretical maximum number of unique sequences for that region length.

**Parameters:**
- `first_region`: Single region definition for exact matching (becomes HashMap key)
- `second_regions`: One or more regions for probabilistic matching
- `false_positive_rate`: The false positive rate for the Cuckoo filter (0..1). Lower values require more memory.
  - Set to 0.0 to use exact HashSet matching for the second step (no false positives, but higher memory usage)
- `seed`: Required when `false_positive_rate > 0.0`. Ensures reproducible results.

**Example use case:**
If your reads have an 8bp UMI followed by variable sequence content, you can use the UMI as `first_region` for exact binning, then detect duplicates within each UMI bin using the remaining sequence.

Please note our [remarks about cuckoo filters]({{< relref "docs/faq/_index.md" >}}#cuckoo-filtering).
