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

seen_mapped = False
seen_unmapped = False
for line in zstd_decoded.splitlines():
    if line.startswith("@"):
        name = line[1:].split(" ")[0]
        if name in mapped_names:
            seen_mapped = True
        elif name in unmapped_names:
            seen_unmapped = True

assert seen_mapped
assert seen_unmapped


both = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_both/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8")
for line in both:
    if line.startswith("@"):
        name = line[1:].split(" ")[0]
        assert name not in mapped_names
        assert name not in unmapped_names

unmapped_only = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_unmapped_only/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8").split("\n")
any_mapped = False
for line in unmapped_only:
    if line.startswith("@"):
        name = line[1:].split(" ")[0]
        assert name not in unmapped_names
        if name in mapped_names:
            any_mapped = True
assert any_mapped # make sure teh mapped ones passed

mapped_only = subprocess.check_output(
    ["zstd", "-d", str(Path("remove_bam_mapped_only/output_read1.fq.zst").resolve()), "-c"]
).decode("utf-8").split("\n")
any_unmapped = False
for line in mapped_only:
    if line.startswith("@"):
        name = line[1:].split(" ")[0]
        assert name not in mapped_names
        if name in unmapped_names:
            any_unmapped = True
assert any_unmapped # make sure the unmapped ones passed.
