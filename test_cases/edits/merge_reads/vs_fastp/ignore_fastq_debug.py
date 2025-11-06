#!/usr/bin/env python3

import gzip
import sys
from typing import Dict, Tuple, Optional


def reverse_complement(seq: str) -> str:
    """Return reverse complement of DNA sequence"""
    complement = {"A": "T", "T": "A", "G": "C", "C": "G", "N": "N"}
    return "".join(complement.get(base, base) for base in reversed(seq))


def load_fastq(filename: str) -> Dict[str, Tuple[str, str]]:
    """Load FASTQ file into memory as dict {read_name: (sequence, quality)}"""
    reads = {}

    if filename.endswith(".gz"):
        open_func = gzip.open
        mode = "rt"
    else:
        open_func = open
        mode = "r"

    try:
        with open_func(filename, mode) as f:
            while True:
                header = f.readline().strip()
                if not header:
                    break
                if not header.startswith("@"):
                    continue

                seq = f.readline().strip()
                plus = f.readline().strip()
                qual = f.readline().strip()

                # Extract read name (remove @ and everything after first space)
                read_name = header[1:].split()[0]
                reads[read_name] = (seq, qual)

    except FileNotFoundError:
        print(f"Warning: Could not find file {filename}")
        return {}
    except Exception as e:
        print(f"Error loading {filename}: {e}")
        return {}

    return reads


def format_alignment(
    seq1: str, qual1: str, seq2: str, qual2: str, read_name: str, width: int = 80
) -> str:
    """Format alignment with quality scores, wrapped at specified width"""
    result = []
    result.append(f"\nAlignment for read: {read_name}")
    result.append("=" * min(width, len(read_name) + 20))

    for i in range(0, len(seq1), width):
        chunk1_seq = seq1[i : i + width]
        chunk1_qual = qual1[i : i + width]
        chunk2_seq = seq2[i : i + width]
        chunk2_qual = qual2[i : i + width]

        # Create alignment line
        alignment = []
        for j in range(len(chunk1_seq)):
            if j < len(chunk2_seq):
                if chunk1_seq[j] == chunk2_seq[j]:
                    alignment.append("|")
                else:
                    alignment.append(".")
            else:
                alignment.append(" ")

        # Check if qualities differ for this chunk
        qual_diff = chunk1_qual != chunk2_qual

        result.append(f"Pos {i + 1:4d}:")
        if qual_diff:
            result.append(f"  Q1: {chunk1_qual}")
        result.append(f"  S1: {chunk1_seq}")
        result.append(f"  Al: {''.join(alignment)}")
        result.append(f"  S2: {chunk2_seq}")
        if qual_diff:
            result.append(f"  Q2: {chunk2_qual}")
        result.append("")

    return "\n".join(result)


def find_mismatches(
    seq1: str,
    qual1: str,
    seq2: str,
    qual2: str,
    orig_r1: Tuple[str, str],
    orig_r2: Tuple[str, str],
) -> str:
    """Find mismatches and trace back to original reads"""
    result = []
    result.append("\nMismatch analysis:")
    result.append("-" * 40)

    mismatches = []
    for i, (base1, base2) in enumerate(zip(seq1, seq2)):
        if base1 != base2:
            mismatches.append((i, base1, base2, qual1[i], qual2[i]))

    if not mismatches:
        result.append("No mismatches found!")
        return "\n".join(result)

    result.append(f"Found {len(mismatches)} mismatches:")

    for pos, base1, base2, q1, q2 in mismatches:
        result.append(
            f"\nPosition {pos + 1}: {base1}(Q{ord(q1) - 33}) vs {base2}(Q{ord(q2) - 33})"
        )

        # Try to trace back to original reads
        orig_r1_seq, orig_r1_qual = orig_r1
        orig_r2_seq, orig_r2_qual = orig_r2
        orig_r2_rc_seq = reverse_complement(orig_r2_seq)
        orig_r2_rc_qual = orig_r2_qual[::-1]  # Reverse quality for RC


        pos_in_r1 = pos
        offset = orig_r1_seq.rfind(orig_r2_rc_seq[:7])
        if offset == -1:
            offset = orig_r1_seq.rfind(orig_r2_rc_seq[7:14]) -7
            if offset == -1:
                print("could not find start of r2")
                continue
        pos_in_r2 = pos - offset
        print('pos_in_r2', pos_in_r2, 'offset', offset)

        print(pos, "In r1:", orig_r1_seq[pos_in_r1], orig_r1_qual[pos_in_r1], ord(orig_r1_qual[pos_in_r1]))
        print(pos, "in r2:", orig_r2_rc_seq[pos_in_r2], orig_r2_rc_qual[pos_in_r2], ord(orig_r2_rc_qual[pos_in_r2]))
        print("mine", base1, q1, ord(q1))
        print("fastp", base2, q2, ord(q2))
        print("")


    return "\n".join(result)


def main():
    # Load all FASTQ files
    print("Loading FASTQ files...")

    output_reads = load_fastq(f"output_read1.fq.gz")
    print(f"Loaded {len(output_reads)} reads from output_read1.fq")

    fastp_reads = load_fastq("ignore_fastp_output/merged.fastp.gz")
    print(f"Loaded {len(fastp_reads)} reads from ignore_fastp_output/merged.fastp.gz")

    orig_r1_reads = load_fastq("input_ERR777676_1.fastq.gz")
    print(f"Loaded {len(orig_r1_reads)} reads from input_ERR777676_1.fastq.gz")

    orig_r2_reads = load_fastq("input_ERR777676_2.fastq.gz")
    print(f"Loaded {len(orig_r2_reads)} reads from input_ERR777676_2.fastq.gz")

    print("\nReady for debugging!")
    print("Enter read names (or substrings) to search. Press Ctrl+C to exit.")
    print("=" * 60)

    try:
        while True:
            try:
                query = input("\nEnter read name (substring): ").strip()
                if query.startswith("@"):
                    query = query[1:]
                if not query:
                    continue

                # Find matching reads
                matches = []
                for read_name in output_reads.keys():
                    if query in read_name:
                        matches.append(read_name)

                if not matches:
                    print(f"No reads found containing '{query}'")
                    continue

                print(f"Found {len(matches)} matching reads:")
                for i, match in enumerate(matches[:5]):  # Show first 5 matches
                    print(f"  {i + 1}. {match}")

                if len(matches) > 5:
                    print(f"  ... and {len(matches) - 5} more")

                # Process first match
                read_name = matches[0]
                print(f"\nAnalyzing: {read_name}")

                # Get sequences from both files
                if read_name in output_reads and read_name in fastp_reads:
                    seq1, qual1 = output_reads[read_name]
                    seq2, qual2 = fastp_reads[read_name]

                    # Show alignment
                    alignment = format_alignment(seq1, qual1, seq2, qual2, read_name)
                    print(alignment)

                    # Get original reads for mismatch analysis
                    orig_r1 = orig_r1_reads.get(read_name, ("", ""))
                    orig_r2 = orig_r2_reads.get(read_name, ("", ""))

                    if orig_r1[0] and orig_r2[0]:
                        mismatch_analysis = find_mismatches(
                            seq1, qual1, seq2, qual2, orig_r1, orig_r2
                        )
                        print(mismatch_analysis)
                    else:
                        print(
                            "Could not find original R1/R2 reads for mismatch analysis"
                        )

                elif read_name in output_reads:
                    print(f"Read found in output but not in fastp file")
                elif read_name in fastp_reads:
                    print(f"Read found in fastp but not in output file")
                else:
                    print(f"Read not found in either file")

            except EOFError:
                break

    except KeyboardInterrupt:
        print("\nExiting...")


if __name__ == "__main__":
    main()
