#!/usr/bin/env python3
"""
Script to set up test cases from reject FastQ files.
Creates folders, moves FastQ files, creates input.toml files, and runs tests.
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path


def main():
    reject_dir = Path("test_cases/input_validation/cock_et_all_testdata/reject")

    if not reject_dir.exists():
        print(f"Error: Directory {reject_dir} does not exist")
        sys.exit(1)

    # Find all .fastq files
    fastq_files = list(reject_dir.glob("*.fastq"))

    if not fastq_files:
        print("No .fastq files found in reject directory")
        sys.exit(1)

    print(f"Found {len(fastq_files)} FastQ files to process")

    zero_exit_codes = []

    for fastq_file in fastq_files:
        # Create folder name (fastq name without .fastq extension)
        folder_name = fastq_file.stem
        test_folder = reject_dir / folder_name

        print(f"Processing {fastq_file.name} -> {folder_name}/")

        # Create the test folder
        test_folder.mkdir(exist_ok=True)

        # Move fastq file to the folder
        new_fastq_path = test_folder / fastq_file.name
        if not new_fastq_path.exists():
            shutil.move(str(fastq_file), str(new_fastq_path))

        # Create input.toml
        toml_content = f"""[input]
    error_read = "{fastq_file.name}"
"""

        toml_path = test_folder / "input.toml"
        with open(toml_path, "w") as f:
            f.write(toml_content)

        # Run cargo and capture output
        print(f"  Running cargo run -- process input.toml in {folder_name}/")

        try:
            # Change to the test folder directory
            result = subprocess.run(
                ["cargo", "run", "--", "process", "input.toml"],
                cwd=test_folder,
                capture_output=True,
                text=True,
                timeout=30,
            )

            # Write output to expected_panic.txt
            expected_panic_path = test_folder / "expected_panic.txt"
            with open(expected_panic_path, "w") as f:
                f.write(f"{result.stderr}")

            if result.returncode == 0:
                zero_exit_codes.append(folder_name)
                print(
                    f"  ⚠️  WARNING: {folder_name} returned exit code 0 (expected non-zero)"
                )
            else:
                print(
                    f"  ✓ {folder_name} returned exit code {result.returncode} (as expected)"
                )

        except subprocess.TimeoutExpired:
            print(f"  ⚠️  TIMEOUT: {folder_name} timed out after 30 seconds")
            expected_panic_path = test_folder / "expected_panic.txt"
            with open(expected_panic_path, "w") as f:
                f.write("Exit code: TIMEOUT\n")
                f.write("Process timed out after 30 seconds\n")
        except Exception as e:
            print(f"  ❌ ERROR: {folder_name} failed with exception: {e}")
            expected_panic_path = test_folder / "expected_panic.txt"
            with open(expected_panic_path, "w") as f:
                f.write(f"Exit code: ERROR\n")
                f.write(f"Exception: {e}\n")

    # Summary
    print(f"\nSummary:")
    print(f"Processed {len(fastq_files)} FastQ files")

    if zero_exit_codes:
        print(f"\n⚠️  Files that returned exit code 0 (unexpected):")
        for file in zero_exit_codes:
            print(f"  - {file}")
        print(f"\nTotal files with zero exit code: {len(zero_exit_codes)}")
    else:
        print("✓ All files returned non-zero exit codes as expected")


if __name__ == "__main__":
    main()
