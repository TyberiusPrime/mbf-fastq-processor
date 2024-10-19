# mbf_fastq_processor


The swiss army knife of fastq (pre-)processing.


It (will eventually) filters, samples, slices, dices, demultiplexse
s, and generally does all the things you might want to do with a et fastq files. 

It's primary concern is correctness.
And flexibility.

It's two primary concerns are correctness and flexibility, and speed.


# Installation

This is a nix flake.

There are musl-linked binaries in the github releases section that will run on any linux.

Currently not packaged by any distribution.

# Usage

`mbf_fastq_processor what_do_to.toml`

We use a [TOML](https://toml.io/en/) file, because command lines are limited and prone to misunderstandings. And you should ybe writing down what you are doing anyway.

Here's a minimal example:

```toml
[input]
    # supports multiple input files.
    # in many autodetected formats.
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd']
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd']


[[report]]
    # we can generate a report at any point in the pipeline. 
    # filename is output.prefix_infix.html/json
    infix = "pre_filter"
    json = true
    html = true

[[transform]]
    # take the first five thousand reads
	action = "Head"
	n = 5000

[[transform]]
	# extract umi and place it in the read nameo
	action = "ExtractUmi"
    # the umi is the first 8 bases of the read
    start = 0
    length = 8

[[report]]
    infix = "post_filter"
    json = true
    html = true

[output]
    #generates output_1.fq and output_2.fq
	prefix = "output"
    # uncompressed
	suffix = ".fq"

    
```


# TOML details

## Input


```
[input]
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    read2 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zstd']
    index1 = ['index1_A.fastq', 'index1_B.fastq.gz', 'index1_C.fastq.zstd']
    index2 = ['index2_A.fastq', 'index2_B.fastq.gz', 'index2_C.fastq.zstd']
```

You can ommit all inputs but read1. Values may be lists or single filenames.
Compression is detected from file contents (.gz/bzip2/zstd).
Files must match, i.e. matching files must have the same number of lines.

Todo: interleaved support


## Output
```
[output]
	prefix = "output"
        format = "Gzip"
	suffix = ".fq.gz" # you can leave this off, it's then autodetermined by the format
    compression_level = 3
    keep_index = false # write index?
```
Generates files named output_1.fq.gz, output_2.fq.gz, (optional output_i1.fq.gz, output_i2.fq.gz if keep_index is true)
Compression is independent of file ending.

Supported compression formats: Raw, Gzip, Zstd


### Inspect
Dump a few reads to a file for inspection at this point in the graph.
```
[[transform]]
    action = inspect
    n  = 1000 # how many reads
    prefix = "inspect_at_point
```

## Available transformations

Think of the transformations as defining a graph that starts with the input files,
and ends in the respective number of output files.

If the transformation splits the streams (think demultiplex), 
all subsequent transformations are applied to each stream.


### No transformation
If you specify just input and output, it's a cat equivalent +- (de)compression.

### Head
```
Arguments:
    n: int, number of reads to keep
```

### Skip
```
Arguments:
    n: int, number of reads to skip
```

### ExtractUmi
Extract a sequence from the read and place it in the read name.

```
Arguments:
    source: Read1 | Read2 | Index1 | Index2, where to extract the UMI from
    start: int, where to start extracting
    length: int, how many bases to extract
Optional:
    separator: str, what to put between the read name and the umi, defaults to '_'
    readname_end_chars: Place (with sep) at the first of these characters. 
                        Defaults to " /" (which are where STAR strips the read name).
                        If none are found, append it to the end.
```

### CutStart
```
Arguments:
    n: cut n nucleotides from the start of the read
    target: Read1|Read2|Index1|Index2 (default: read1)
```

### CutEnd
```
Arguments:
    n: cut n nucleotides from the end of the read
    target: Read1|Read2|Index1|Index2 (default: read1)
```

### MaxLen
```
Arguments:
    n: the maximum length of the read. Cut at end if longer 
    target: Read1|Read2|Index1|Index2 (default: read1)
```
### Reverse 
Reverse the read sequence.
```
Arguments:
    target: Read1|Read2|Index1|Index2 (default: read1)
```


### TrimPolyTail (todo)
Trim either a specific base repetition, or any base repetition at the end of the read.
```
Arguments:
    target: Read1|Read2|Index1|Index2 (default: read1)
    min_length: int, the minimum number of repeats of the base
    base: AGTCN., the 'base' to trim (or . for any repeated base)
    max_mismatche_rate: float 0..1, how many mismatches are allowed in the repeat
```


# Todo

### demultiplex
todo

### Remaining trimmomatic options not yet supported
```
LLUMINACLIP: Cut adapter and other illumina-specific sequences from the read.
SLIDINGWINDOW: Performs a sliding window trimming approach. It starts
scanning at the 5‟ end and clips the read once the average quality within the window
falls below a threshold.
MAXINFO: An adaptive quality trimmer which balances read length and error rate to
maximise the value of each read
LEADING: Cut bases off the start of a read, if below a threshold quality
TRAILING: Cut bases off the end of a read, if below a threshold quality

MINLEN: Drop the read if it is below a specified length
AVGQUAL: Drop the read if the average quality is below the specified level
TOPHRED33: Convert quality scores to Phred-33
TOPHRED64: Convert quality scores to Phred-64
````

### Remaining fastp not yet supported

- interleaved fastqs.

```
 -A, --disable_adapter_trimming       adapter trimming is enabled by default. If this option is specified, adapter trimming is disabled

  -a, --adapter_sequence               the adapter for read1. For SE data, if not specified, the adapter will be auto-detected. For PE data, this is used if R1/R2 are found not overlapped. (string [=auto])

      --adapter_sequence_r2            the adapter for read2 (PE data only). This is used if R1/R2 are found not overlapped. If not specified, it will be the same as <adapter_sequence> (string [=auto])

      --adapter_fasta                  specify a FASTA file to trim both read1 and read2 (if PE) by all the sequences in this FASTA file (string [=])

      --detect_adapter_for_pe          by default, the auto-detection for adapter is for SE data input only, turn on this option to


  -5, --cut_front                      move a sliding window from front (5') to tail, drop the bases in the window if its mean quality < threshold, stop otherwise.

  -3, --cut_tail                       move a sliding window from tail (3') to front, drop the bases in the window if its mean quality < threshold, stop otherwise.

  -r, --cut_right                      move a sliding window from front to tail, if meet one window with mean quality < threshold, drop the bases in the window and the right part, and then stop.

  -W, --cut_window_size                the window size option shared by cut_front, cut_tail or cut_sliding. Range: 1~1000, default: 4 (int [=4])

  -M, --cut_mean_quality               the mean quality requirement option shared by cut_front, cut_tail or cut_sliding. Range: 1~36 default: 20 (Q20) (int [=20])
      --cut_front_window_size          the window size option of cut_front, default to cut_window_size if not specified (int [=4])

      --cut_front_mean_quality         the mean quality requirement option for cut_front, default to cut_mean_quality if not specified (int [=20])

      --cut_tail_window_size           the window size option of cut_tail, default to cut_window_size if not specified (int [=4])

      --cut_tail_mean_quality          the mean quality requirement option for cut_tail, default to cut_mean_quality if not specified (int [=20])


      --cut_right_window_size          the window size option of cut_right, default to cut_window_size if not specified (int [=4])

      --cut_right_mean_quality         the mean quality requirement option for cut_right, default to cut_mean_quality if not specified (int [=20])

  -Q, --disable_quality_filtering      quality filtering is enabled by default. If this option is specified, quality filtering is disabled

  -q, --qualified_quality_phred        the quality value that a base is qualified. Default 15 means phred quality >=Q15 is qualified. (int [=15])


  -u, --unqualified_percent_limit      how many percents of bases are allowed to be unqualified (0~100). Default 40 means 40% (int [=40])


  -n, --n_base_limit                   if one read's number of N base is >n_base_limit, then this read/pair is discarded. Default is 5 (int [=5])

  -e, --average_qual                   if one read's average quality score <avg_qual, then this read/pair is discarded. Default 0 means no requirement (int [=0])

  -L, --disable_length_filtering       length filtering is enabled by default. If this option is specified, length filtering is disabled

  -l, --length_required                reads shorter than length_required will be discarded, default is 15. (int [=15])

      --length_limit                   reads longer than length_limit will be discarded, default 0 means no limitation. (int [=0])

  -y, --low_complexity_filter          enable low complexity filter. The complexity is defined as the percentage of base that is different from its next base (base[i] != base[i+1]).

  -Y, --complexity_threshold           the threshold for low complexity filter (0~100). Default is 30, which means 30% complexity is required. (int [=30])

      --filter_by_index1               specify a file contains a list of barcodes of index1 to be filtered out, one barcode per line (string [=])

      --filter_by_index2               specify a file contains a list of barcodes of index2 to be filtered out, one barcode per line (string [=])

      --filter_by_index_threshold      the allowed difference of index barcode for index filtering, default 0 means completely identical. (int [=0])

  -c, --correction                     enable base correction in overlapped regions (only for PE data), default is disabled

      --overlap_len_require            the minimum length to detect overlapped 

region of PE reads. This will affect overlap analysis based PE merge, adapter trimming and correction. 30 by default. (int [=30])

      --overlap_diff_limit             the maximum number of mismatched bases to detect overlapped region of PE reads. This will affect overlap analysis based PE merge, adapter trimming and correction. 5 by default. (int [=5])

      --overlap_diff_percent_limit     the maximum percentage of mismatched bases to detect overlapped region of PE reads. This will affect overlap analysis based PE merge, adapter trimming and correction. Default 20 means 20%. (int [=20])

  -p, --overrepresentation_analysis    enable overrepresented sequence analysis.

  -P, --overrepresentation_sampling    one in (--overrepresentation_sampling) reads will be computed for overrepresentation analysis (1~10000), smaller is slower, default is 20. (int [=20])

  -w, --thread                         worker thread number, default is 3 (int [=3])

  -s, --split                          split output by limiting total split file number with this option (2~999), a sequential number prefix will be added to output name ( 0001.out.fq, 0002.out.fq...), disabled by default (int [=0])

  -S, --split_by_lines                 split output by limiting lines of each file with this option(>=1000), a sequential number prefix will be added to output name ( 0001.out.fq, 0002.out.fq...), disabled by default (long [=0])

  -d, --split_prefix_digits            the digits for the sequential number padding (1~10), default is 4, so the filename will be padded as 0001.xxx, 0 to disable padding (int [=4])

further ideas:
quantifyRegions
take a region def [{target,start,len}] and dump sorted kmer count to.json

plots: use plotters-rs

demultiplex:
a) every bc combo define s a bucket.
b)reads start in the default bucket.
c) relevant transforms keep data per bucket (skip, head, dedup).
d)output looks at the bucket and writes I to the appropriate file
e)demultiplex is as simple as read barcode from region def (see quantifyRegions), hamming match to bucket, assign to read.
f) reads not matching a barcode stay in the default bucket
g) filename for default.bucket is different depending on wether we have a demultiplex
h) at most one demultiplex step. mostly a limitation in the bucket defa, but n^k is not fun and I don't see the use case.
I)we stay with the limitation that all transforms happen to all buckets. though I see a use case for reports and quantifyRegions especially, to identify undefined barcodes. could maybe add a toggle for "with barcode / wo barcode only" with the default being both? just dont want to have to define a bucket matching lang.

check out https://lib.rs/crates/gzp for Gzip writing in parallel. might read in parallel, but I don't think Gzip is amendable to that.

consider noodles or rust-bio for the fast parsing.

prepare benchmarks.

 also profile, do our many reallocs hurt us (I suspect not, my gut feeling is that we are essentially limited by the decompression speed i.e our runtime is basically "how fast can we read&decompress" plus the lag until the last reads have trickled through.
might try to split read and decompress?
but the os caches should do that anyway.
also Zstd should be much faster then.
try cargo flame to profile.
limit to one core for profiling?
https://unix.stackexchange.com/questions/23106/how-to-limit-a-process-to-one-cpu-core-in-linux#23109

if the splitting / newline searching is a concern, look into https://github.com/BurntSushi/memchr
but before that check out https://docs.rs/fastq/latest/fastq/
and
https://github.com/moold/kseq-rs

review https://github.com/angelovangel/faster for more statistics / a direct competitor.
new version of that https://github.com/angelovangel/faster2
https://bioinf.shenwei.me/seqkit/usage/
more stats to check out https://github.com/clwgg/seqstats

add subsample function. deterministic randomness though!

we might need to replace flate2, since it will fail with bgfz files that contain multiple Gzip blocks
try https://crates.io/crates/niffler for that. that also does Zstd apparently 


```

### Options
Options unreleated to the transformations

```
[options]
    thread_count = 12  # number of cores to use. default: -1 = all cores.
	block_size = 10_000 # how many reads per block to process

```
