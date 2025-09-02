For the paper   
    - show fastp being unreproducible


# implementation and co:

- documentation for all the new things...

## Test case for StoreTagsInTable, but no tags definied, same for StoreTagsInSled

## AnnotateBamWithTags

## ConcatTags
Needs 'location less' tags.

## RewriteTag
with a regex

## invert filters

Some filters can invert (e.g. FilterOtherFile), some filters are inverse of each other
(e.g. FilterMinLen, FilterMaxLen), that's a hobgoblin, 
we want to have a consistent flag on the filters.

## StoreTagInSequence
if growing/shrinking, we currently eat all tag locations.
That's unnecessary, we can do better

## PE to SE with overlap 

(what do the other tools do here).

"""
   fastp: fastp perform overlap analysis for PE data, which try to find an overlap of each pair of reads. If an proper overlap is found, it can correct mismatched base pairs in overlapped regions of paired end reads, if one base is with high quality while the other is with ultra low quality. If a base is corrected, the quality of its paired base will be assigned to it so that they will share the same quality.  

    This function is not enabled by default, specify -c or --correction to enable it. This function is based on overlapping detection, which has adjustable parameters overlap_len_require (default 30), overlap_diff_limit (default 5) and overlap_diff_percent_limit (default 20%). Please note that the reads should meet these three conditions simultaneously.
"""

I think the implementation is just checking all possible offsets 
for whether it's a 'valid' overlap, starts with the longest possible overlap
and returns the fast one. At least that's how I glean the cpp.

Should be something we can show we're both better and faster on?

Threw up a mod. smith waterman from rust-bio that works nicely.

Need some test datasets to evaluate.

### insert size histogram 
  (fastp style 'overlaping reads processing'
    We have  the merging, we just need the statistics on this one.

## eserde

switch to https://github.com/mainmatter/eserde for improved error messages 
if multiple things are wrong- once it supports TOML
(I made a PR, number #48, still out 3weeks after)


## read1/read2/index1/index2 limitations

refactor to take any number of input files, not just read1, read2, index1, index2

A surprisingly big task.

Or maybe at least refactor that read1, (read2), index1, no index2 and keep_index works?

## CountForReport

```
[[transform]]
    action = "CountForReport"
    tag = "Between Step 3 and 4"
```

Include a count of reads in this processing step in the report.
Does not cross 'demultiplex' boundaries. (What did I mean by that?)



## stdin input (+- interleaved)
don't we have this?
Yeah, see test_input_interleaved

## Hash output

Output hash of the compressed data instead? that would allow the user to easily
check the files with sha256sum.


## overrepresented regions
 
 Ideas for overrepresented sequence finding
 - skip x reads
 - count 12mers. (2^24 of them) for the next n reads
 - for the next nx reads, 
     for each possible start pos
       calculate max occurance (using the kmer table from above),
       basically min (kmer split)
       if that's still above our enrichment threshold, count it
 - go through the counted kmers. Calculate enrichment based on their 
   actual counts. 
 - Remove all that are prefixes of others?
 - Report
       
# Investigate kalistoo BUS format
    might be useful for scRNAseq?

 

## further report ideas

Report Maybe todo:

- reads with expected error rate < 1% (not quite q20 average)
- reads with expected error rate < 0.1% (not quite q30 average)


# Out of scope

## - fast5 

    https://medium.com/@shiansu/a-look-at-the-nanopore-fast5-format-f711999e2ff6
    nanopore squiggle data.
    apparently no formal spec.

# Other

- why are we slow in decompressing ERR13885883
    - as is                 ~ 44.7 s  (43.07 without output)
    - recompressed gz       - 44.7 s (42.39)
    - zstd                  - 43.53 s (24) 
    -> it's just slow to decompress?
    - how fast is fastp



# Unsorted


check out https://lib.rs/crates/gzp for Gzip writing in parallel.
might read in parallel, but I don't think Gzip is amendable to that.

prepare benchmarks.
- benchmark against fastp, faster, faster2, seqsstats

review  for more statistics / a direct competitor.

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

ce writer with niffler  (but check out gpz first)

report ideas:
    -  Histogram of base quality scores (fastqc like, but not a line graph...)
    - sequence length histogram?
    - duplication distribution (how many how often...)
    - overrespresented sequences
        (I think fastp takes one in 20ish reads up to 10k to make this calculation? check the source.)


 regex based read filter.


- what is our maximum read length / test with pacbio data.

 

-- implement order shuffleing?
-- implement Collapse (remove true duplicates). Is this helpful?

-- remove reads with high kmers? https://lskatz.github.io/fasten/fasten_normalize/index.html
   - sounds like a multipass problem.

- low quality base to N


- implement a 'template' option that gives you a config file to work with.

- fold the testing back into the cargo harness, like mbf-bam-quantifier does

 -demultiplexing should happen on tags?

- document extractregex

- UppercaseTag

- do we need the separator on extract regions, or is the store-in-comment one enough?
-- support for tihs Read overlap detection
(from BD's rhapsody pipeline)

First, read 1 and read 2 are tested to see if they overlap, so that read 1 content can be removed from read 2.
This will prevent downstream mis-alignment and mis-assembly of any cell label sequences present in read 2.
An overlap detection percent metric is calculated and may help troubleshoot PCR cleanup and library
preparation steps. This overlap step does not remove any read pairs from subsequent steps.
Read 1 artifacts are removed from read 2 with the following steps:
l Read 1 and 2 are compared with a modified Knuth-Morris-Pratt substring search algorithm that allows
for a variable number of mismatches. The maximum mismatch rate is set to 9% by default with a
minimum overlap length of 25 bases. Read 1 is scanned right to left on the reverse complement of
read 2. The closest offset from the end of the reverse complement of read 2 with the lowest number of
mismatches (below the maximum mismatch rate threshold) is considered to be the best fit overlap.
l The merged read will be split back into a read pair. The merged read will be split according to the bead
specific R1 minimum length (described in Annotate R1 Cell Label and UMI (page 31)). The bases at the
beginning of the merged read up to the R1 minimum length, plus the length of the bead capture
sequence, will be assigned to read 1, and the rest will be assigned to read 2.
