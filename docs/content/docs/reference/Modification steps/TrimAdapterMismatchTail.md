# TrimAdapterMismatchTail


```toml
[[step]]
    action = "TrimAdapterMismatchTail"
    query = "AGTCA"  # the adapter to trim. Straigth bases only, no IUPAC.
    target = "Read1"   # Read1|Read2|Index1|Index2 (default: read1)
    min_length = 5     # uint, the minimum length of match between the end of the read and
                       # the start of the adapter
    max_mismatches = 1 # How many mismatches to accept
```



Trim the end of a read if it matches the adapter.

Simple comparison with a max mismatch hamming distance.
