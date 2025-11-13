import random
import urllib.request
import gzip
from pathlib import Path
urls = ["http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR777/ERR777676/ERR777676_1.fastq.gz",
   "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/ERR777/ERR777676/ERR777676_2.fastq.gz"]

for url in urls:
    output_name = Path('input_' + url.rsplit("/",1)[-1])
    if not output_name.exists():
        # download, split into reads, sample 2k reads with fixed seed,
        # write to output file
        raw = urllib.request.urlopen(url)
        decompressed = gzip.GzipFile(fileobj=raw)
        lines = decompressed.readlines()
        of = gzip.GzipFile(output_name,'w')
        random.seed(42)
        for read in random.sample(range(0, len(lines)//4), 2000):
        #for read in range(0, len(lines)//4):
            for i in range(4):
                of.write(lines[read*4 + i])
        print("Wrote sampled reads to", output_name)


