### FilterOtherFile


```toml
[[step]]
    action = "FilterOtherFile"
    filename = "other_file.fq" # fastq, sam, or bam. 
                               # fastq can be compressed.
    keep_or_remove = "Remove" # or Keep
    false_positive_rate = 0.0..1.0
    seed = 42
    readname_end_chars = " /" # Optional String. Example " /" .
      # Clip the name of the FastQ read at the first occurring
      # of these characters.
      # Useful when you want to filter aligned reads,
      # but their names have for example been clipped by STAR.
```

Filter to or remove reads contained in another file.

Read the other files read names, and then either keep only reads that were present,
or remove all reads that were present.

If false_positive_rate is > 0, the filter will be a probabilistic Cuckoo filter.

If false_positive_rate == 0, we use an exact HashSet (this might use a lot of memory,
depending on your file size).


