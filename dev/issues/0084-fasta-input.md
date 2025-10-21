status: done
# Fasta and BAM input

Just fake the quality scores (configurable).

Also add BAM input. 
That needs considerations regarding whether to include
mapped/unmapped reads. Default include both?

That means we need to introduce Inputs.options,
blacklist options as a segment name.

struct InputOptions { // lives in config/mod.rs
    fasta_fake_quality: Option<u8>, 
    bam_include_mapped: Option<bool>,
    bam_include_unmapped: Option<bool>
}

Fasta option must be set if a fasta input file is detected,
bam options must be set if a bam input file is detected.
(validation of from Config.check() introduce new Config.check_input())

We'll need a parser trait,
trait Parser {
    pub fn parse(&mut self) -> Result<(FastQBlock, bool)>;
}
For FastQ that's just a wrapper around the existing parser functions.

And two more parsers for fasta / BAM. 

Move the fastq parser into it's own file under src/parsers. 
BAM parser can use noodles, apply_to_readnames has an example on how to read BAM with noodles.

For Fasta parsing, check rust-bio: https://docs.rs/bio/latest/bio/io/fasta/
We don't need a hand made parser at this point.


Then config::InputFile must become an 
enum {
    Fastq(ex::fs::File), 
    Fasta(ex::fs::File), 
    Bam(whatever noodles handle is)
}
and it get's a get_parser method returning a Box<dyn Parser> trait object.


Format detection should be automatic and happe non magic bytes.
BAM starts with "BAM\1". Fasta with a '>', and FastQ with a '@' at the start of the file.



Testing: Add test cases reading a BAM file / a Fasta file and converting it into FastQ.
We can symlink the file in test_cases/integration_tests/filter_other_file_by_seq_remove_bam_unaligned/input_ERR12828869_10k_1.head_500.all_unaligned.bam as our test input for all unaligned reads.

We'll need to fake a BAM file with aligned reads though.

Documentation: We'll need to extend the input section both in the markdown and the template.toml documentation.
