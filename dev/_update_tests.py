#!/usr/bin/env python3

from pathlib import Path
import subprocess


assert Path("test_cases").exists(), "Starting from the wrong dir, test_cases not found"

out = """
// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;
"""
for input_toml in sorted(Path("test_cases").rglob("input.toml")):
    case_dir = Path(input_toml.parent)
    if case_dir.name == "actual":
        continue

    name = str(case_dir.relative_to("test_cases"))
    name = "".join(
        c if (c.isascii() and c.isalnum() or c == "_") else "_x_" for c in name
    ).lower()
    case_path = str(case_dir)

    out += f'''
#[test]
fn test_cases_x_{name}() {{
    println!("Test case is in: {case_path}");
    run_test(std::path::Path::new("{case_path}"));
}}
'''

out_path = Path("tests/generated.rs")
out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(out)

subprocess.check_call(["cargo", "fmt", "--", out_path])
print("updated tests")
