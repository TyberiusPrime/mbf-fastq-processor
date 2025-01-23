---
weight: 3
---

# Input section

```toml
[input]
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd'] # (optional)
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd'] # (optional)
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd'] # (optional)
    # but if index1 is set, index2 must be present as well
    interleaved = false # (optional) read1 is actually read1/2 interleaved. Read2 must not be set.
                        # Interleaved input needs twice as much memory than non-interleaved input.
                        # (We duplicate a whole block instead of allocating each read for performance reasons)
```

You can omit all inputs but read1. 

Values may be lists or single filenames.

Compression is detected from file contents (.gz/bzip2/zstd).

Files must match, i.e. the first file in read1 must have the same number of reads (lines) as the first file in read2, etc.


