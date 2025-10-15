status: open
# Duplication quantification


## For the reports / statistics:
    - **Feature**: Duplication distribution reporting (frequency of duplicates)
    - **Reference**: Compare with fastp's approach (samples ~1 in 20 reads up to 10k)

    Sampling the first 10k reads is problematic (for many files, 
    these are not representative).

    We already know whether it was present at least once if it's the reports dedup
    filter. Could add a subsample of reads to a hashmap to estimate frequency .


## For individual reads: 

Tally has a mode where it adds the replication count to each read.
That's going to be difficult without a second pass.

Maybe we can have some memory mapped file in FIFO mode to store the reads that passed the filters,
then do a second pass through those?


