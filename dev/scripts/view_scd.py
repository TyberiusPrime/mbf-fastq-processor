#!/usr/bin/env python3
"""Inspect a StoreSingleCellData .bin file.

Usage:
    view_scd.py <file.bin> [cell_barcodes.txt] [genes.txt] [reads.fq]

Positional arguments:
    file.bin            Required. The .bin file to inspect.
    cell_barcodes.txt   Optional. Lookup table for cell barcode names.
    genes.txt           Optional. Lookup table for gene names.
    reads.fq            Optional. FASTQ file; prints read name next to each row.
                        Reads are matched by position (row 0 = read 0 after sort,
                        which does NOT correspond to original read order).

All file arguments are auto-detected as gzip or zstd by magic bytes and
decompressed transparently regardless of file extension.

With no lookup tables the indices are shown as raw numbers.
"""

import gzip
import io
import struct
import sys
from pathlib import Path

MAGIC = b"FQRSCD\x00\x02"
HEADER_SIZE = 13  # 8 magic + 4 num_rows + 1 umi_length
ROW_SIZE = 12     # 3 × u32 LE

_GZIP_MAGIC = b"\x1f\x8b"
_ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"


def open_maybe_compressed(path):
    """Return a binary file-like object, decompressing gzip/zstd transparently."""
    raw = Path(path).read_bytes()
    if raw[:2] == _GZIP_MAGIC:
        return io.BytesIO(gzip.decompress(raw))
    if raw[:4] == _ZSTD_MAGIC:
        try:
            import zstandard
        except ImportError:
            print(
                f"Error: {path} is zstd-compressed but the 'zstandard' package is not installed.\n"
                "Install it with: pip install zstandard",
                file=sys.stderr,
            )
            sys.exit(1)
        return io.BytesIO(zstandard.ZstdDecompressor().decompress(raw))
    return io.BytesIO(raw)


def read_bytes(path):
    """Read a file, decompressing gzip/zstd transparently."""
    return open_maybe_compressed(path).read()


def read_lookup(path):
    """Return list of names from a lookup .txt file (line 0 = 'unmatched')."""
    return read_bytes(path).decode().splitlines()


def read_fastq_names(path):
    """Return list of read names (without '@') from a FASTQ file."""
    names = []
    fh = io.TextIOWrapper(open_maybe_compressed(path), encoding="utf-8")
    for i, line in enumerate(fh):
        if i % 4 == 0:
            names.append(line.strip().lstrip("@").split()[0])
    return names


def decode_umi(enc, length):
    """Decode a 2-bit encoded UMI u32 back to a DNA string.

    Encoding: base 0 is at bits 2*(length-1)+1 .. 2*(length-1), ..., base n-1 at bits 1-0.
    u32::MAX is the sentinel for 'contains N'.
    """
    if enc == 0xFFFFFFFF:
        return "N..."
    if length == 0:
        return f"0x{enc:08x}"
    bases = "ACGT"
    result = []
    for i in range(length):
        shift = 2 * (length - 1 - i)
        result.append(bases[(enc >> shift) & 3])
    return "".join(result)


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(1)

    bin_path = args[0]
    cell_names = read_lookup(args[1]) if len(args) > 1 else None
    gene_names = read_lookup(args[2]) if len(args) > 2 else None
    read_names = read_fastq_names(args[3]) if len(args) > 3 else None

    data = read_bytes(bin_path)
    if data[:8] != MAGIC:
        print(f"Error: bad magic bytes in {bin_path}", file=sys.stderr)
        sys.exit(1)

    num_rows = struct.unpack_from("<I", data, 8)[0]
    umi_length = data[12]
    print(f"File   : {bin_path}")
    print(f"Rows   : {num_rows}")
    print(f"UMI len: {umi_length} bp")
    if cell_names:
        print(f"Cells  : {len(cell_names) - 1} barcodes (+ unmatched)")
    if gene_names:
        print(f"Genes  : {len(gene_names) - 1} entries (+ unmatched)")
    print()

    # Header row
    col_gene = "gene_idx" if gene_names is None else "gene"
    col_cell = "cell_idx" if cell_names is None else "cell"
    header = f"{'row':>6}  {col_gene:<20}  {col_cell:<20}  {'umi':>{max(10, umi_length)}}"
    if read_names:
        header += "  read_name"
    print(header)
    print("-" * len(header))

    limit = 50
    for i in range(min(num_rows, limit)):
        offset = HEADER_SIZE + i * ROW_SIZE
        gene_idx, cell_idx, umi_enc = struct.unpack_from("<III", data, offset)

        gene_str = (
            gene_names[gene_idx] if gene_names and gene_idx < len(gene_names)
            else str(gene_idx)
        )
        cell_str = (
            cell_names[cell_idx] if cell_names and cell_idx < len(cell_names)
            else str(cell_idx)
        )
        umi_str = decode_umi(umi_enc, umi_length)
        row = f"{i:>6}  {gene_str:<20}  {cell_str:<20}  {umi_str:>{max(10, umi_length)}}"
        if read_names:
            rname = read_names[i] if i < len(read_names) else "?"
            row += f"  {rname}"
        print(row)

    if num_rows > limit:
        print(f"  ... {num_rows - limit} more rows")


if __name__ == "__main__":
    main()
