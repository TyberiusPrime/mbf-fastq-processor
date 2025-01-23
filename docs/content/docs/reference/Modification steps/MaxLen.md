# MaxLen

```toml
[[step]]
    action= "MaxLen"
    n = int # the maximum length of the read. Cut at end if longer
    target = Read1|Read2|Index1|Index2 (default: read1)
```

Cut the read down to `n' bases.


