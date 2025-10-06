# ValidateName

Verify that all segments expose the same read name (or a shared prefix).

```toml
[[step]]
    action = "ValidateName"
    # Optional separator character; the comparison stops at the first match
    readname_end_char = "_" # Optional. Do not set for exact matching. Otherwise, a byte character
```

When no separator character is provided the
entire name must match exactly across all segments.

Otherwise the read names are truncated after the first readname_end_char,
and the prefixes must match exactly. For example, use `readname_end_char = "_"` for typical
Illumina _1/_2 suffixes.

Note that this validation requires at least two input segments so there is a
name to compare against.
