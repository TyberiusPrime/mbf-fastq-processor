# ExtractLowComplexity


```toml
[[step]]
    action = "ExtractLowComplexity"
    label = "complexity"
    target = "Read1" # Read1|Read2|Index1|Index2|All
```


Calculate read complexity, based on the percentage of bases that are changed from their predecessor.

A good filter value might be 0.30, which means 30% complexity is required (See 
[FilterByNumericTag)[../Filter steps/FilterByNumericTag].


## Corresponding options in other software 
- fastp: -low_complexity_filter
