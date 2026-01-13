read1 = "CCTACGGGAGGCAGCAGTGGGGAATATTGCGCAATGGGCGAAAGCCTGACGCAGCGACGCCGCGTGAGGGATGAAGGTCTTCGGATCGTAAACCTCTGTCAGAAGGGAAGAAACTAGGGTGTTCTAATCAGCATCCTACTGACGGTACCTTCAAAGGAAGCACCGGCTAACTCCGTGCCAGCAGCCCCGGTAATACGGAGGGTNCAAGCGTTAATCGGGATCACTGGGCGTAAAGCGCACGTAGGCTGTT"
read2= "AACAGCCTACGTGCGCTTTACGCCCAGTGATCCCGATTAACGCTTGNACCCTCCGTATTACCGGGGCTGCTGGCACGGAGTTAGCCGGTGCTTCCTTTGAAGGTACCGTCAGTAGGATGCTGATTAGAACACCCTAGTTTCTTCCCTTCTGACAGAGGTTTACGATCCGAAGACCTTCATCCCTCACGCGGCGTCGCTGCGTCAGGCTTTCGCCCATTGCGCAATATTCCCCACTGCTGCCTCCCGTAGG"

names = []
reads1 = []
reads2 = []

def add(name, read1, read2):
    if not read1 or not read2:
        return # fastp always *filters* reads where one is empty. We don't so we don't test this

    if name in names:
        raise ValueError("duplicate name", name)
    try: 
        ri1 = reads1.index(read1)
    except ValueError:
        ri1 = -1
    try:
        ri2 = reads2.index(read2)
    except ValueError:
        ri2 = -1
    if ri1 != -1 and ri1 == ri2: # already present
        print("Skipping duplicate", name)
    names.append(name)
    reads1.append(read1)
    reads2.append(read2)

for i in range(len(read1)):
    shifted_read2 = ('A' * i) + read2[-i:]
    add(f'Xread2_right_shifted_by_{i}', read1, shifted_read2)

for i in range(len(read1)):
    shifted_read2 =  read2[-i:] + ('A' * i) 
    add(f'Xread2_left_shifted_by_{i}', read1, shifted_read2)

for i in range(len(read1)):
    shifted_read1 = read1[i:] + ('A' * i)
    add(f'Xread1_left_shifted_by_{i}', shifted_read1, read2)


for i in range(len(read1)):
    shifted_read1 = ('A' * i) + read1[i:] 
    add(f'Xread1_right_shifted_by_{i}', shifted_read1, read2)

def left_shift(r, l, fill='A'):
    return r[l:] + fill * l

def right_shift(r, l, fill='A'):
    return fill * l + r[:-l]


assert left_shift("AGTC",1) == "GTCA"
assert left_shift("AGTC",2) == "TCAA"
assert left_shift("AGTC",3) == "CAAA"
assert left_shift("AGTC",4) == "AAAA"

assert right_shift("AGTC",1) == "AAGT"
assert right_shift("AGTC",2) == "AAAG"
assert right_shift("AGTC",3) == "AAAA"
assert right_shift("AGTC",3, 'C') == "CCCA"
assert right_shift("AGTC",4) == "AAAA"


for ii in range(len(read1)):
    add(f"read1_left_shift_{ii}", left_shift(read1,ii), read2)
    add(f"read1_right_shift_{ii}", right_shift(read1,ii), read2)

for ii in range(len(read2)):
    add(f"read2_left_shift_{ii}", read1, left_shift(read2, ii))
    add(f"read2_right_shift_{ii}", read1, right_shift(read2, ii))


with open("reads_1.fq", "w") as f1, open("reads_2.fq", "w") as f2:
    for i in range(len(reads1)):
        f1.write(f"@{names[i]}/1\n{reads1[i]}\n+\n{'I'*len(reads1[i])}\n")
        f2.write(f"@{names[i]}/2\n{reads2[i]}\n+\n{'I'*len(reads2[i])}\n")

#now gzip all .fq
import subprocess
subprocess.run(["gzip", "reads_1.fq"])
subprocess.run(["gzip", "reads_2.fq"])
