# MaxLen

```toml
[[step]]
    action = "Truncate"
    n = 100 # the maximum length of the read. Cut at end if longer
    segment = "read1" # Any of your input segments (default: read1)
```

Cut the read down to `n' bases.