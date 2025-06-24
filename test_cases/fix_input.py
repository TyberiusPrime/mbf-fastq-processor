from pathlib import Path
import tomllib


for p in Path(".").rglob("input.toml"):
    if p.is_file():
        r = p.read_text()
        r = r.replace("read1 = 'read1.fq", "read1 = 'input_read1.fq")
        r = r.replace("read2 = 'read2.fq", "read2 = 'input_read2.fq")
        r = r.replace("index1 = 'index1.fq", "index1 = 'input_index1.fq")
        r = r.replace("index2 = 'index2.fq", "index2 = 'input_index2.fq")
        p.write_text(r)
