# verify that the by-bam filter do what tehy're supposed to
import pysam
from pathlib import Path
import subprocess


input = pysam.Samfile(
    "remove_bam_both/input_ERR12828869_250_aligned_250_unaligned.bam", "rb"
)
mapped_names = set()
unmapped_names = set()
mapped_seqs = set()
unmapped_seqs = set()
for read in input.fetch(until_eof=True):
    if read.is_unmapped:
        unmapped_names.add(read.query_name)
        unmapped_seqs.add(read.query_sequence)
    else:
        mapped_names.add(read.query_name)
        mapped_seqs.add(read.query_sequence)

assert mapped_names.isdisjoint(unmapped_names)

zstd_decoded = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_both/input_read1.fq.zst").resolve()), "-c"]
).decode("utf-8")

input_seqs = set()
seen_mapped = False
seen_unmapped = False
next_is_seq = False
for line in zstd_decoded.splitlines():
    if line.startswith("@"):
        next_is_seq = True
    elif next_is_seq:
        next_is_seq = False
        seq = line
        input_seqs.add(seq)
        if seq in mapped_seqs:
            seen_mapped = True
        elif seq in unmapped_seqs:
            seen_unmapped = True

assert seen_mapped
assert seen_unmapped


both = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_both/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8").split("\n")
next_is_seq = False
for line in both:
    if line.startswith("@"):
        next_is_seq = True
    elif next_is_seq:
        next_is_seq = False
        seq = line
        assert seq and seq in input_seqs
        assert seq not in mapped_seqs
        assert seq not in unmapped_seqs


mapped = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_mapped_only/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8").split("\n")
next_is_seq = False
any_unmapped = False
for line in mapped:
    if line.startswith("@"):
        next_is_seq = True
    elif next_is_seq:
        next_is_seq = False
        seq = line
        assert seq and seq in input_seqs
        assert seq not in mapped_seqs
        if seq in unmapped_seqs:
            any_unmapped = True
assert any_unmapped

unmapped = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_unmapped_only/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8").split("\n")
next_is_seq = False
any_mapped = False
for line in unmapped:
    if line.startswith("@"):
        next_is_seq = True
    elif next_is_seq:
        next_is_seq = False
        seq = line
        assert seq and seq in input_seqs
        assert seq not in unmapped_seqs
        if seq in mapped_seqs:
            any_mapped = True
assert any_mapped
