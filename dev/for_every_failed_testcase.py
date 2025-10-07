#!/usr/bin/env python3
import subprocess
from pathlib import Path
import os
import sys
import shutil

args = sys.argv[1:] if len(sys.argv) > 1 else []

print("Enter python command. Cwd is test case dir")
command  = sys.stdin.read()
print("accepted python command")

# print("cargo test")
# cargo_test = subprocess.Popen(["cargo", "test"] + args, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
# stdout, stderr = cargo_test.communicate()

stdout = Path('t.txt').read_text()
failures = stdout.rsplit("failures:",1)[1].strip().splitlines()[1:]
print(failures)
failures = [x.strip() for x in failures if '_x_' in x]
print(failures)

start = Path('.').resolve().absolute()

for failure in failures:
    if failure.strip():
        path = Path(failure.replace("_x_", "/").strip())
        print(path)
        assert path.exists(), f"Test case path '{path}' does not exist"
        os.chdir(path)
        eval(command)
        os.chdir(start)
    else:
        break
