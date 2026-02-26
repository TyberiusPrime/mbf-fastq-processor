---
weight: 110
---

# Benchmark


For profiling and benchmarking (individual) steps, 
mbf-fastq-processor has a special benchmark mode.

This mode focuses on benchmarking the steps,
and avoids (most) input and output runtime.

Enable it by adding this TOML section. 
The output section becomes optional (and ignored)
when benchmarking is enabled.

```toml
[benchmark]
    enable = true # required to enable benchmark mode
    quiet = false # default. If true, don't output timing information
    molecule_count = 1_000_000
```

Benchmark mode:

- Disables (regular) output
- runs in a temp directory,
- repeats the first molecule 'block' of [`Options.block_size`]({{< relref "docs/reference/Options.md" >}}) reads
  until `molecule_count` has been exceeded.


The last point means that we will spent very little time in 
reading & decompression (without rapidgzip / parallel BAM processing the largest
runtime parts), and focus on the steps. The drawback here is that your pipeline
sees the same reads over and over, which of course will lead to a different
'hit' profile for set based tests such as duplication counting, 
[TagOtherFileByName]({{< relref "docs/reference/tag-steps/tag/TagOtherFileByName.md" >}}),  
and [Demultiplex]({{< relref "docs/reference/Demultiplex.md" >}})  



# Results

## Parsing, Ryzen Ai Max+ 395, 2025 NVME

Benchmarking the parsing performance with a no-output, report read count only configuration,
using one file of about 44 million 75bp reads, we observe:

- FASTQ, gzip, rapidgzip, 5 threads:  11.2 million reads / second
- FASTQ, gzip, single thread / flate2 : 4.7 million reads / second

- FASTQ, uncompressed: about 6.4 million reads / second

- BAM, single-threaded:  4.7 million reads / second
- BAM, multi-threaded, 5 threads:  4.3 million reads / second (slower than single-thread. We have a bottleneck in read-to-fastq-adaptation)

- FASTA, uncompressed: 3.4 million reads / second
- FASTA, gzip, single thread, flate2 : 2.8 million reads / second
- FASTA, gzip, rapidgzip: 4 million reads / second

## Per step micro benchmarks

This section is mostly useful to estiamte which steps are fast and which are not.

On Ryzen AI Max+ 395 using benchmark mode, 12 threads, 10 million (10k repeated) single end reads.

| Time (ms) | Step |
|----------:|------|
| 2592.60 | ExtractIUPACWithIndel |
| 1944.30 | Report_count_oligios |
| 1918.40 | Rename |
| 1801.40 | FilterReservoirSample |
| 1692.80 | ConcatTags |
| 1186.90 | Report_duplicate_count_per_fragment |
| 908.83 | HammingCorrect |
| 905.40 | Demultiplex |
| 859.17 | StoreTagInSequence |
| 743.61 | TrimAtTag |
| 725.05 | QuantifyTag |
| 704.68 | StoreTagLocationInComment |
| 702.65 | MergeReads |
| 691.08 | StoreTagInComment |
| 684.80 | UppercaseTag |
| 681.20 | Report_duplicate_count_per_read |
| 659.23 | ExtractRegex |
| 620.23 | TagDuplicates |
| 619.42 | ConvertRegionsToLength |
| 531.71 | FilterByTag |
| 526.09 | LowercaseTag |
| 514.62 | ExtractRegion |
| 502.24 | ReplaceTagWithLetter |
| 498.61 | ExtractRegions |
| 477.21 | EvalExpression |
| 388.73 | Report_base_statistics |
| 380.06 | ReverseComplement |
| 352.31 | ExtractIUPAC |
| 304.53 | ValidateName |
| 215.64 | ExtractLongestPolyX |
| 209.48 | ExtractLowQualityEnd |
| 207.28 | ValidateSeq |
| 196.47 | ExtractLowQualityStart |
| 190.81 | Postfix |
| 172.78 | ValidateReadPairing |
| 172.69 | Swap |
| 166.42 | Prefix |
| 158.80 | ConvertQuality |
| 158.20 | ExtractPolyTail |
| 158.02 | Report_length_distribution |
| 153.09 | CalcComplexity |
| 153.07 | TagOtherFileBySequence |
| 150.06 | CalcGCContent |
| 149.37 | CalcLength |
| 147.20 | TagOtherFileByName |
| 140.28 | ExtractRegionsOfLowQuality |
| 139.93 | CalcQualifiedBases |
| 138.66 | CalcBaseContent |
| 137.49 | CalcExpectedError |
| 136.57 | ExtractIUPACSuffix |
| 136.45 | CalcNCount |
| 134.07 | ValidateQuality |
| 128.60 | FilterSample |
| 117.46 | LowercaseSequence |
| 114.71 | UppercaseSequence |
| 110.16 | CalcKmers |
| 108.80 | Skip |
| 106.57 | FilterEmpty |
| 104.83 | Report_count |
| 102.38 | Progress |
| 101.75 | CutStart |
| 98.83 | Truncate |
| 96.69 | CutEnd |
| 91.84 | FilterByNumericTag |
| 26.11 | Head |