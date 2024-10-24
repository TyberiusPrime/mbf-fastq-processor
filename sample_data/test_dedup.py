def read_fastq_iterator(file_object):
    """A very dump and simple fastq reader, mostly for testing the other more sophisticated variants

    Yield (seq, name, quality)
    """
    row1 = file_object.readline()
    row2 = file_object.readline()
    row3 = file_object.readline()
    row4 = file_object.readline()
    while row1:
        seq = row2[:-1]
        quality = row4[:-1]
        name = row1[1:-1]
        yield (seq, name, quality)
        row1 = file_object.readline()
        row2 = file_object.readline()
        row3 = file_object.readline()
        row4 = file_object.readline()


def get_duplicates(fn):
    seen = set()
    dups = set()
    for seq, name, qual in read_fastq_iterator(open(fn)):
        if seq in seen:
            dups.add(name.rsplit("/")[0])
        seen.add(seq)
    return dups


a = get_duplicates("ERR12828869_10k_1.fq")
b = get_duplicates("ERR12828869_10k_2.fq")
print(len(a), len(b), len(a.intersection(b)), len(a.union(b)))


actual = set([x[1].split("/")[0] for x in read_fastq_iterator(open("output_1.fq"))])

def get_duplicates(fn1, fn2):
    seen = set()
    dups = set()
    for (seq, name, qual), (seq2, name2, qual2) in zip( read_fastq_iterator(open(fn1)), read_fastq_iterator(open(fn2))):
        if (seq, seq2) in seen:
            dups.add(name.rsplit("/")[0])
        seen.add((seq, seq2))
    return dups


print(len(get_duplicates("ERR12828869_10k_1.fq", "ERR12828869_10k_2.fq")))




