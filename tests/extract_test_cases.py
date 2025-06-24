#!/usr/bin/env python3

import os
import re
import shutil
import textwrap
from pathlib import Path

def extract_test_functions(file_path):
    """Extract test functions from a Rust file."""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Find all test functions
    pattern = r'#\[test\]\s*fn\s+([a-zA-Z_0-9]+)\s*\(\)\s*\{(.*?)^\}'
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
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()
    
    # Try the run_and_capture pattern
    pattern = r'let\s+\(td,\s*.*?\)\s*=\s*run_and_capture\s*\(\s*"(.*?)"\s*\)'
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()
    
    return None

def parse_input_files(config_str):
    """Parse input files from TOML config string."""
    input_files = {}
    
    # Match patterns like: read1 = 'sample_data/file.fq'
    single_file_pattern = r'(\w+)\s*=\s*\'([^\']+)\''
    for match in re.finditer(single_file_pattern, config_str):
        key = match.group(1)
        if key in ['read1', 'read2', 'index1', 'index2']:
            input_files[key] = [match.group(2)]
    
    # Match patterns like: read1 = ['sample_data/file1.fq', 'sample_data/file2.fq']
    array_file_pattern = r'(\w+)\s*=\s*\[([^\]]+)\]'
    for match in re.finditer(array_file_pattern, config_str):
        key = match.group(1)
        if key in ['read1', 'read2', 'index1', 'index2']:
            files = []
            file_matches = re.finditer(r'\'([^\']+)\'', match.group(2))
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
            extension = '.fq'
            if filename.endswith('.gz'):
                extension = '.fq.gz'
            elif filename.endswith('.zst'):
                extension = '.fq.zst'
            
            new_files.append(f"{key}{extension}")
        
        # Replace file paths in config
        if len(files) == 1:
            pattern = f"{key}\\s*=\\s*'[^']+'".replace('0','')
            replacement = f"{key} = '{new_files[0]}'"
            updated_config = re.sub(pattern, replacement, updated_config)
        else:
            files_str = "', '".join(new_files)
            pattern = f"{key}\\s*=\\s*\\[[^\\]]+\\]".replace('0','')
            replacement = f"{key} = ['{files_str}']"
            updated_config = re.sub(pattern, replacement, updated_config)
    
    return updated_config

def check_for_should_panic(test_body):
    """Check if test is expected to panic."""
    should_panic = re.search(r'#\[should_panic', test_body)
    if should_panic:
        panic_msg = re.search(r'expected\s*=\s*"([^"]+)"', test_body)
        if panic_msg:
            return panic_msg.group(1)
        return True
    return False

def copy_files(input_files, test_name):
    """Copy input files to the test case directory."""
    test_dir = Path("test_cases") / test_name
    os.makedirs(test_dir, exist_ok=True)
    
    # Copy input files
    for key, files in input_files.items():
        for i, file in enumerate(files):
            src = file
            filename = os.path.basename(file)
            extension = '.fq'
            if filename.endswith('.gz'):
                extension = '.fq.gz'
            elif filename.endswith('.zst'):
                extension = '.fq.zst'
                
            dst = test_dir / f"{key}{extension}"
            
            try:
                shutil.copy(src, dst)
                print(f"Got it! Copied {src} to {dst}")
            except Exception as e:
                print(f"Oops! Trouble copying {src}: {e}")

def extract_assert_file_content(test_body):
    """Extract file paths and expected content from assertions."""
    output_assertions = []
    
    # Look for file content assertions
    patterns = [
        r'let\s+should\s*=\s*std::fs::read_to_string\("([^"]+)"\).*?assert_eq!\(should,\s*actual\)',
        r'let\s+actual\s*=\s*std::fs::read_to_string\([^)]+\).*?assert_eq!\(should,\s*actual\)',
        r'let\s+actual\s*=\s*std::fs::read_to_string\([^)]+\).*?compare_fastq\(&actual,\s*should\)'
    ]
    
    for pattern in patterns:
        for match in re.finditer(pattern, test_body, re.DOTALL):
            output_file = test_body[match.start():match.end()]
            output_assertions.append(output_file)
    
    return output_assertions

def process_tests(file_path):
    """Process all test functions in the file."""
    test_functions = extract_test_functions(file_path)
    print(f"Hey! Found {len(test_functions)} test functions")
    
    for name, body in test_functions.items():
        print(f"Working on test: {name}")
        
        config_str = extract_toml_config(body)
        if not config_str:
            print(f"  Hmm, can't find any TOML config in {name}")
            continue
        
        input_files = parse_input_files(config_str)
        
        if not input_files:
            print(f"  No input files in {name}, skipping this one")
            continue
        
        test_dir = Path("test_cases") / name
        os.makedirs(test_dir, exist_ok=True)
        
        # Check if the test should panic
        panic_msg = check_for_should_panic(body)
        
        if panic_msg:
            # Create expected_panic.txt file
            with open(test_dir / "expected_panic.txt", 'w') as f:
                if isinstance(panic_msg, str):
                    f.write(panic_msg)
                else:
                    f.write("Test is expected to panic")
            print(f"  Added expected_panic.txt for {name}")
        
        updated_config = update_toml_config(config_str, input_files)
        
        with open(test_dir / "input.toml", 'w') as f:
            f.write(updated_config)
        
        copy_files(input_files, name)
        
        print(f"  All set! Created test directory: {test_dir}")
        
        # Output file extraction is a bit more complex due to the wide variety of assertion patterns
        assertions = extract_assert_file_content(body)
        if assertions:
            print(f"  Spotted {len(assertions)} output assertions we'll need to handle manually")
            # Here we would extract and copy expected output files
            # but that requires more complex logic specific to your test structure

if __name__ == "__main__":
    process_tests("tests/integration_tests.rs")
