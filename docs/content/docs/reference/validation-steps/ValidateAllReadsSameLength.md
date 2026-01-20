# ValidateAllReadsSameLength

```toml
[[step]]
   action = "ValidateAllReadsSameLength"
   source = "read1" # Any segment, All, tag:<name> or 'name:segment>'
```

Validates that all reads have the same sequence/tag/name length.

Useful when you want to verify read length consistency in your pipeline.

(For names, the names without comments - 
that is up to the first input.options.read_comment_character are used).
