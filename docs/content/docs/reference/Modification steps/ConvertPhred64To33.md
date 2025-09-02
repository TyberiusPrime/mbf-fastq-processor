# ConvertPhred64To33


```toml
[[step]]
    action = "ConvertPhred64To33"
```

Older Illumina data had a different encoding for the quality stores,

starting at 64 instead of 33.
This transformation converts the quality scores to the 33 encoding.



## Corresponding options in other software 

- trimmomatic TOPHRED33


