
# FilterMaxLen



```toml
[[step]]
    action = "FilterMaxLen"
    n = int, maximum length
    target = Read1|Read2|Index1|Index2
```


Drop the molecule if the read is above a specified length.


# Corresponding options in other software #

- fastp: --length_limit
