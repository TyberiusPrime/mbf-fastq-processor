#!/usr/bin/env python3

import os
import glob
import re
from pathlib import Path

def find_and_rename_output_files():
    """
    Find files matching output*_1.fq.* that don't end in read1.fq.*
    and rename them to the read1 variant.
    """
    # Pattern to match output*_1.fq.* files
    pattern = "output*_1.fq.*"
    
    # Find all matching files recursively
    matches = []
    for root, dirs, files in os.walk("."):
        for pattern_match in glob.glob(os.path.join(root, pattern)):
            matches.append(pattern_match)
    
    renamed_count = 0
    
    for file_path in matches:
        # Skip files that already end with read1.fq.*
        if re.search(r'read1\.fq\.[^/]*$', file_path):
            continue
            
        # Create new filename by replacing _1.fq with _read1.fq
        new_path = re.sub(r'_1\.fq\.', '_read1.fq.', file_path)
        
        # Only rename if the pattern was actually found and replaced
        if new_path != file_path:
            try:
                os.rename(file_path, new_path)
                print(f"Renamed: {file_path} -> {new_path}")
                renamed_count += 1
            except OSError as e:
                print(f"Error renaming {file_path}: {e}")
    
    print(f"\nRenamed {renamed_count} files total.")

if __name__ == "__main__":
    find_and_rename_output_files()