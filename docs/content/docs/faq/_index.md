+++
title = "FAQ"
BookFlatSection = true
+++


## Why are there so few defaults?

mbf-fastq-processor is following the python mantra 'explicit is better than implicit'.

It's presumptuous to assume our user's use case, and mismatches between an assumed
and actual use case lead to unwelcome surprises that the user might only discover 
much later if at all.

Defaults also make for difficult upgrade paths - you can't really change them later on
without silently breaking your user's outputs (They'll be different, but it will be 
not immediately clear to the user why).

Instead have a look at the how-to section.


## Empty Reads

Some of the modifying steps may produce empty reads.

Some downstream aligners, notably STAR, will fail on such empty reads
in FastQ files (STAR specifically will complain that sequence length is unequal
quality length, even though they're both 0).

To remove such reads, deploy a [FilterEmpty](../reference/filter-steps/filterempty) step after the trimming
(or a [FilterMinLen](../reference/filter-steps/filterminlen)).
