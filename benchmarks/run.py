import sys
from pathlib import Path
import datetime
import time
import subprocess

import socket
import download_data


hostname = socket.gethostname()

todo = sys.argv[1:]
if not todo:
    todo = list(download_data.parts)


todo = [x for x in todo if x in download_data.parts]
if not todo:
    raise ValueError(f"nothing todo. Available: {list(download_data.parts.keys())}")

log_file = Path("run.log")

subprocess.check_call(["cargo", "build", "--release"])


for part in todo:
    config = f"""
        [input]
            read1 = "{part}"

        [output]
            prefix = "output"
            format = "Raw"
            output_hash = true
        """
    tf = open("input.toml", "w")
    tf.write(config)
    tf.close()
    start = time.time()
    print("starting", part)
    subprocess.check_call(["../target/release/mbf-fastq-processor", "input.toml"])
    end = time.time()
    date = datetime.datetime.now().isoformat()
    msg = (f"{date}\t{hostname}\t{part}\t{end-start}\n")
    print(msg)
    with open(log_file,'a') as op:
        op.write(msg)
    #for fn in Path('.').glob("output_*"):
        #fn.unlink()
