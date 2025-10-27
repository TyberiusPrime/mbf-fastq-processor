status: open
# Zero copy fasta parser

Our fasta parsing is  using bio::io::fasta::FastaRecord/FastaRead
and thats' not as zero-copy as our fastq reader is

