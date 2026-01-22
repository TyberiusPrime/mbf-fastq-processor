import subprocess
from pathlib import Path
import collections


def count_in_fastq(file_path):
    counts = collections.defaultdict(int)
    file_path = Path(file_path).resolve()


    zstd_decoded = subprocess.check_output(['zstd','-cd', file_path]).decode('utf-8')


    next_is_seq = False
    for line in zstd_decoded.split('\n'):
        if line.startswith('@'):
            next_is_seq = True
        elif next_is_seq:
            next_is_seq = False
            key = line[:4]
            counts[key] += 1
    return counts

for k,v in count_in_fastq('input_read1.fq.zst').items():
    if v > 1:
        break
else:
    raise ValueError("All sequences in input_read1.fq.zst are unique, cannot perform the test.")



for k,v in count_in_fastq('output_read1.fq.zst').items():
    assert v == 1

print('ok')
