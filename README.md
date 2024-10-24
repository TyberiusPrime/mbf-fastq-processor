# mbf_fastq_processor


The swiss army knife of fastq (pre-)processing.


It filters, samples, slices, dices, analysis(*), demultiplexes (*) and generally
does all the things you might want to do with a set of fastq files. 

(* yet to be implemented).

It's primary concern is correctness.
And flexibility.

It's two primary concerns are correctness and flexibility, and speed.

It's three main objectives are correctness, flexibility, speed and reproducible results.

Among it's objectives...


# Status

It's in beta until the 1.0 release.
The basic functionality and testing is in place,
what's currently lacking is advanced features (everything 
releated to adapters, the demultiplexing, deduplication, 
pretty reporting (json is available)), 


# Installation

This is a [nix flake](https://nixos.wiki/wiki/flakes) exporting a defaultPackage.

There are statically-linked binaries in the github releases section that will run on any linux with a recent enough glibc.

Currently not packaged by any distribution.

# Usage

`mbf_fastq_processor what_do_to.toml`

We use a [TOML](https://toml.io/en/) file, 
because command lines are limited and prone to misunderstandings. 

And you should be writing down what you are doing anyway.

Here's a brief example:

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
    html = true # to be implemented.

[[transform]]
    # take the first five thousand reads
	action = "Head"
	n = 5000

[[transform]]
	# extract umi and place it in the read name
	action = "ExtractToName"
    # the umi is the first 8 bases of read1
    source = 'Read1'
    start = 0
    length = 8

[[report]]
    infix = "post_filter"
    json = true
    html = true # to be implemented.

[output]
    #generates output_1.fq and output_2.fq. For index reads see below.
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

### Report  (todo)
Write a statistics report, either machine-readable (json) 
or human readable (HTML with fancy graphs.

You can add multiple reports, at any stage of your transformation chain 
to get e.g. before/after filtering reports

```
Arguments:
    infix = "report" # str, a string to insert into the filename, betwen output.prefix and .html/json
    html= true # bool, wether to output html report (not yet implemented)
    json= true # bool, wether to output json report
```

Statistics available:

* read counts
* total base count
* bases count q20 or better
* bases count q30 or better
* read length distribution
* AGTCN counts at each position
* expected error rate at each position

Maybe todo:
 
* reads with expected error rate < 1% (not quite q20 average)
* reads with expected error rate < 0.1% (not quite q30 average)



### Progress

```
[[transform]
   action = "Progress" 
   n = 100_000
```
Every n reads, report on total progress, total reads per second, and thread local progress/reads per second

## Available transformations

Think of the transformations as defining a graph that starts with the input files,
and ends in the respective number of output files.

If the transformation splits the streams (think demultiplex), 
all subsequent transformations are applied to each stream.

Filters always remove complete 'molecules', not just a read1.

Many transformations take a target, which is one of Read1, Read2, Index1, Index2,
on which they work on, or base their decisions on.

Some 'Transformations' are no-ops done for side effects, like Progress
or Report.

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

### ExtractToName   
Extract a sequence from the read and place it in the read name, for example for an UMI.

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


### TrimPolyTail 
Trim either a specific base repetition, or any base repetition at the end of the read.
```
Arguments:
    target: Read1|Read2|Index1|Index2 (default: read1)
    min_length: int, the minimum number of repeats of the base
    base: AGTCN., the 'base' to trim (or . for any repeated base)
    max_mismatche_rate: float 0..=1, how many mismatches are allowed in the repeat
```

### TrimQualityStart
Cut bases off the start of a read, if below a threshold quality.

Trimmomatic: LEADING 

```
Arguments:
    min - minimum quality to keep (in whatever your score is encoded in)
          either a char like 'A' or a number 0..128 (typical phred score is 33..75)
    target - which Read1|Read2|Index1|Index2 to modify
```

### TrimQualityEnd
Cut bases off the end of a read, if below a threshold quality.

Trimmomatic: TRAILING 
```
Arguments:
    min - minimum quality to keep (in whatever your score is encoded in.) 
          either a char like 'A' or a number 0..128 (typical phred score is 33..75)
    target - which Read1|Read2|Index1|Index2 to modify
```

### FilterMinLen

Drop the molecule if the read is below a specified length.

Trimmomatic MINLEN 

fastp: --length_required                   

```
Arguments:
    n - minimum length
    target - which Read1|Read2|Index1|Index2 to filter on 
```

### FilterMaxLen

Drop the molecule if the read is above a specified length.

fastp: --length_limit                   

```
Arguments:
    n - minimum length
    target - which Read1|Read2|Index1|Index2 to filter on 
```



###  FilterMeanQuality
Drop the molecule if the average quality is below the specified level.
This is typically a bad idea see https://www.drive5.com/usearch/manual/avgq.html
Trimmomatic: AVGQUAL: 

fastp: --average_qual                   
```
Arguments:
    min - (float) minimum average quality to keep (in whatever your score is encoded in.
          Typical Range is 33..75)
    target - which Read1|Read2|Index1|Index2 to filter on
```

### FilterQualifiedBases
Filter by the maximum percentage of bases that are 'unqualified', that is below a threshold.

fastp : --qualified_quality_phred / --unqualified_percent_limit    

```
Arguments:
    min_quality: u8, the quality value >= which a base is qualified. In your phred encoding. Typically 33..75
    min_percentage: the minimum percentafe of qualified bases necessary (0..=1)
    target: Read1|Read2|Index1|Index2 
```

### FilterTooManyN
Filter by the count of N in a read.

fastp: --n_base_limit                   

```
Arguments:
    n: u8, the maximum number of Ns allowed
    target: Read1|Read2|Index1|Index2 
```

### FilterSample
Randomly sample a percentage of reads.
Requires a random seed, so always reproducible
```
Arguments:
    p - 0..=1, the chance for any given read to be kept
    seed - u64, the seed for the random number generator
```


### ValidateSeq
Validate that only allowed characters are in the sequence.
```
Arguments:
    allowed = string. Example 'ACGTN', the allowed characters
    target = Read1|Read2|Index1|Index2 
```

### ValidatePhred
Validate that all scores are between 33..=41
```
Arguments:
    target = Read1|Read2|Index1|Index2 
```

### ConvertPhred64To33
Older Illumina data had a different encoding for the quality stores,
starting at 64 instead of 33.
This transformation converts the quality scores to the 33 encoding.
(Inspired by trimmomatic TOPHRED33)

```
(no arguments, always applies to all your reads)
```

## Options
Options unrelated to the transformations

```
[options]
    thread_count = 12  # number of cores to use. default: -1 = all cores.
    block_size = 10_000 # how many reads per block to process
```

Thread_count is in addition to the input & output threads, 
and controls how many concurrent 'processing' threads are used.


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

````

### Remaining ideas from other programs...

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


  -L, --disable_length_filtering       length filtering is enabled by default. If this option is specified, length filtering is disabled


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

  -s, --split                          split output by limiting total split file number with this option (2~999), a sequential number prefix will be added to output name ( 0001.out.fq, 0002.out.fq...), disabled by default (int [=0])

  -S, --split_by_lines                 split output by limiting lines of each file with this option(>=1000), a sequential number prefix will be added to output name ( 0001.out.fq, 0002.out.fq...), disabled by default (long [=0])

  -d, --split_prefix_digits            the digits for the sequential number padding (1~10), default is 4, so the filename will be padded as 0001.xxx, 0 to disable padding (int [=4])

further ideas:
quantifyRegions
take a region def [{target,start,len}] and dump sorted kmer count to a json ( barcode hunt...)

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

consider noodles or rust-bio for the fast parsing (we got a custom non alloc parser now).

prepare benchmarks.
- benchmark against fastp, faster, faster2, seqsstats


fastp 
    - uses plotly for the graphs. Apperantly that's opensource now?
        I'd vendor the js though (it's giant... 1.24mb)
    
-

do our many reallocs hurt us (Not in the transformations, but the parsing was massively allocation bound)

review https://github.com/angelovangel/faster for more statistics / a direct competitor.
(only new things listed)
 - geometric mean of pred scores 'per read' (guess that's the one one should filter on)
 - nx values e.g. N50

new version of that https://github.com/angelovangel/faster2
faster2 outputs 
    'gc content per read' (really per read)j
    -read lengths (again per read)j
    -avg qual per read (again per read) 
    -nx50 (ie. shortest length af 50% of the bases covered, I believe). Useful for pacbio/oxford nanopore I suppose.
      (How can I calculate that 'streaming')
    -percentage of q score of x or higher (1..93???)
    --table gives us:
            file	reads	bases	n_bases	min_len	max_len	N50	GC_percent	Q20_percent
            ERR12828869_1.fastq.gz	25955972	3893395800	75852	150	150	150	49.91	97.64
    (and goes about 388k reads/s from an ERR12828869_1.fastq.gz, single core. 67s for the file. 430k/s for uncompressed. no Zstd)

seqstats: 
    c, last update 7 years ago (very mature software, I suppose)
    total n, total seq, avng len, median len, n50, min len, max len
    very fast: 20.7s for gz, 11s for uncompressed, no zstd
        How is it decompressing the file so fast? 
        gzip itself takes 29.5 seconds for me!. 
        Pigz does it in 12.8s, so you *can* parallel decompress gzip..
        crabz doesn't manage the same trick cpu load (seems to stay single core), but does decompress in 11.2s/
        I think it's just choosing a different zlib? hm...

seqkit.
    -detailed sequenc length distribution (min,max,mean, q1,q2,q3), 
    - 'number of gaps' (?, is that a space? no, '- .' is the default, it's configurable.)
    -L50 - https://en.wikipedia.org/wiki/N50,_L50,_and_related_statistics#L50
    - optional: other NX values
    -sana 'skip malformed records' in fastq.
    -conversions fq to fasta, fasta2fq, a tab conversion.
    -search by iupac?
    -fish 'looc for short sequences in larger sequneces using local alignment
    -filter duplicates by id, name ,sequence,
    -find common entries between files
    - regex name replacement
    -duplicate id fixer.
    -shuffle (not on fastq though)

cutadapt
    -adapter removal    
    -quality trimming
    -nextseq polyG trimming (like quality trimming, but G bases are ignored).
    -readname prefix, postfix, add length=, strip_suffix.


open questions:
    - how does fastp determine the false positive rate for it's 'hash filter' (some kind of bloom filter I think).
    - what's the usual adapter sequences, how does the adapter based trimming work anyway, check out cutadapt?
        see https://cutadapt.readthedocs.io/en/stable/guide.html#adapter-types
        https://cutadapt.readthedocs.io/en/stable/algorithms.html
        https://support.illumina.com/downloads/illumina-adapter-sequences-document-1000000002694.html


other quality encodings:
 fastq quality encoding. available values: 'sanger'(=phred33), 'solexa',
                             'illumina-1.3+', 'illumina-1.5+', 'illumina-1.8+'. 
Illumina 1.8+ can report scores above 40!
(default "sanger")
 see https://bioinf.shenwei.me/seqkit/usage/#convert




- idea have Progress not output a new line each time.

https://bioinf.shenwei.me/seqkit/usage/
more stats to check out https://github.com/clwgg/seqstats

- validator tha the fastq contains only DNA or AGTCN?

ce writer with niffler  (but check out gpz first)

report ideas: 
    -  Histogram of base quality scores (fastqc like, but not a line graph...)
    - sequence length histogram?
    - duplication distribution (how many how often...)
    - overrespresented sequences
        (I think fastp takes one in 20ish reads up to 10k to make this calculation? check the source.)
 


- Regex based barcode extractor https://crates.io/crates/barkit
- regex based read filter.


- what is our maximum read length / test with pacbio data.

```


