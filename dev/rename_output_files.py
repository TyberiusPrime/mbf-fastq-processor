#!/usr/bin/env python3

import os
import glob
import sys

def rename_output_files():
    """Rename output_*.fq* files to use read1/read2/index1/index2 naming convention"""
    
    # Mapping of old patterns to new patterns
    rename_map = {
        'output_1.fq': 'output_read1.fq',
        'output_2.fq': 'output_read2.fq', 
        'output_i1.fq': 'output_index1.fq',
        'output_i2.fq': 'output_index2.fq'
    }
    
    # Also handle compressed versions
    for ext in ['.gz', '.zst']:
        for old, new in list(rename_map.items()):
            rename_map[old + ext] = new + ext
    
    renamed_count = 0
    
    # Find and rename files
    for old_pattern, new_pattern in rename_map.items():
        for filepath in glob.glob(f'**/{old_pattern}', recursive=True):
            # Generate new filename
            dirname = os.path.dirname(filepath)
            new_filepath = os.path.join(dirname, new_pattern)
            
            if os.path.exists(new_filepath):
                print(f"Warning: {new_filepath} already exists, skipping {filepath}")
                continue
                
            print(f"Renaming: {filepath} -> {new_filepath}")
            os.rename(filepath, new_filepath)
            renamed_count += 1
    
    print(f"Renamed {renamed_count} files")

if __name__ == "__main__":
    rename_output_files()