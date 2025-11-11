# ExtractPolyTail


```toml
[[step]]
    action = "ExtractPolyTail"
    out_label = "tag_label"
    segment = "read1" # Any of your input segments (default: read1)
    min_length = 5 # positive integer, the minimum number of repeats of the base
    base = "A" # one of AGTCN., the 'base' to trim (or . for any repeated base)
    max_mismatch_rate = 0.1 # float 0.0..=1.0, how many mismatches are allowed in the repeat
    max_consecutive_mismatches = 3 # how many consecutive mismatches are allowed

```

Identify either a specific letter (AGTC or N) repetition, 
or any base repetition (base = '.') at the end of the read.

Trimming on this may produce empty reads, See the warning about [empty reads](#empty-reads).

Together, this is similar to fastp's `trim_poly_g`/`trim_poly_x` but with a different implementation.
