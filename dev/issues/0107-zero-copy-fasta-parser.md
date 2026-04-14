status: closed
# Zero copy fasta parser

Our fasta parsing is  using bio::io::fasta::FastaRecord/FastaRead
and thats' not as zero-copy as our fastq reader is

But then, fasta is an inherently more 'you have to copy to remove newlines'
protocol.

And since we're still allocating into an arena,
I'm going to close this.

