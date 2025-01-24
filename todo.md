- why are we slow in decompressing ERR13885883
    - as is                 ~ 44.7 s  (43.07 without output)
    - recompressed gz       - 44.7 s (42.39)
    - zstd                  - 43.53 s (24) 
    -> it's just slow to decompress?


### other

- Test with very long (1MB) reads.
- test for report across demultiplex boundaries
- stdin input (+- interleaved)
- CountForReport
- overrepresented regions
- refactor to take any number of input files, not just read1, read2, index1, index2
- or at least refactor that read1, (read2), index1, no index2 and keep_index works?


Report Maybe todo:

- reads with expected error rate < 1% (not quite q20 average)
- reads with expected error rate < 0.1% (not quite q30 average)
- output hash of the compressed data instead?


```
[[transform]]
    action = "CountForReport"
    tag = "Between Step 3 and 4"
```

Include a count of reads in this processing step in the report.
Does not cross 'demultiplex' boundaries.

### Remaining trimmomatic options we might support

```
LLUMINACLIP: Cut adapter and other illumina-specific sequences from the read.
(that's one underdocumented and non tested piece of algorithm...)
MAXINFO: An adaptive quality trimmer which balances read length and error rate to
maximise the value of each read

```

### Remaining ideas from other programs...

```
 -A, --disable_adapter_trimming       adapter trimming is enabled by default. If this option is specified, adapter trimming is disabled

  -a, --adapter_sequence               the adapter for read1. For SE data, if not specified, the adapter will be auto-detected. For PE data, this is used if R1/R2 are found not overlapped. (string [=auto])

      --adapter_sequence_r2            the adapter for read2 (PE data only). This is used if R1/R2 are found not overlapped. If not specified, it will be the same as <adapter_sequence> (string [=auto])

      --adapter_fasta                  specify a FASTA file to trim both read1 and read2 (if PE) by all the sequences in this FASTA file (string [=])

      --detect_adapter_for_pe          by default, the auto-detection for adapter is for SE data input only, turn on this option to



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


further ideas:

plots: use plotters-rs?

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

check out https://lib.rs/crates/gzp for Gzip writing in parallel.
might read in parallel, but I don't think Gzip is amendable to that.

prepare benchmarks.
- benchmark against fastp, faster, faster2, seqsstats

fastp
    - uses plotly for the graphs. Apperantly that's opensource now?
        I'd vendor the js though (it's giant... 1.24mb)

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
            file    reads    bases    n_bases    min_len    max_len    N50    GC_percent    Q20_percent
            ERR12828869_1.fastq.gz    25955972    3893395800    75852    150    150    150    49.91    97.64
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


seqfu
    https://telatin.github.io/seqfu2/tools/
    go through the list


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

