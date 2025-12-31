import pysam


a = pysam.Samfile("input_ERR12828869_10k_1.head_500.bam", "rb")
mapped = []
mapped_reads = set()
for read in a.fetch(until_eof=True):
    if len(mapped) < 250:
        if not read.is_unmapped:
            mapped.append(read)
            mapped_reads.add(read.qname)
    else:
        break

b = pysam.Samfile("input_ERR12828869_10k_1.head_500.all_unaligned.bam")
unmapped = []
for read in b.fetch(until_eof=True):
    if len(unmapped) < 250:
        if read.qname not in mapped_reads:
            unmapped.append(read)
    else:
        break


with pysam.Samfile(
    "input_ERR12828869_250_aligned_250_unaligned.bam", mode="wb", template=a
) as out:
    for read in mapped + unmapped:
        out.write(read)
pysam.index("input_ERR12828869_250_aligned_250_unaligned.bam")
