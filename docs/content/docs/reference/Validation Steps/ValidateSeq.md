# ValidateSeq

Validate that only allowed characters are in the sequence.

```toml
[[step]]
    action = "ValidateSeq"
    allowed = "AGTC" # String. Example 'ACGTN', the allowed characters
    target = "Read1"|"Read2"|"Index1"|"Index2"
```

