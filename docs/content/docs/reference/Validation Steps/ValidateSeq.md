# ValidateSeq

Validate that only allowed characters are in the sequence.

```toml
[[step]]
    action = "ValidateSeq"
    allowed = "AGTC" # String. Example 'ACGTN', the allowed characters
    segment = "read1" # Any of your input segments, or 'All'
```