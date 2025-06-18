import pysam

# Open input BAM and create output BAM
in_bam = pysam.AlignmentFile("ERR12828869_10k_1.head_500.bam", "rb")
out_bam = pysam.AlignmentFile("ERR12828869_10k_1.head_500.all_unaligned.bam", "wb", header=in_bam.header)

for read in in_bam:
    read.is_unmapped = True
    read.is_secondary = False
    read.is_supplementary = False
    read.reference_id = -1
    read.reference_start = -1
    read.cigar = None
    read.next_reference_id = -1
    read.next_reference_start = -1
    read.template_length = 0
    out_bam.write(read)

in_bam.close()
out_bam.close()

