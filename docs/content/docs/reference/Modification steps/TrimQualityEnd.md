# TrimQualityEnd


```toml
[[step]]
    action = "TrimQualityEnd"
    min = 20 # u8, minimum quality to keep (in whatever your score is encoded in)
             # either a char like 'A' or a number 0..128 (typical phred score is 33..75)
    target = "Read1" # Read1|Read2|Index1|Index2
```

Cut bases off the end of a read, if below a threshold quality.



## Corresponding options in other software 
- Trimmomatic: TRAILING
