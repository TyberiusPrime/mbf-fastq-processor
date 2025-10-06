# ExtractLowComplexity


```toml
[[step]]
    action = "ExtractLowComplexity"
    label = "complexity"
    segment = "read1" # Any of your input segments, or 'All'
```


Calculate read complexity, based on the percentage of bases that are changed from their predecessor.

A good filter value might be 0.30, which means 30% complexity is required. See
[FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}).


## Corresponding options in other software 
- fastp: -low_complexity_filter
