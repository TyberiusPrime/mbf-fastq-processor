import shutil
import os
import subprocess
shutil.copy("../../../../sample_data/zstd/input_read1.fq.zst", "input_read1.fq.zst")

try:
    os.unlink('input_read1.fq')
except FileNotFoundError:
    pass

subprocess.run(["zstd", "-d", "input_read1.fq.zst"], check=True)

op = open('input_read1.fq', 'rb').read()

first_newline = op.find(b'\r\n')
second_newline = op.find(b'\r\n', first_newline + 2)
# now replace the second newline with \r!\n
op = op[:second_newline] + op[second_newline:].replace(b"\r\n", b"\r!\n")

open("input_read1.fq", 'wb').write(op)

os.unlink('input_read1.fq.zst')
subprocess.run(["zstd", "input_read1.fq"], check=True)
