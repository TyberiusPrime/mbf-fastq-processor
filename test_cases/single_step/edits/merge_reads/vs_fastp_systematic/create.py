read1 = "CCTACGGGAGGCAGCAGTGGGGAATATTGCGCAATGGGCGAAAGCCTGACGCAGCGACGCCGCGTGAGGGATGAAGGTCTTCGGATCGTAAACCTCTGTCAGAAGGGAAGAAACTAGGGTGTTCTAATCAGCATCCTACTGACGGTACCTTCAAAGGAAGCACCGGCTAACTCCGTGCCAGCAGCCCCGGTAATACGGAGGGTNCAAGCGTTAATCGGGATCACTGGGCGTAAAGCGCACGTAGGCTGTT"
read2= "AACAGCCTACGTGCGCTTTACGCCCAGTGATCCCGATTAACGCTTGNACCCTCCGTATTACCGGGGCTGCTGGCACGGAGTTAGCCGGTGCTTCCTTTGAAGGTACCGTCAGTAGGATGCTGATTAGAACACCCTAGTTTCTTCCCTTCTGACAGAGGTTTACGATCCGAAGACCTTCATCCCTCACGCGGCGTCGCTGCGTCAGGCTTTCGCCCATTGCGCAATATTCCCCACTGCTGCCTCCCGTAGG"

names = []
reads1 = []
reads2 = []

for i in range(len(read1)):
    shifted_read2 = ('A' * i) + read2[-i:]
    reads1.append(read1)
    reads2.append(shifted_read2)
    names.append(f'read2_right_shifted_by_{i}')

for i in range(len(read1)):
    shifted_read2 =  read2[-i:] + ('A' * i) 
    reads1.append(read1)
    reads2.append(shifted_read2)
    names.append(f'read2_left_shifted_by_{i}')


for i in range(len(read1)):
    shifted_read1 = read1[i:] + ('A' * i)
    reads1.append(shifted_read1)
    reads2.append(read2)
    names.append(f'read1_left_shifted_by_{i}')


for i in range(len(read1)):
    shifted_read1 = ('A' * i) + read1[i:] 
    reads1.append(shifted_read1)
    reads2.append(read2)
    names.append(f'read1_right_shifted_by_{i}')

with open("reads_1.fq", "w") as f1, open("reads_2.fq", "w") as f2:
    for i in range(len(reads1)):
        f1.write(f"@{names[i]}/1\n{reads1[i]}\n+\n{'I'*len(reads1[i])}\n")
        f2.write(f"@{names[i]}/2\n{reads2[i]}\n+\n{'I'*len(reads2[i])}\n")

#now gzip all .fq
import subprocess
subprocess.run(["gzip", "reads_1.fq"])
subprocess.run(["gzip", "reads_2.fq"])
