from pathlib import Path
import subprocess

# download or de/recompress


def cut_to_length(input, len):
    def do_cut(output):
        with open(output, "wb") as op:
            with open(input, "rb") as ip:
                lines = iter(ip)
                name = next(lines)
                while name:
                    seq = next(lines)
                    plus = next(lines)
                    qual = next(lines)
                    op.write(name)
                    op.write(seq[:len] + b"\n")
                    op.write(plus)
                    op.write(qual[:len] + b"\n")
                    name = next(lines, None)

    return do_cut


parts = {
    "data/large/ERR12828869_1.fq.gz": "http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR128/069/ERR12828869/ERR12828869_1.fastq.gz",
    "data/large/ERR12828869_2.fq.gz": "http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR128/069/ERR12828869/ERR12828869_2.fastq.gz",
    "data/large/ERR12828869_1.fq": "data/large/ERR12828869_1.fq.gz",
    "data/large/ERR12828869_2.fq": "data/large/ERR12828869_2.fq.gz",
    "data/large/ERR12828869_1.fq.zst": "data/large/ERR12828869_1.fq",
    "data/large/ERR12828869_2.fq.zst": "data/large/ERR12828869_2.fq",
    "data/large/ERR12664935_1.fq.gz": "http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR126/035/ERR12664935/ERR12664935_1.fastq.gz",
    "data/large/ERR12828869_1_pseudo_index.fq": cut_to_length(
        "data/large/ERR12828869_1.fq", 8
    ),
    "data/large/ERR2093901_pacbio.fq.gz": "ftp://ftp.sra.ebi.ac.uk/vol1/fastq/ERR209/002/ERR2093902/ERR2093902_1.fastq.gz",
}


if __name__ == "__main__":
    for fn, url in parts.items():
        fn = Path(fn)
        if not fn.exists():
            if not callable(url) and not url in parts:
                print("downloading", url)
                fn.parent.mkdir(exist_ok=True, parents=True)
                subprocess.check_call(["curl", "-o", fn, url])
            elif callable(url):
                url(fn)
            else:
                if fn.suffix == ".fq":
                    print("decompressing", url)
                    subprocess.check_call(["gzip", "-cd", url], stdout=open(fn, "wb"))
                elif fn.suffix == ".zst":
                    # compress into  zstd
                    subprocess.check_call(["zstd", url, "-o", fn])
                else:
                    raise ValueError(fn)
                # decompress into
