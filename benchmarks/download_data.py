from pathlib import Path
import subprocess

parts = {
    "data/large/ERR12828869_1.fq.gz": "http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR128/069/ERR12828869/ERR12828869_1.fastq.gz",
    "data/large/ERR12828869_2.fq.gz": "http://ftp.sra.ebi.ac.uk/vol1/fastq/ERR128/069/ERR12828869/ERR12828869_2.fastq.gz",
    "data/large/ERR12828869_1.fq": "data/large/ERR12828869_1.fastq.gz",
    "data/large/ERR12828869_2.fq": "data/large/ERR12828869_2.fastq.gz",

    "data/large/ERR12828869_1.fq.zst": "data/large/ERR12828869_1.fq",
    "data/large/ERR12828869_2.fq.zst": "data/large/ERR12828869_2.fq",
}

for fn, url in parts.items():
    fn = Path(fn)
    if not fn.exists():
        if not url in parts:
            print("downloading", url)
            subprocess.check_call(["curl", "-o", fn, url])
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
