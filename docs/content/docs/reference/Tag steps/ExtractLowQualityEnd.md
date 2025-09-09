# TrimQualityEnd


```toml
[[step]]
    action = "ExtractLowQualityEnd"
    label = "low_quality_ends"
    min_qual = 20 # u8, minimum quality to keep (in whatever your score is encoded in)
             # either a char like 'A' or a number 0..128 (typical phred score is 33..75)
    target = "Read1" # Read1|Read2|Index1|Index2
```

Define a region of low quality bases at the end of reads.


## Corresponding options in other software 
- Trimmomatic: TRAILING (if paired with TrimAtTag)
