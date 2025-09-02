# ReverseComplement


```toml
[[step]]
    action = "ReverseComplement"
    target = "Read1" # Read1|Read2|Index1|Index2 (default: read1)
```

ReverseComplement the read sequence (and reverse the quality).

This supports IUPAC codes (U is complemented to A, so it's not strictly
reversible). Unknown letters are output verbatim.
