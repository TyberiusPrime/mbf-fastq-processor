# ValidateAllReadsSameLength

```toml
[[step]]
   action = "ValidateAllReadsSameLength"
   source = "read1" # Any segment, All, tag:<name> or 'name:segment>'
```

Validates that all reads have the same sequence/tag/read length.

Useful when you want to verify read length consistency in your pipeline.
