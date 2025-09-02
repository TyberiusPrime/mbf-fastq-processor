---
not-a-transformation: true
---
# Out of scope

Things mbf-fastq-processor will explicitly not do and that won't be implemented.

## Anything based on averaging phred scores

Based on the average quality in a sliding window.
Arithmetic averaging of phred scores is wrong.


### Corresponding options in other software 
- Trimmomatic SLIDINGWINDOW
- fastp --cut_front
- fastp --cut_tail
- fastp --cut_right


