---
not-a-transformation: true
---
# Out of scope

Things mbf-fastq-processor will explicitly not do and that won't be implemented.

## Anything based on averaging phred scores

Based on the average quality in a sliding window.
Arithmetic averaging of phred scores is wrong.

see [ExtractMeanQuality]({{< relref "docs/reference/tag-steps/calc/CalcMeanQuality.md" >}})


### Corresponding options in other software 
- Trimmomatic SLIDINGWINDOW
- fastp --cut_front
- fastp --cut_tail
- fastp --cut_right

## Fast5

https://medium.com/@shiansu/a-look-at-the-nanopore-fast5-format-f711999e2ff6
Oxford Nanopore squiggle data.
Apparently no formal spec.


## kallisto BUS format 
    - a brief barcode/umi format for single cell RNA-seq
    - needs an 'equivalance class' - i.e. at least pseudo alignment
    - weird length restrictions on barcodes and umis (1(!)-32), 
      but stores the length in an uint32...


## Alignment

While it's tempting to leverage the fastq parsing for an aligner,
aligning molecules to references is out of scope for the 1.0 target.

