status: open
# read count estimation


The read count estimation has a couple of problems:

a) it's not considering multiple files per segment. That's fixable on the ChainedParser level.
b) If the buffer size from the underlying compression library exceeds what the first block
actually needs, we'll underestimate the read count.
This is a hard problem, I'd need 3 values, compressed bytes read, uncompressed bytes read,
and bytes-actually-used, and frankly I don't want to count the latter.


We could just assume a constant compression ratio (fastq runs about 1 byte per base on average)
and read length, and estimate from there instead...

