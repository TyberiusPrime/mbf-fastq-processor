import json
import io
import subprocess
from pathlib import Path

import collections


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


fn = Path("ERR664392_1250.fq.quantify.json")
if not fn.exists():
    kmers = collections.Counter()

    for read in read_fastq_iterator(open("ERR664392_1250.fq")):
        seq, name, quality = read
        kmer = seq[6:12]
        kmers[kmer] += 1

    fn.write_text(json.dumps(dict(kmers.items()), indent=4))

fn = Path("ERR12828869_10k_1.quantify.json")

def zstd_open(filename):
    # use zstd cli to open file into string, then stringio
    decoded = subprocess.check_output(["zstd", "-dc", filename]).decode('utf-8')
    return io.StringIO(decoded)

if not fn.exists():
    kmers = collections.Counter()

    for read1, read2 in zip(
        read_fastq_iterator(zstd_open("ERR12828869_10k_1.fq.zst")),
        read_fastq_iterator(zstd_open("ERR12828869_10k_2.fq.zst")),
    ):
        kmer = read1[0][6:12] + "xyz" + read2[0][10:17]
        kmers[kmer] += 1

    fn.write_text(json.dumps(dict(kmers.items()), indent=4))

