#Ktrim
Ktrim: an extra-fast and accurate adapter- and quality-trimmer for sequencing dataKtrim: an extra-fast and accurate adapter- and quality-trimmer for sequencing data

https://academic.oup.com/bioinformatics/article/36/11/3561/5803071
https://github.com/hellosunking/Ktrim/

- Includes common adapters
- Multithreded, should we benchmark this? I mean it should be slower than rabbittrim

- window based quality check,
- some kind of 'minimum quality score to keep the cycle',
- min read size, 
- adapter trimming.

gzip input, 



> 
Here are the built-in adapter sequences (the copyright should belong to the corresponding companies):

Illumina TruSeq kits:
AGATCGGAAGAGC (for both read 1 and read 2)

Nextera kits (suitable for ATAC-seq, Cut & tag data):
CTGTCTCTTATACACATCT (for both read 1 and read 2)

BGI adapters:
Read 1: AAGTCGGAGGCCAAGCGGTC
Read 2: AAGTCGGATCGTAGCCATGT

CLIP-seq adapters:
Read 1: TGGAATTCTCGGGTGCCAAGG
Read 2: GATCGTCGGACTGTAGAACTCTGAAC

(that goes into our cookbook)



c++

Seems to include the binary in the github repo???
