#FilterQualifiedBases


```toml
[[step]]
    action = "FilterQualifiedBases"
    min_quality: 'c' # the quality value >= which a base is qualified. 
                    # In your phred encoding. Typically 33..75
                    # a byte or a number 0...255
    min_ratio= 0.5 # minimum ratio (0..1) of qualified bases required
    target: Read1|Read2|Index1|Index2
```

Filter by the maximum percentage of bases that are 'unqualified',
that is below a threshold.


## Corresponding options in other software #
 - fastp : --qualified_quality_phred / --unqualified_percent_limit
