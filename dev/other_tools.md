# Other tools

Tools I have examined, features I have adapt, or chosen not to adopt.


## fastp


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


```

# AfterQC
investigate https://github.com/OpenGene/AfterQC (it's a fastq predecestor, I don't expect many suprises)
(What is a bubble artifact though?)

## trimmomatic
```
LLUMINACLIP: Cut adapter and other illumina-specific sequences from the read.
(that's one underdocumented and non tested piece of algorithm...)
MAXINFO: An adaptive quality trimmer which balances read length and error rate to
maximise the value of each read
```

## faster
https://github.com/angelovangel/faster

(only new things listed)
 - geometric mean of pred scores 'per read' (guess that's the one one should filter on)
 - nx values e.g. N50


## faster2

https://github.com/angelovangel/faster2
new version of faster  
faster2 outputs
    'gc content per read' (really per read)
    -read lengths (again per read)j
    -avg qual per read (again per read)
    -nx50 (ie. shortest length af 50% of the bases covered, I believe). Useful for pacbio/oxford nanopore I suppose.
      (How can I calculate that 'streaming')
    -percentage of q score of x or higher (1..93???)
    --table gives us:
            file    reads    bases    n_bases    min_len    max_len    N50    GC_percent    Q20_percent
            ERR12828869_1.fastq.gz    25955972    3893395800    75852    150    150    150    49.91    97.64
    (and goes about 388k reads/s from an ERR12828869_1.fastq.gz, single core. 67s for the file. 430k/s for uncompressed. no Zstd)


## seqstats
    c, last update 7 years ago (very mature software, I suppose)
    total n, total seq, avng len, median len, n50, min len, max len
    very fast: 20.7s for gz, 11s for uncompressed, no zstd
        How is it decompressing the file so fast?
        gzip itself takes 29.5 seconds for me!.
        Pigz does it in 12.8s, so you *can* parallel decompress gzip..
        crabz doesn't manage the same trick cpu load (seems to stay single core), but does decompress in 11.2s/
        I think it's just choosing a different zlib? hm...

## seqkit
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

## cutadapt
    https://cutadapt.readthedocs.io/en/stable/index.html
    repo: https://github.com/marcelm/cutadapt/
    nixpkgs: no
    flakeable: in imtmarburg/flakes.

    Modifies and filters reads.

### adapter removal

    - can remove 5' or 3' adapters (with partial overlap.).
    - can remove 5' *and* 3' adapters (linked adapters), but only trim when both are present.
    - 'complex' adapter definitions (mas mismatc,h indels, alignment algorithm, iupac)
    - can remove adapters up to n times.
    - can trim so the adapter is cut of, or is cut of after the adapter
    - can mask the adapter sequence with N
    - can mark the adapter by turing it into lowercase
    - rev comp support.

Support: We have 3' adapter trimming in TrimAdapterMismatchTail

Todo: TrimAdapter 5'. 
todo: Trimming retaining adapter.
Todo: Trim requiring both adapters
Todo: FilterAdapterMissing
Todo: Masking
todo: lowercase matches?
todo: Rev comp search support?


### --cut length

Remove fixed number of bases.

Supported. CutStart|CutEnd

## Trim low quality bases

Supported. TrimQualityStart | TrimQualityEnd

## --nextseq-trim Trim dark cycles, trim poly
    Trim polyG with high quality at the end.

Todo: Check if implementation is compatibly with our polyTail trimming.

## --poly-a trim poly-a
Supported.

## trim-n 
Trim polyN from end and start.

Todo: Trim from start.

## --length-tag TAG

Insert a 'TAG=(len)' read 'comment' in the name.

Todo: Should be in our RenameRead step.

## --strip-suffix, -- prefix, --suffix, --rename

Supported: Rename.

template based renaming has the following fields:


    {header} – the full, unchanged header

    {id} – the read ID, that is, the part of the header before the first whitespace

    {comment} – the part of the header after the whitespace following the ID

    {adapter_name} – the name of adapter that was found in this read or no_adapter if there was no adapter match. If you use --times to do multiple rounds of adapter matching, this is the name of the last found adapter.

    {match_sequence} – the sequence of the read that matched the adapter (including errors). If there was no adapter match, this is set to an empty string. If you use a linked adapter, this is to the two matching strings, separated by a comma.

    {cut_prefix} – the prefix removed by the --cut (or -u) option (that is, when used with a positive length argument)

    {cut_suffix} – the suffix removed by the --cut (or -u) option (that is, when used with a negative length argument)

    {rc} – this is replaced with the string rc if the read was reverse complemented. This only applies when reverse complementing was requested.

    \t – not a placeholder, but will be replaced with the tab character.


Todo:  implement templates

##  --zero-cap

Clip negative quality values to zero.

Todo.

## --minimum-length --maximum-length, --max-n

Supported.

## -- max-expected-errors 
Discard reads whose expected number of errors exceeds the value E.
https://cutadapt.readthedocs.io/en/stable/algorithms.html#expected-errors

Todo.

## --discard-trimmed
Discard reads with adapter match.

Todo

## --discard-casava
discard reads that have ':Y:' in their name.
todo: FilterName(regexp...)


## json based report.
Todo: inspect what cutadapt outputs.
Example here:
https://cutadapt.readthedocs.io/en/stable/reference.html#json-report-format

## Can export filtered reads to other files.

Support: no.

## properly paired reads

cutadapt verifies that the names in r1/r2 match.
Todo: add as step

## seqfu
    https://telatin.github.io/seqfu2/tools/
    repo :https://github.com/telatin/seqfu2)
    nixpgs: no
    flakeable: the binaries at least in github:/imtmarburg/flakes/seqfu2
    language: python, c/c++, Nim
    test cases: many.
    locked build: no.
    CI: yes.
    publication: Telatin A, Fariselli P, Birolo G. SeqFu: A Suite of Utilities for the Robust and Reproducible Manipulation of Sequence Files. Bioengineering 2021, 8, 59. doi.org/10.3390/bioengineering8050059

    Command options

### · bases               

bases               : count bases in FASTA/FASTQ files
Described: : count bases in FASTA/FASTQ files. Print the DNA bases, and %GC content, in the input files

-> covered by our report.

### check

  · check               : check FASTQ file for errors

Implicit in our pipeline.

### deinterleave
  · deinterleave [dei]  : deinterleave FASTQ

Covered by our interlaved read with non-interleaved output.

###  · derep 
derep [der]         : feature-rich dereplication of FASTA/FASTQ files

We call this deduplication. It seems to work on complete sequences.

- can report count ('sizes') in name of the output
- Can filter for min count.
- can 'rename' sequencs.
- can aggregate (sizes) from multiple files
- optionally filter reads by min/max length.

Documentation doesn't describe algorithm,
but reading the source it's obviously an exact hash table.. 

todo: decide if we want to support the count use case.
If so, special case for FilterDuplicates


## interleave
  · interleave [ilv]    : interleave FASTQ pair ends

Supported in output options.


## lanes
  · lanes [mrl]         : merge Illumina lanes

A fast 'concat fq.gz' utility that turns a directory
of ID1_S99_L001_R1_001.fastq.gz like named files into
one fastq.gz per sample.

So essentially 'group by Sample/R2|R2', cat together.

It's claimed to be faster than than just cat-ing the gz files together.

I'm somewhat sceptical, the benchmark times are in single digit ms
(what kind of *tiny* fastqs.gz are that?), and the source suggest
it's actually parsing the files, and I can't see that being faster than
just streaming the bytes around.

Support: No, grouping the file names isn't appropriately placed in the level
we're working at.

## list
  · list [lst]          : print sequences from a list of names

Search | filter reads by prefix of their name.

This is essentially, given a list of these prefixes in a file list (prefix with @, or ^@)
`gzip -cd test.fq.gz | grep "$(cat list)"  -A3`


Support: Todo: extend FilterOtherFile to read fasta and newline separated lists. 
We'll require full names though. 

The prefix use case seem esoteric - grep might be better in that case.

##  metadata
  · metadata [met]      : print a table of FASTQ reads (mapping files)

Generates various metadata files for tools processing FASTQ files?

Support: Out of scope. If you can create our input.toml, you're probably already
in a place to create your required metadata files.

## rotate
  · rotate [rot]        : rotate a sequence with a new start position

Base pair shift in a ring buffer.

So AGTC, --start-pos 3 becomes TCAG (1 based, start-pos = 3 moves the first 2 bases to the end)

Also can rotate based on a oligonucleotide position (optionally including reverse complement).
Optional filtering of unmatched reads. 

I can't see the biological motivation vs *cutting* the read.

Support: Unsupported. Don't see the use case.

## sort
    · sort [srt]          : sort sequences by size (uniques)

Sorts sequences by *length* (not the same 'size' as in the dereplication tool, confusing),
and deduplicate.

Reads all sequences into memory.

Support: No. What's the use case, and would need a scalable implementation.

## stats
  · stats [st]          : statistics on sequence lengths

Reports 
    n50, n75 or N90
    count or counts (number of reads)
    sum or tot (total bases)
    min or minimum (minimum length)
    max or maximum (maximum length)
    avg or mean (average length)
    aun (area under the Nx curve)

Support: our reports offer counts, total bases, full length histogram (
 from which min/max/avg/ can be derived.

Todo: Consider whether we want to repotr N50, N75, N90, and AUN, so downstream
doesn't have to figure out how.

## cat
  · cat                 : concatenate FASTA/FASTQ files

Cat + options. 
Can 'Head', 'Skip', 'Sample' (by taking every nth read),

Can prefix, relabel, postfix, split names, prepend file base name.,
add 'sequence comments' such as len,  gc,  original-name, expected-error (+ 'before-processing')
variants, filter reads my N count, min/max length, max expected error, trim reads, truncate reads.

Can create a file of original-name -> new name.<D-q>

Writes fastq, fasta.

Todo: Truncate reads from the back (we have MaxLen - should rename that to Truncate)
Todo: Filter by max expected error.
Todo: FilterSampleNth 
Todo: Consider introducing replacables into Rename for input-file-basename, length, 
Todo: Consider reporting output  for Renome

Support: We have a regex based read rewriting. Could be improved.


## grep
  · grep                : select sequences with patterns

Filter to reads having substring in name, or matching name regex.
Optionally search the 'name comment' as well.

Filters by sequence: IUPAC. Can extract matches into the sequence comment.
With mismatches.


Todo: Name filter (regex), Sequence filter by IUPAC (with mismatches.



## · head                : print first sequences

Also samples, renames, removes comments.

Supported.

Todo: Either introduce NameReadsByNumber or include this in the read naming regex method.

## rc
  · rc                  : reverse complement strings or files

Supported in `ReverseComplement`

Todo: Rename `Reverse`  'flip' and describe in docs

## tab
· tab                 : tabulate reads to TSV (and viceversa)

Turn reads into tsv, and back again. PE support but needs to be interleaved.

Support: None. Trivial to implement, though would need to change output
to be format & compression separately.

Todo: Rename 'format' in output to 'compression' anyway.

## tail
  · tail                : view last sequences

Tails, samples, renames, 

  · view                : view sequences with colored quality and oligo matches

View FASTQ 'colorful', with >>>>> marked oligos (in two colors).

Support: out of scope.



## Other
Todo: Have a look at their test cases, maybe we can steal some.


## Summary
Some useful tools, some random stuff, a lot of heterogeneity, and no composition whatsoever.

