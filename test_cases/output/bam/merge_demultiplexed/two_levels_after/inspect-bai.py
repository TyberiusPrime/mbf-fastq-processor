#!/usr/bin/env python3
import struct
import sys


def read_bai(path):
    with open(path, "rb") as f:
        magic = f.read(4)
        if magic != b"BAI\x01":
            print(f"Invalid magic: {magic!r}", file=sys.stderr)
            sys.exit(1)

        (n_ref,) = struct.unpack("<I", f.read(4))

        print(f"Magic:  BAI\\1")
        print(f"n_ref: {n_ref}")
        print()

        for ref_i in range(n_ref):
            (n_bin,) = struct.unpack("<I", f.read(4))

            print(f"--- Reference {ref_i} ({n_bin} bins) ---")
            print(f"{'bin':>8s}  {'chunk':>6s}  {'chunk_beg':>20s}  {'chunk_end':>20s}")
            print("-" * 62)

            for _ in range(n_bin):
                (bin_id,) = struct.unpack("<I", f.read(4))
                (n_chunk,) = struct.unpack("<I", f.read(4))

                for c in range(n_chunk):
                    chunk_beg, chunk_end = struct.unpack("<QQ", f.read(16))
                    label = f"  [{c}]" if c == 0 else f"  [{c}]"
                    print(f"{bin_id:>8d}{label:>6s}  {chunk_beg:>20d}  {chunk_end:>20d}")

                    if c == 0:
                        bin_id = ""
                        label = ""

            (n_intv,) = struct.unpack("<I", f.read(4))
            print(f"\n  Linear index: {n_intv} intervals")
            print(f"  {'interval':>10s}  {'ioffset':>20s}")
            print(f"  {'-'*34}")

            for i in range(n_intv):
                (ioffset,) = struct.unpack("<Q", f.read(8))
                print(f"  {i:>10d}  {ioffset:>20d}")

            print()

        remaining = f.read(8)
        if remaining and len(remaining) == 8:
            (n_no_coor,) = struct.unpack("<Q", remaining)
            print(f"n_no_coor (unplaced/unmapped): {n_no_coor}")
        elif remaining:
            print(f"Trailing bytes ({len(remaining)}): {remaining.hex()}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <file.bai>", file=sys.stderr)
        sys.exit(1)
    read_bai(sys.argv[1])
