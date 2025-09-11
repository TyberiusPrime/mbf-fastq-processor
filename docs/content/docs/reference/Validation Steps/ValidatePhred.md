# ValidatePhred

Validate that all scores are between 33..=41

```toml
[[step]]
    action = "ValidatePhred"
    segment = "read1" # Any of your input segments, or 'All'
```