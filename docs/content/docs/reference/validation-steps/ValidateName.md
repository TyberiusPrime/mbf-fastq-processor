# ValidateName

Verify that all segments have the same read name (or a shared prefix).

```toml
[[step]]
    action = "ValidateName"
    # Optional separator character; the comparison stops at the first match
    readname_end_char = "_" # Optional. Do not set for exact matching. Otherwise, a byte character
    sample_stride = 1000 # Check every nth fragment, default 1000. Must be > 0. Starts with first read
```

When no separator character  (readname_end_char) is provided the
entire name must match exactly across all segments.

When a readname_end_char is provided, it must occur in the read names.

The read names are truncated after the first readname_end_char,
and the prefixes must match exactly. 

For example, use `readname_end_char = "_"` for typical older Illumina _1/_2 suffixes.

Note that this validation requires at least two input segments so there is a
name to compare against, mbf-fastq-processor will return an error otherwise.


See also: [`ValidateReadPairing`]({{< relref "docs/reference/validation-steps/ValidateReadPairing.md" >}}) 
which confirms read names are within a hamming distance of each other.

