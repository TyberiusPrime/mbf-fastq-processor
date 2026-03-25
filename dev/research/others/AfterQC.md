Fastp Predecessor: AfterQC
https://github.com/OpenGene/AfterQC

Basic filters, polyX filters,
trim by qc at front and tail
correction of overlapping paired end reads
shift-barcode-to-molecule-name.
adapter cutting.
'sequencing error estimation'.


'detects and eliminates bubble artifacts'

It's not assembler preprocessing, but a phisical artifact of (old?, NextSeq, not HiSeq) illumina sequencers.
Local increased polyX density... find those spots, draw a circle in flow cell space, 
remove all reads in that circle.

Not sure it's even beneficial vs 'filter by alignment'.


Should skim the paper

https://bmcbioinformatics.biomedcentral.com/articles/10.1186/s12859-017-1469-3
