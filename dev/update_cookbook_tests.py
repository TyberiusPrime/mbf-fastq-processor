#!/usr/bin/env python3
"""
Generate cookbook tests from the cookbooks directory.
Similar to update_tests.py but for cookbooks.
"""

from pathlib import Path
import subprocess


assert Path("cookbooks").exists(), "Starting from the wrong dir, cookbooks not found"

out = """
// This file is written by dev/update_cookbook_tests.py
// Cookbook tests verify that example cookbooks run successfully.
// They are run separately from regular tests (only in CI/nix builds).

#[cfg(feature = "cookbook_tests")]
mod test_runner;

#[cfg(feature = "cookbook_tests")]
use test_runner::run_test;
"""

for input_toml in sorted(Path("cookbooks").rglob("input.toml")):
    cookbook_dir = Path(input_toml.parent)

    # Skip if this is in a subdirectory like input/ or reference_output/
    if cookbook_dir.name in ["input", "reference_output", "actual"]:
        continue

    name = cookbook_dir.name.replace("-", "_")
    cookbook_path = str(cookbook_dir)

    out += f'''
#[test]
#[cfg(feature = "cookbook_tests")]
fn cookbook_{name}() {{
    println!("Testing cookbook: {cookbook_path}");
    run_test(std::path::Path::new("{cookbook_path}"));
}}
'''

out_path = Path("tests/cookbook_tests.rs")
out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(out)

subprocess.check_call(["cargo", "fmt", "--", str(out_path)])
print(f"Generated cookbook tests in {out_path}")
