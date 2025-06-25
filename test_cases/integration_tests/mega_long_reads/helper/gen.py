import subprocess
import os
with open("../input_1.fq",'w') as op:
    op.write("@")
    op.write("A"* 10_000)
    op.write("\n")
    op.write("AGTC" * int(10e6//4))
    op.write("\n")
    op.write("+\n")
    op.write("B"* int(10e6//1))
    op.write("\n")

subprocess.check_call(['zstd', '../input_1.fq', '-o', '../input_1.fq.zst'])
os.unlink('../input_1.fq')
