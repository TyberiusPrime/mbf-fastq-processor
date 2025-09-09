# ExtractIUPACSuffix


```toml
[[step]]
    action = "ExtractIUPACSuffix"
    label = "mytag"
    query = "AGTCA"  # the adapter to trim. Straigth bases only, no IUPAC.
    target = "Read1"   # Read1|Read2|Index1|Index2 (default: read1)
    min_length = 3     # uint, the minimum length of match between the end of the read and
                       # the start of the adapter
    max_mismatches = 1 # How many mismatches to accept
```

Find a potentially truncated IUPAC sequence at the end of a read.

Simple comparison with a max mismatch hamming distance, requiring only the first min length
bases of the query to match at the end of the read.

Trim with TrimAtTag if you want to remove the found suffix.
