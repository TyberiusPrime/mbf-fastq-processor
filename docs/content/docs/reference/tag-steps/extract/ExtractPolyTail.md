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
    fastp_mode = false # optional (default: false). See below.

```

Identify either a specific letter (AGTC or N) repetition, 
or any base repetition (base = '.') at the end of the read.

Trimming on this may produce empty reads, See the warning about [empty reads](#empty-reads).

Together, this is similar to fastp's `trim_poly_g`/`trim_poly_x` but with a different implementation.

## fastp_mode

When `fastp_mode = true`, the step uses the exact same algorithm as fastp's `trimPolyG` instead of the default mismatch-rate algorithm. Requires `base = 'G'`.

The fastp algorithm applies fixed constants:
- 1 mismatch allowed per 8 bases (integer division, so positions 1–7 allow 0, 8–15 allow 1, etc.)
- Maximum 5 total mismatches regardless of read length
- The tail starts at the leftmost G in the identified tail region

Use this when you need byte-for-byte compatibility with fastp's polyG trimming output.
