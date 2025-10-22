# ExtractLongestPolyX

Find the longest homopolymer stretch anywhere in the read (unlike `ExtractPolyTail`, which only considers suffixes).

```toml
[[step]]
    action = "ExtractLongestPolyX"
    label = "poly_any"
    segment = "read1"
    min_length = 5
    base = '.'              # search for any homopolymer (A/C/G/T/N)
    max_mismatch_rate = 0.15
    max_consecutive_mismatches = 2

[[step]]
    action = "StoreTagLocationInComment"
    label = "poly_any"
    segment = "read1"
```

- `base` accepts a concrete nucleotide (`A`, `C`, `G`, `T`, `N`) or `.` to search all of `ACGT` and report the longest hit.
- `max_mismatch_rate` and `max_consecutive_mismatches` mirror `ExtractPolyTail`; they control how permissive the run detection is.
- When no run satisfies `min_length`, the tag is reported as missing.
- only one run is reported, even if multiple runs of the same length exist; in this case, the first run found is reported.
