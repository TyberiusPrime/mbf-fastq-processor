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

