
https://github.com/jdidion/atropos
 a cutadapt fork wit hmulti threading 
(they seem to have a parallel write trick?)

"""Implementation of a new insert alignment-based trimming algorithm for paired-end reads that is substantially more sensitive and specific than the original Cutadapt adapter alignment-based algorithm. This algorithm can also correct mismatches between the overlapping portions of the reads."""

(merging is experimental though?)

an adapter-and-other-potential-contaminats detection mode.

- the error command estimates sequencing error rate helps with thresholds,
maybe interesting for our plots..

- can write (unaligned) BAM
