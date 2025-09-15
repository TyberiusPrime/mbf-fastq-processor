We're currently missing a StoreTagInFastQStep.

It takes a (location) tag and stores it in an FASTQ file of arbitrary
filename. Naming Scheme is $output_prefix.tag.$tag_name.fastq.$suffix.
Compression is allowed just like in the regular output files.

Also takes a list of tags to put into the read name, and the same 
arguments that StoreTagInComment does.
