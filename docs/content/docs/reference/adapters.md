---
weight: 1000
---

# Sequencing Adapters

## Illumina

From (https://support-docs.illumina.com/SHARE/AdapterSequences/adapter-sequences.htm)

* Prep & Nextera, & TruSight DNA Enrichment  - `CTGTCTCTTATACACATCT`
* miRNA `AGATCGGAAGAGCACACGTCTGAACTCCAGTCA`
* AmpliSeq `CTGTCTCTTATACACATCT`
* TruSeq Read1: `AGATCGGAAGAGCACACGTCTGAACTCCAGTCA`
* TruSeq Read2: `AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT`

## BGI

See [here](http://seqanswers.com/forums/showthread.php?t=87647) for the thread (2nd post).

* Forward filter:  `AAGTCGGAGGCCAAGCGGTCTTAGGAAGACAA`
* Reverse filter:  `AAGTCGGATCGTAGCCATGTCGTTCTGTGAGCCAAGGAGTTG`

## From fastp:

You can find more adapters in the [fastp source](https://github.com/OpenGene/fastp/blob/master/src/knownadapters.h)

