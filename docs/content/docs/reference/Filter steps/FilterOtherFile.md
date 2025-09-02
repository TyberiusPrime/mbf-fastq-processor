### FilterOtherFile


```toml
[[step]]
    action = "FilterOtherFile"
    filename = "other_file.fq" # fastq, sam, or bam. 
                               # fastq can be compressed.
    keep_or_remove = "Remove" # or Keep
    false_positive_rate = 0.01 # 0.0..1.0
    ignore_unmapped = true # Required if filename is SAM or BAM
    seed = 42
    readname_end_chars = " /" # Optional String. Example " /" .
      # Clip the name of the FastQ read at the first occurring
      # of these characters.
      # Useful when you want to filter aligned reads,
      # but their names have for example been clipped by STAR.
```

Filter to or remove reads contained in another file.

Read the other files read names, and then either keep only reads that were present (`keep_or_remove` = "Keep" ),
or remove all reads that were present (`keep_or_remove` = "Remove".

If `false_positive_rate` is > 0, the filter will be a probabilistic Cuckoo filter.

If `false_positive_rate` is 0.0, we use an exact HashSet (this might use a lot of memory,
depending on your file size).

`ignore_unmappede`is useful when your aligner has outputted unmapped reads into your BAM
(SAM) file, because otherwise you'd filter all reads.
