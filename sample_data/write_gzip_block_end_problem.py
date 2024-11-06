# We had some issue in our parser when gzip wrote really tiny blocks
# that would not actually include a new line.
# this is big (and brokenish) enough to trigger al the cases in the parser
read = """@A02023:169:HKGW7DRX5:2:1167:3784:{i} 1:N:0:ACCTACAGACCT+ATTGCGTGATTG
GGTGTATATAAGGCGGAGGTTGCAGTGAGCTGGGGTCGTGCCATTGCACTCCAGCCTGGGCGACAGAGTAAGACTC
+
FFF:FFF,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF::FFFFF
""".encode('utf-8')
import gzip
import random


random.seed(42)
fh= open('test_gzip_block_unaligned.fastq.gz', 'wb')

for ii in range(0, 10000):
    out_read = read.replace(b'{i}', str(ii).encode('utf-8'))
    split = random.randrange(1, len(out_read))
    fh.write(gzip.compress(out_read[:split]))
    fh.write(gzip.compress(out_read[split:]))
    fh.flush()
