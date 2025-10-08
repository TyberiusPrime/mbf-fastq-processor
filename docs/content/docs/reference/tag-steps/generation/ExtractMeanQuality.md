### ExtractMeanQuality

We don't support calculating the 'average quality'. 

This is typically a bad idea, see https://www.drive5.com/usearch/manual/avgq.html for a discussion of the issues.

To illustrate, 140 x Q35 + 10 x Q2 reads have an 'average' phred of 33, but 6.4 expected wrong bases
A read with  150 x Q25 has a much wores 'average' phred of 25, but a much lower expected number of errors at 0.5!


## Corresponding options in other software 

- Trimmomatic: AVGQUAL:
- fastp: --average_qual
