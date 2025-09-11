# ReverseComplement


```toml
[[step]]
    action = "ReverseComplement"
    segment = "read1" # Any of your input segments (default: read1)
```

ReverseComplement the read sequence (and reverse the quality).

This supports IUPAC codes (U is complemented to A, so it's not strictly
reversible). Unknown letters are output verbatim.