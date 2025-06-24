#!/usr/bin/env python3

import os
import re
import shutil
import json
from pathlib import Path

def extract_test_functions(file_path):
    """Extract test functions from a Rust file."""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Find all test functions
    pattern = r'#\[test\]\s*(?:#\[should_panic[^\]]*\]\s*)?fn\s+([a-zA-Z_0-9]+)\s*\(\)\s*\{(.*?)^\}'
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
    
    # Try format pattern
    pattern = r'let\s+td\s*=\s*run\(\&format!\(\s*"(.*?)"\s*,'
    match = re.search(pattern, test_body, re.DOTALL)
    if match:
        return match.group(1).strip()
    
    return None

def parse_input_files(config_str):
    """Parse input files from TOML config string."""
    # Simple parsing to get input files
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
    
    # Match filename = 'path' for FilterOtherFile
    other_file_pattern = r'filename\s*=\s*\'([^\']+)\''
    for match in re.finditer(other_file_pattern, config_str):
        key = 'other_file'
        if key not in input_files:
            input_files[key] = []
        input_files[key].append(match.group(1))
    
    return input_files

def identify_expected_outputs(test_body):
    """Identify expected outputs from test assertions."""
    expected_outputs = {}
    
    # Look for file content assertions
    read_to_string_pattern = r'let\s+should\s*=\s*std::fs::read_to_string\("([^"]+)"\)'
    for match in re.finditer(read_to_string_pattern, test_body):
        file_path = match.group(1)
        file_name = os.path.basename(file_path)
        expected_outputs[file_name] = file_path
    
    # Look for JSON file content
    json_pattern = r'let\s+(?:json_)?should\s*=\s*std::fs::read_to_string\("([^"]+\.json)"\)'
    for match in re.finditer(json_pattern, test_body):
        file_path = match.group(1)
        file_name = os.path.basename(file_path)
        expected_outputs[file_name] = file_path
    
    return expected_outputs

def get_test_metadata(test_body, test_name):
    """Extract metadata about the test."""
    metadata = {
        "name": test_name,
        "should_panic": False,
    }
    
    # Check if test is expected to panic
    panic_pattern = r'#\[should_panic\(expected\s*=\s*"([^"]+)"\)\]'
    panic_match = re.search(panic_pattern, test_body)
    if panic_match:
        metadata["should_panic"] = True
        metadata["panic_message"] = panic_match.group(1)
    
    return metadata

def update_toml_config(config_str, input_files, test_name):
    """Update TOML config to use the new file paths."""
    updated_config = config_str
    
    # Replace input file paths
    for key, files in input_files.items():
        if key not in ['read1', 'read2', 'index1', 'index2', 'other_file']:
            continue
            
        for i, file in enumerate(files):
            orig_file = f"'{file}'"
            basename = os.path.basename(file)
            prefix = "input_" if key != 'other_file' else "other_"
            new_file = f"'{prefix}{key}_{i}_{basename}'"
            updated_config = updated_config.replace(orig_file, new_file)
    
    return updated_config

def copy_files(input_files, expected_outputs, test_dir):
    """Copy input and expected output files to the test directory."""
    copied_files = {
        "inputs": [],
        "outputs": []
    }
    
    # Create directories
    os.makedirs(os.path.join(test_dir, "inputs"), exist_ok=True)
    os.makedirs(os.path.join(test_dir, "expected"), exist_ok=True)
    
    # Copy input files
    for key, files in input_files.items():
        for i, file in enumerate(files):
            if not os.path.exists(file):
                copied_files["inputs"].append({
                    "status": "error",
                    "source": file,
                    "error": "File not found"
                })
                continue
                
            basename = os.path.basename(file)
            prefix = "input_" if key != 'other_file' else "other_"
            dest = os.path.join(test_dir, "inputs", f"{prefix}{key}_{i}_{basename}")
            
            try:
                shutil.copy2(file, dest)
                copied_files["inputs"].append({
                    "status": "copied",
                    "source": file,
                    "destination": dest
                })
            except Exception as e:
                copied_files["inputs"].append({
                    "status": "error",
                    "source": file,
                    "error": str(e)
                })
    
    # Copy expected output files
    for name, file in expected_outputs.items():
        if not os.path.exists(file):
            copied_files["outputs"].append({
                "status": "error",
                "source": file,
                "error": "File not found"
            })
            continue
            
        dest = os.path.join(test_dir, "expected", name)
        
        try:
            shutil.copy2(file, dest)
            copied_files["outputs"].append({
                "status": "copied",
                "source": file,
                "destination": dest
            })
        except Exception as e:
            copied_files["outputs"].append({
                "status": "error",
                "source": file,
                "error": str(e)
            })
    
    return copied_files

def main():
    output_dir = "test_cases"
    os.makedirs(output_dir, exist_ok=True)
    
    test_functions = extract_test_functions("tests/integration_tests.rs")
    print(f"Found {len(test_functions)} test functions")
    
    for test_name, test_body in test_functions.items():
        print(f"Processing test: {test_name}")
        test_dir = os.path.join(output_dir, test_name)
        os.makedirs(test_dir, exist_ok=True)
        
        # Extract test configuration
        toml_config = extract_toml_config(test_body)
        if not toml_config:
            print(f"  Could not extract TOML config for {test_name}")
            continue
        
        # Parse input files
        input_files = parse_input_files(toml_config)
        if not input_files:
            print(f"  No input files found for {test_name}")
        
        # Identify expected outputs
        expected_outputs = identify_expected_outputs(test_body)
        
        # Get test metadata
        metadata = get_test_metadata(test_body, test_name)
        
        # Update TOML config
        updated_config = update_toml_config(toml_config, input_files, test_name)
        
        # Write updated config
        with open(os.path.join(test_dir, "config.toml"), "w") as f:
            f.write(updated_config)
        
        # Copy files
        copy_result = copy_files(input_files, expected_outputs, test_dir)
        
        # Write test metadata
        metadata["input_files"] = input_files
        metadata["expected_outputs"] = expected_outputs
        metadata["copy_results"] = copy_result
        
        with open(os.path.join(test_dir, "metadata.json"), "w") as f:
            json.dump(metadata, f, indent=2)
        
        print(f"  Created test case in {test_dir}")

if __name__ == "__main__":
    main()
