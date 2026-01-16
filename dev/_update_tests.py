#!/usr/bin/env python3

from pathlib import Path
import subprocess
import collections


assert Path("test_cases").exists(), "Starting from the wrong dir, test_cases not found"

out = """
// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;
"""
counts = collections.Counter()
for test_path in ["test_cases", "cookbooks"]:
    for input_toml in sorted(Path(test_path).rglob("input*.toml")):
        case_dir = Path(input_toml.parent)
        if case_dir.name == "actual":
            continue

        name = str(case_dir.relative_to(test_path))
        name = "".join(
            c if (c.isascii() and c.isalnum() or c == "_") else "_x_" for c in name
        ).lower()
        case_path = str(case_dir)
        counts[input_toml.parent] += 1
        count = counts[input_toml.parent]

        out += f"""
    #[test]
    fn test_cases_x_{name}_{count}() {{
        println!("Test case is in: {case_path}");
        run_test(std::path::Path::new("../{case_path}"), "{input_toml.name}");
    }}
    """

out_path = Path("mbf-fastq-processor/tests/generated.rs")
out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(out)

subprocess.check_call(["cargo", "fmt", "--", out_path])
print("updated tests")
