### FilterMeanQuality

```toml
[[step]]
    action = "FilterMeanQuality"
    min = # float, minimum average quality to keep 
          # (in whatever your score is encoded in.
          # Typical Range is 33..75)
    target = Read1|Read2|Index1|Index2
```


Drop the molecule if the average quality is below the specified level.

This is typically a bad idea, see https://www.drive5.com/usearch/manual/avgq.html for a discussion of the issues.


## Corresponding options in other software 

- Trimmomatic: AVGQUAL:
- fastp: --average_qual


