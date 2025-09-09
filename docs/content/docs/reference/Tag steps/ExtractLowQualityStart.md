# TrimQualityStart


```toml
[[step]]
    action = "ExtractLowQualityStart"
    min_qual = 20 # u8, minimum quality to keep (in whatever your score is encoded in)
             # either a char like 'A' or a number 0..128 (typical phred score is 33..75)
    target = "Read1" # Read1|Read2|Index1|Index2
    label = "bad_starts"
```

Define a region with low quality bases (below threshold) at steart of read.

## Corresponding options in other software 

- Trimmomatic: LEADING (if combined with TrimAtTag)
