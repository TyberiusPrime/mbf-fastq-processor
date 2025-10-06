---
weight: 3
not-a-transformation: true
---

# Input section

```toml
[input]
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd'] #one is requered
    read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd'] # (optional)
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd'] # (optional)
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd'] # (optional)
    # interleaved = [...] # Activates interleaved reading, see below
```

Input names define 'segments' of reads, which are referenced by name in the later steps in the pipeline.

The names and order are arbitrary, though read1/read2(/index1/index2) is custom in Illumina sequencing.

Values may be lists or single filenames.

Compression is detected from file contents (.gz/bzip2/zstd).

Files must match, i.e. all segments must have the same number of files (and reads).

## Interleaved input

At times one encounters fastq files that are not split into one file per read segment,
but contain all segments ('reads') of one molecule one after the other.

For those files, you can use interleaved mode, which supports an arbitrary
number of segments.

```toml
[input]
    source = ['interleaved.fq'] # The name does not matter here. Exactly one key .
    interleaved = ["read1","read2","index1","index2"] # list of 'segment names'. 
```
