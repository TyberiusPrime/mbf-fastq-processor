#!/usr/bin/env python3
import subprocess
from pathlib import Path
import os
import sys
import shutil

args = sys.argv[1:] if len(sys.argv) > 1 else []

test_output_filename = ".test_output.txt"

print("running cargo test")
p = subprocess.Popen(["cargo", "test"], stdout=open(test_output_filename, "wb"))
p.communicate()
print('carco test return code (Non-zero is expected)', p.returncode)

stdout = Path(test_output_filename).read_text()
failures = stdout.rsplit("failures:", 1)[1].strip().splitlines()[1:]
print('detected', len(failures), 'failed test cases')
failures = [x.strip() for x in failures if "_x_" in x]


print("First run cargo test >t.txt")
print("Enter python command. Cwd is test case dir")
print("example: shutil.copy('actual/output.json','output.json')")
print("finish with ctrl-d")
command = sys.stdin.read()
print("accepted python command")

# print("cargo test")
# cargo_test = subprocess.Popen(["cargo", "test"] + args, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
# stdout, stderr = cargo_test.communicate()

start = Path(".").resolve().absolute()

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

print("Now deleting t.txt")
Path(test_output_filename).unlink()
