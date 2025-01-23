# TrimPolyTail


```toml
[[step]]
    action = "TrimPolyTail"
    target = Read1|Read2|Index1|Index2 (default: read1)
    min_length = positive integer, the minimum number of repeats of the base
    base = 'A' # one of AGTCN., the 'base' to trim (or . for any repeated base)
    max_mismatche_rate = 0.1 # float 0.0..=1.0, how many mismatches are allowed in the repeat
    max_consecutive_mismatches = 3, # how many consecutive mismatches are allowed
```

Trim either a specific base repetition, or any base repetition at the end of the read.

May produce empty reads, See the warning about [empty reads](#empty-reads).


Similar to fastp's trim_poly_g/ trim_poly_x but with a different implementation.
