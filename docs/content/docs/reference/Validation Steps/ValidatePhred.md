# ValidatePhred

Validate that all scores are between 33..=41

```toml
[[step]]
    action = "ValidatePhred"
    segment = "read1" # Any of your input segments, or 'All'
    encoding = 'Illumina' # 'Illumina|Sanger|Solexa'
```

The encoding defines the accepted range of values.
No offset correction is being performed.


See https://pmc.ncbi.nlm.nih.gov/articles/PMC2847217/ , table 1
