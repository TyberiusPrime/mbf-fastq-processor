+++
title = "FAQ"
BookFlatSection = true
+++


## Why are there so few defaults?

mbf-fastq-processor is following the python mantra 'explicit is better than implicit'.

It's presumptuous to assume our user's use case, and mismatches between an assumed
and actual use case lead to unwelcome surprises that the user might only discover 
much later, if at all.

Defaults also make for difficult upgrade paths - you can't really change them later on
without silently breaking your user's outputs (They'll be different, but it will be 
not immediately clear to the user why).

Instead have a look at the how-to section.


## Empty Reads

Some of the modifying steps may produce empty reads.

Some downstream aligners, notably STAR, will fail on such empty reads
in FASTQ files (STAR specifically will complain that sequence length is unequal
quality length, even though they're both 0).

To remove such reads, deploy a [FilterEmpty]({{< relref "docs/reference/filter-steps/FilterEmpty.md" >}}) step after the trimming
(or a `FilterMinLen` step).


## Wrapped FASTQs

The FASTQ 'standard' ([Cock et al.](https://pmc.ncbi.nlm.nih.gov/articles/PMC2847217/)) 
allows for 'wrapped' sequence and quality lines, which contain newlines that are omitted
when parsing the file.

mbf-fastq-processor does currently not support such wrapped FASTQ files.

This variation seems to be very rare in the wild, at least for sequencing data - it might 
be different if you look at assemblies with quality data attached?

If this turns out to be necessary / requested, we'll have to rework the parser.


## Cuckoo Filtering

All steps that involve set-membership tests (
such as [TagDuplicates]({{< relref "docs/reference/tag-steps/tag/TagDuplicates.md" >}})
and [TagOtherFile]({{< relref "docs/reference/tag-steps/tag/TagOtherFile.md" >}}),
offer to use either an exact data structure (HashSet)
that uses a lot of RAM, or a probabilistic data (scalable Cuckoo filtering) structure which offers greatly reduced RAM 
usage, but has a (configurable) false positive rate.

Cuckoo filtering  is a stochastic algorithm, the reproducibility of which we enforce by requiring
a seed for it's randomness. Unfortunately, that seed is not the only (nuisance) parameter influencing
the outcome. The order of reads (both in the reference and your FASTQ file), the initial size of the 
filter, the growth rate and the false positive rate also influence the outcome. 

mbf-fastq-processor automatically chooses the initial size (aligned read count for index BAM files
as reference, 10 million otherwise) and the growth rate. You influence the input files and the chosen 
false positive rate.

The RAM usage for runs fairly linearly with the number of entries in the filter
(false positive rate 0.001, initial capacity 10 million):

| Entries         | RAM Usage |
|-----------------|-----------|
| 10 million      | ~28 MiB    |
| 30 million      | ~88 MiB    |
| 100 million     | ~220 MiB   |
| 200 million     | ~488 MiB   |

(In contrast, an exact filter of 100 million 150 bp reads will need in excess of 14 GiB, just for the read sequences).

The effect of the false positive rate at 100 million reads is roughly:
| FP         | RAM Usage |
|------------|-----------|
| 0.01       | ~175 MiB    |
| 0.001      | ~220 MiB    |
| 0.0001     | ~270 MiB    |
| 0.0001     | ~314 MiB    |
| 0.00001    | ~356 MiB    |
| 0.000001   | ~412 MiB   |
| 0.0000000001   | ~552 MiB   |


## BAM output missing @PG header
mbf-fastq-processor is not putting an @PG header into it's BAM files.
That is by design. 

One the one hand, our configuration file driven workflow really does not fit 
into the command line based concept @PG (and the CL field may not contain new lines).

On the other, @PG purposes to track BAM provenance, but your scientific workflow 
must do that for every type of file, not just BAM, so it's a blatant layering violation :).

Having it leads to hash instability of BAM files 
(since some tool will inevitably record the *order* of command line arguments even when tha doesn't matter).


## Security

mbf-fastq-processor is MIT licensed - no warranties of any kind.

That being said, we use cargo-deny to check our dependencies for known issues
(integrated into our release workflow).


