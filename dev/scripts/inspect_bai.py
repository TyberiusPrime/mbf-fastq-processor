#!/usr/bin/env python3
"""Inspect a BAM index (.bai) file and print a human-readable table."""

import struct
import sys

BAI_MAGIC = b"BAI\1"


def decode_virtual_offset(voi):
    """Decode a BGZF virtual file offset into (coffset, uoffset).

    Formula: coffset << 16 | uoffset
    coffset = file offset to the beginning of the BGZF block
    uoffset = offset within the uncompressed data of that block
    """
    coffset = voi >> 16
    uoffset = voi & 0xFFFF
    return coffset, uoffset


def print_table(title, rows, headers):
    """Print rows as a formatted ASCII table."""
    print(f"\n{title}")
    print("-" * (sum(len(h) + 2 for h in headers) + len(headers) - 1))
    header_line = "  ".join(f"{h:>14}" for h in headers)
    print(header_line)
    for row in rows:
        print("  ".join(f"{v:>14}" for v in row))


def inspect_bai(filepath):
    with open(filepath, "rb") as f:
        data = f.read()

    buf = memoryview(data)
    off = 0

    # Magic
    magic = buf[off : off + 4]
    off += 4
    if magic != BAI_MAGIC:
        print(f"ERROR: Expected magic 'BAI\\1', got {magic!r}", file=sys.stderr)
        sys.exit(1)
    print(f"Magic   : BAI\\1 (OK)")

    # n_ref
    n_ref = struct.unpack_from("<I", buf, off)[0]
    off += 4
    print(f"n_ref   : {n_ref}")

    refs = []
    for ri in range(n_ref):
        n_bin = struct.unpack_from("<I", buf, off)[0]
        off += 4

        bins_rows = []
        for bi in range(n_bin):
            bin_idx = struct.unpack_from("<I", buf, off)[0]
            off += 4
            n_chunk = struct.unpack_from("<I", buf, off)[0]
            off += 4

            for ci in range(n_chunk):
                chunk_beg = struct.unpack_from("<Q", buf, off)[0]
                off += 8
                chunk_end = struct.unpack_from("<Q", buf, off)[0]
                off += 8

                cb_coff, cb_uoff = decode_virtual_offset(chunk_beg)
                ce_coff, ce_uoff = decode_virtual_offset(chunk_end)
                bins_rows.append(
                    (
                        str(bin_idx),
                        str(n_chunk),
                        f"{cb_coff:#010x}+{cb_uoff}",
                        f"{ce_coff:#010x}+{ce_uoff}",
                    )
                )

        # n_intv
        n_intv = struct.unpack_from("<I", buf, off)[0]
        off += 4
        print(f"\n  Ref #{ri}: n_bin={n_bin}, n_intv={n_intv}")

        intv_rows = []
        for ii in range(n_intv):
            ioffset = struct.unpack_from("<Q", buf, off)[0]
            off += 8
            i_coff, i_uoff = decode_virtual_offset(ioffset)
            intv_rows.append(
                (
                    str(ii),
                    str(n_intv),
                    f"{i_coff:#010x}+{i_uoff}",
                )
            )

        refs.append(
            {
                "ref": ri,
                "n_bin": n_bin,
                "n_intv": n_intv,
                "bins": bins_rows,
                "intv": intv_rows,
            }
        )

    # Optional: n_no_coor
    n_no_coor = None
    if off + 8 <= len(buf):
        n_no_coor = struct.unpack_from("<Q", buf, off)[0]
        off += 8
        print(f"\nn_no_coor       : {n_no_coor}")

    # Print tables per reference
    for r in refs:
        print_table(
            f"BINS — Ref #{r['ref']} ({r['n_bin']} bins)",
            r["bins"][:15] or [("—",) * 4],
            ["Bin #", "Chunks", "Chunk Start", "Chunk End"],
        )
        if len(r["bins"]) > 15:
            print(f"  ... ({len(r['bins'])} bins total, showing first 15)")

        print_table(
            f"INTERVALS — Ref #{r['ref']} ({r['n_intv']} intervals)",
            r["intv"][:15] or [("—",) * 3],
            ["Interval #", "Total Intvs", "Offset"],
        )
        if len(r["intv"]) > 15:
            print(f"  ... ({len(r['intv'])} intervals total, showing first 15)")

    if n_no_coor is not None:
        print(f"\nUnplaced unmapped reads (RNAME *): {n_no_coor}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <index.bai>", file=sys.stderr)
        sys.exit(1)
    inspect_bai(sys.argv[1])
