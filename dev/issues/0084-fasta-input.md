status: open
# Fasta input

why not... 

Just fake the quality scores (configurable).

Also add BAM input. 
That needs considerations regarding whether to include
mapped/unmapped reads. Default include both?

That means we need to introduce inputs.options,
blacklist options as a segment name.

struct InputOptions {
    fasta_fake_quality: Option<u8>, 
    bam_include_mapped: Option<bool>,
    bam_include_unmapped: Option<bool>
}

fasta must be set if a fasta input file is detected,
bam optinos must be set if a bam input file is detected.
(validation)

We'lle need a parser trait,
two more parsers for fasta / bam, move the fastq parser into it's own file
under parsers. BAM parser can use noodles...

Then InputFile must become an enum
Fastq(ex::fs::File), Fasta(ex::fs::File), Bam(whatever noodles has)
and it get's a get_parser method returning a boxed dyn Parser trait object.

