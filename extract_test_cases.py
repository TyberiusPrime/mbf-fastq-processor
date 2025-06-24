#!/usr/bin/env python3

import os
import re
import shutil
import textwrap
from pathlib import Path


def extract_test_functions(file_path):
    """Extract test functions from a Rust file."""
    with open(file_path, "r") as f:
        content = f.read()

    # Find all test functions
    pattern = r"#\[test\]\s*fn\s+([a-zA-Z_0-9]+)\s*\(\)\s*\{(.*?)^\}"
    matches = re.finditer(pattern, content, re.DOTALL | re.MULTILINE)

    test_functions = {}
    for match in matches:
        name = match.group(1)
        body = match.group(2)
        test_functions[name] = body

    return test_functions


def extract_toml_config(test_body):
    """Extract TOML configuration from test body."""
    # Find the TOML configuration in triple quotes
    pattern = r'let\s+td\s*=\s*run\s*\(\s*"(.*?)"\s*\)'
    pattern = r'run\s*\(\s*"(.*?)"\s*\)'
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()

    # Try the run_and_capture pattern
    pattern = r'let\s+\(td,\s*.*?\)\s*=\s*run_and_capture\s*\(\s*"(.*?)"\s*\)'
    pattern = r'run_and_capture\s*\(\s*"(.*?)"\s*\)'
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()
    pattern = r'run_and_capture_failure\s*\(\s*"(.*?)"\s*\)'
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()


    return None


def parse_input_files(config_str):
    """Parse input files from TOML config string."""
    input_files = {}

    # Match patterns like: read1 = 'sample_data/file.fq'
    single_file_pattern = r"(\w+)\s*=\s*\'([^\']+)\'"
    for match in re.finditer(single_file_pattern, config_str):
        key = match.group(1)
        if key in ["read1", "read2", "index1", "index2"]:
            input_files[key] = [match.group(2)]

    # Match patterns like: read1 = ['sample_data/file1.fq', 'sample_data/file2.fq']
    array_file_pattern = r"(\w+)\s*=\s*\[([^\]]+)\]"
    for match in re.finditer(array_file_pattern, config_str):
        key = match.group(1)
        if key in ["read1", "read2", "index1", "index2"]:
            files = []
            file_matches = re.finditer(r"\'([^\']+)\'", match.group(2))
            for file_match in file_matches:
                files.append(file_match.group(1))
            input_files[key] = files

    return input_files


def update_toml_config(config_str, input_files):
    """Update TOML config to use the new file paths."""
    updated_config = config_str

    for key, files in input_files.items():
        new_files = []
        for i, file in enumerate(files):
            filename = os.path.basename(file)
            extension = ".fq"
            if filename.endswith(".gz"):
                extension = ".fq.gz"
            elif filename.endswith(".zst"):
                extension = ".fq.zst"

            new_files.append(f"{key}{extension}")

        # Replace file paths in config
        if len(files) == 1:
            pattern = f"{key}\\s*=\\s*'[^']+'".replace("0", "")
            replacement = f"{key} = '{new_files[0]}'"
            updated_config = re.sub(pattern, replacement, updated_config)
        else:
            files_str = "', '".join(new_files)
            pattern = f"{key}\\s*=\\s*\\[[^\\]]+\\]".replace("0", "")
            replacement = f"{key} = ['{files_str}']"
            updated_config = re.sub(pattern, replacement, updated_config)

    return updated_config


def check_for_should_panic(test_body):
    """Check if test is expected to panic."""
    should_panic = re.search(r"#\[should_panic", test_body)
    if should_panic:
        panic_msg = re.search(r'expected\s*=\s*"([^"]+)"', test_body)
        if panic_msg:
            return panic_msg.group(1)
        return True
    return False

def decide_output_name(path):
    if '_i1' in path:
        return 'index1.fq'
    elif '_i2' in path:
        return 'index2.fq'
    elif '_1' in path:
        return 'read1.fq'
    elif '_2' in path:
        return 'read2.fq'
    else:
        raise ValueError(f"Cannot determine output name for {path}")

def identify_expected_outputs(test_body):
    """Identify expected outputs from test assertions."""
    expected_outputs = {}

    # Look for file content assertions
    read_to_string_pattern = r'let\s+should\s*=\s*std::fs::read_to_string\("([^"]+)"\)'
    for match in re.finditer(read_to_string_pattern, test_body):
        file_path = match.group(1)
        file_name = os.path.basename(file_path)
        try:
            expected_outputs[decide_output_name(file_name)] = file_path
        except ValueError as e:
            print("could not decide on output name for ", file_path, ":", e)

    # Look for JSON file content
    json_pattern = (
        r'let\s+(?:json_)?should\s*=\s*std::fs::read_to_string\("([^"]+\.json)"\)'
    )
    for match in re.finditer(json_pattern, test_body):
        file_path = match.group(1)
        file_name = os.path.basename(file_path)
        expected_outputs[file_name] = file_path

    return expected_outputs


def copy_files(input_files, test_dir):
    """Copy input files to the test case directory."""
    os.makedirs(test_dir, exist_ok=True)

    # Copy input files
    for key, files in input_files.items():
        for i, file in enumerate(files):
            src = file
            filename = os.path.basename(file)
            extension = ".fq"
            if filename.endswith(".gz"):
                extension = ".fq.gz"
            elif filename.endswith(".zst"):
                extension = ".fq.zst"

            dst = test_dir / f"{key}{extension}"

            try:
                shutil.copy(src, dst)
                print(f"Got it! Copied {src} to {dst}")
            except Exception as e:
                print(f"Oops! Trouble copying {src}: {e}")




def process_tests(file_path):
    """Process all test functions in the file."""
    top_level = Path(file_path).name.replace(".rs", "")
    test_functions = extract_test_functions(file_path)
    print(f"Hey! Found {len(test_functions)} test functions")

    for name, body in test_functions.items():
        print(f"Working on test: {name}")

        config_str = extract_toml_config(body)
        if not config_str:
            print(f"  Hmm, can't find any TOML config in {name}")
            continue

        input_files = parse_input_files(config_str)

        # if not input_files:
        #     print(f"  No input files in {name}, skipping this one")
        #     continue

        test_dir = Path("test_cases") / top_level / name
        os.makedirs(test_dir, exist_ok=True)

        # Check if the test should panic
        panic_msg = check_for_should_panic(body)

        if panic_msg:
            # Create expected_panic.txt file
            with open(test_dir / "expected_panic.txt", "w") as f:
                if isinstance(panic_msg, str):
                    f.write(panic_msg)
                else:
                    f.write("Test is expected to panic")
            print(f"  Added expected_panic.txt for {name}")

        updated_config = update_toml_config(config_str, input_files)

        with open(test_dir / "input.toml", "w") as f:
            f.write(updated_config)

        if input_files:
            copy_files(input_files, test_dir)

        print(f"  All set! Created test directory: {test_dir}")

        # Output file extraction is a bit more complex due to the wide variety of assertion patterns
        output_files= identify_expected_outputs(body)
        # if assertions:
        #     print(
        #         f"  Spotted {len(assertions)} output assertions we'll need to handle manually"
        #     )
            # Here we would extract and copy expected output files
            # but that requires more complex logic specific to your test structure


if __name__ == "__main__":
    import sys
    for fn in sys.argv[1:]:
        print(fn)
        process_tests(fn)
