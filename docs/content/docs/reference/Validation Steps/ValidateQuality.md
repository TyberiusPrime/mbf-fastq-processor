# ValidateQuality

Validate that all scores are between 33..=41

```toml
[[step]]
    action = "ValidateQuality"
    segment = "read1" # Any of your input segments, or 'All'
    encoding = 'Illumina1.8' # 'Illumina1.8|Illumina1.3|Sanger|Solexa'
    # Illumina1.8 is an alias for Sanger.
```

The encoding defines the accepted range of values.

If you want to convert quality codes, use [ConvertQuality]({{< relref "docs/reference/Modification Steps/ConvertQuality.md" >}}).


See https://pmc.ncbi.nlm.nih.gov/articles/PMC2847217/ , table 1
