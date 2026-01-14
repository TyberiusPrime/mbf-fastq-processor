reads = {
    "complete_overlap_250": (
        "CCTACGGGAGGCAGCAGTGGGGAATATTGCGCAATGGGCGAAAGCCTGACGCAGCGACGCCGCGTGAGGGATGAAGGTCTTCGGATCGTAAACCTCTGTCAGAAGGGAAGAAACTAGGGTGTTCTAATCAGCATCCTACTGACGGTACCTTCAAAGGAAGCACCGGCTAACTCCGTGCCAGCAGCCCCGGTAATACGGAGGGTNCAAGCGTTAATCGGGATCACTGGGCGTAAAGCGCACGTAGGCTGTT",
        "AACAGCCTACGTGCGCTTTACGCCCAGTGATCCCGATTAACGCTTGNACCCTCCGTATTACCGGGGCTGCTGGCACGGAGTTAGCCGGTGCTTCCTTTGAAGGTACCGTCAGTAGGATGCTGATTAGAACACCCTAGTTTCTTCCCTTCTGACAGAGGTTTACGATCCGAAGACCTTCATCCCTCACGCGGCGTCGCTGCGTCAGGCTTTCGCCCATTGCGCAATATTCCCCACTGCTGCCTCCCGTAGG",
    ),
    "50bp_overlap": (
        "CCTACGGGAGGCAGCAGTGGGGAATATTGCGCAATGGGCGAAAGCCTGACGCAGCGACGCCGCGTGAGGGATGAAGGTCTTCGGATCGTAAACCTCTGTC",
        "AACAGCCTACGTGCGCTTTACGCCCAGTGATCCCGATTAACGCTTGNACCTTCAAAGGAAGCACCGGCTAACTCCGTGCCAGCAGCCCCGGTAATACGGAG",
    ),
    "last_i_relevant_first_loop": (
        "AAAAAAAAACAACAAAAAACAAAAAAAAACAAAAAAAAACAAAAAAAAAAC",
        "TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
    ),
    "last_i_relevant_for_2nd_loop": (
        "GCCTACGGGAGGCAGCAGTGGGGAATATTGCGCAATGGGCGAAAGCCTGAT",
        "GTCAGGCTTTCTTTTTTTGCGCAATATTCCCCACTGCTGCCTCCCGTAGGCTTTT",
    ),
}

names = []
reads1 = []
reads2 = []


def left_shift(r, l, fill="A"):
    return r[l:] + fill * l


def right_shift(r, l, fill="A"):
    return fill * l + r[:-l]

import random
random.seed(42)

def random_qualities(count):
    # return count letters between ascii 33 and 126
    return "".join(random.choices([chr(i) for i in range(33, 127)], k=count))



assert left_shift("AGTC", 1) == "GTCA"
assert left_shift("AGTC", 2) == "TCAA"
assert left_shift("AGTC", 3) == "CAAA"
assert left_shift("AGTC", 4) == "AAAA"

assert right_shift("AGTC", 1) == "AAGT"
assert right_shift("AGTC", 2) == "AAAG"
assert right_shift("AGTC", 3) == "AAAA"
assert right_shift("AGTC", 3, "C") == "CCCA"
assert right_shift("AGTC", 4) == "AAAA"


def add(name, read1, read2):
    if not read1 or not read2:
        return  # fastp always *filters* reads where one is empty. We don't so we don't test this

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
    if ri1 != -1 and ri1 == ri2:  # already present
        print("Skipping duplicate", name)
    names.append(name)
    reads1.append(read1)
    reads2.append(read2)


for prefix, (read1, read2) in reads.items():
    for i in range(len(read1)):
        shifted_read2 = ("A" * i) + read2[-i:]
        add(f"X_{prefix}_read2_right_shifted_by_{i}", read1, shifted_read2)

    for i in range(len(read1)):
        shifted_read2 = read2[-i:] + ("A" * i)
        add(f"X_{prefix}_read2_left_shifted_by_{i}", read1, shifted_read2)

    for i in range(len(read1)):
        shifted_read1 = read1[i:] + ("A" * i)
        add(f"X_{prefix}_read1_left_shifted_by_{i}", shifted_read1, read2)

    for i in range(len(read1)):
        shifted_read1 = ("A" * i) + read1[i:]
        add(f"X_{prefix}_read1_right_shifted_by_{i}", shifted_read1, read2)

    for ii in range(len(read1)):
        add(f"_{prefix}_read1_left_shift_{ii}", left_shift(read1, ii), read2)
        add(f"_{prefix}_read1_right_shift_{ii}", right_shift(read1, ii), read2)

    for ii in range(len(read2)):
        add(f"_{prefix}_read2_left_shift_{ii}", read1, left_shift(read2, ii))
        add(f"_{prefix}_read2_right_shift_{ii}", read1, right_shift(read2, ii))

    with open("reads_1.fq", "w") as f1, open("reads_2.fq", "w") as f2:
        for i in range(len(reads1)):
            qual_1 = random_qualities(len(reads1[i]))
            qual_2 = random_qualities(len(reads2[i]))
            f1.write(f"@{names[i]}/1\n{reads1[i]}\n+\n{qual_1}\n")
            f2.write(f"@{names[i]}/2\n{reads2[i]}\n+\n{qual_2}\n")

# now gzip all .fq
import subprocess

subprocess.run(["gzip", "reads_1.fq", "-f"])
subprocess.run(["gzip", "reads_2.fq", "-f"])
