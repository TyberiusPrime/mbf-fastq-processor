#FilterQualifiedBases


```toml
[[step]]
    action = "FilterQualifiedBases"
    min_quality: u8 # the quality value >= which a base is qualified. 
                    # In your phred encoding. Typically 33..75
    max_percentage: the maximum percentage of unqualified bases necessary (0..=1)
    target: Read1|Read2|Index1|Index2
```

Filter by the maximum percentage of bases that are 'unqualified',
that is below a threshold.


## Corresponding options in other software #
 - fastp : --qualified_quality_phred / --unqualified_percent_limit
