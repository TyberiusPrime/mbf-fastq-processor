#ExtractQualifiedBases


```toml
[[step]]
    action = "ExtractQualifiedBases"
    min_quality = 30 # the quality value >= which a base is qualified 
                    # In your phred encoding. Typically 33..75
                    # a byte or a number 0...255
    segment = "read1" # Any of your input segments, or 'All'
    label = "tag_name"
```

Calculate  by the percentage of bases that are 'unqualified',
that is below a user defined threshold.


## Corresponding options in other software #
 - fastp : --qualified_quality_phred / --unqualified_percent_limit (if combined with [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}))
