#CalcQualifiedBases


```toml
[[step]]
    action = "CalcQualifiedBases"
    threshold = 'C' # the quality value >= which a base is qualified 
                    # In your phred encoding. Typically 33..75
                    # a byte or a number 0...255
    op = 'worse' # see below.
    segment = "read1" # Any of your input segments, or 'All'
    label = "tag_name"
```

Calculate the number of bases that are 'qualified', that is 
abov/below a user defined threshold.
that is below a user defined threshold.

Use (`CalcRate`)[{{< relref "docs/reference/tag-steps/calc/CalcRate.md" >}}] to calculate the rate of qualified bases.


Note that smaller Phred values are *better*. 

To remove confusion, op may be 'Better'/'Worse' instead of 'Below'/'Above'.

Accepted values are

* worse / above / > / gt
* worse_or_equal / above_or_equal / >= / gte
* better / below / < / lt
* better_or_equal / below_or_equal / <= / lte



## Corresponding options in other software #
 - fastp : --qualified_quality_phred / --unqualified_percent_limit (if combined with [FilterByNumericTag]({{< relref "docs/reference/filter-steps/FilterByNumericTag.md" >}}))
