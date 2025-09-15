#!/usr/bin/env python3
"""
Script to update test cases that have exactly one segment defined in their input
to remove segment specifications from steps, allowing them to use the new default behavior.
"""

import os
import sys
import tomllib
from pathlib import Path
import argparse
import re


def find_input_toml_files(base_dir):
    """Find all input.toml files in the test_cases directory"""
    test_cases_dir = Path(base_dir) / "test_cases"
    return list(test_cases_dir.glob("**/input.toml"))


def analyze_config(config_data):
    """Analyze a TOML config to determine if it has exactly one segment"""
    if 'input' not in config_data:
        return False, []
    
    input_section = config_data['input']
    
    # Count non-interleaved segments (exclude 'interleaved' key if present)
    segment_keys = [k for k in input_section.keys() if k != 'interleaved']
    
    # Return whether there's exactly one segment and the list of segment names
    return len(segment_keys) == 1, segment_keys


def update_config_steps(config_data, segment_names):
    """Remove segment specifications from steps when there's exactly one segment available"""
    if 'step' not in config_data:
        return False, []
    
    steps = config_data['step']
    if not isinstance(steps, list):
        steps = [steps]  # Handle single step case
    
    changes_made = []
    for i, step in enumerate(steps):
        if 'segment' in step:
            segment_value = step['segment']
            
            # Remove segment specification if it matches the only available segment
            # or if it's 'All' (which would work with single segments anyway)
            if segment_value in segment_names or segment_value in ['All', 'all']:
                del step['segment']
                changes_made.append(f"Step {i}: Removed 'segment = \"{segment_value}\"'")
    
    return len(changes_made) > 0, changes_made


def process_file(file_path, dry_run=True):
    """Process a single input.toml file"""
    try:
        # Read the file
        with open(file_path, 'rb') as f:
            config_data = tomllib.load(f)
        
        # Analyze if it has exactly one segment
        has_single_segment, segment_names = analyze_config(config_data)
        
        if not has_single_segment:
            return False, f"Skipped: Has {len(segment_names)} segments"
        
        # Read original file content for text processing
        with open(file_path, 'r') as f:
            original_content = f.read()
        
        # Find lines to remove using regex
        changes_made = []
        updated_content = original_content
        
        # Look for segment specifications to remove
        for segment_name in segment_names + ['All', 'all']:
            # Pattern to match segment lines
            pattern = rf"^\s*segment\s*=\s*['\"]?{re.escape(segment_name)}['\"]?\s*$"
            matches = list(re.finditer(pattern, updated_content, re.MULTILINE))
            
            for match in reversed(matches):  # Remove from end to preserve line numbers
                line_start = updated_content.rfind('\n', 0, match.start()) + 1
                line_end = updated_content.find('\n', match.end())
                if line_end == -1:
                    line_end = len(updated_content)
                else:
                    line_end += 1  # include the newline
                
                changes_made.append(f"Removed line: {match.group().strip()}")
                updated_content = updated_content[:line_start] + updated_content[line_end:]
        
        if not changes_made:
            return False, "Skipped: No segment specifications to remove"
        
        if not dry_run:
            # Write the updated file
            with open(file_path, 'w') as f:
                f.write(updated_content)
        
        return True, f"Updated: {'; '.join(changes_made)}"
        
    except Exception as e:
        return False, f"Error: {str(e)}"


def main():
    parser = argparse.ArgumentParser(description="Update single-segment test cases to use default segment behavior")
    parser.add_argument("--dry-run", action="store_true", default=True,
                        help="Show what would be changed without making changes (default)")
    parser.add_argument("--apply", action="store_true",
                        help="Actually make the changes to files")
    parser.add_argument("--base-dir", default=".",
                        help="Base directory containing test_cases (default: current directory)")
    
    args = parser.parse_args()
    
    # If --apply is specified, don't do dry run
    dry_run = not args.apply
    
    base_dir = Path(args.base_dir)
    
    if not (base_dir / "test_cases").exists():
        print(f"Error: test_cases directory not found in {base_dir}")
        sys.exit(1)
    
    # Find all input.toml files
    input_files = find_input_toml_files(base_dir)
    print(f"Found {len(input_files)} input.toml files")
    
    if dry_run:
        print("DRY RUN MODE - No files will be modified")
        print("Use --apply to actually make changes")
    
    print()
    
    updated_count = 0
    skipped_count = 0
    
    for file_path in sorted(input_files):
        rel_path = file_path.relative_to(base_dir)
        was_updated, message = process_file(file_path, dry_run)
        
        if was_updated:
            print(f"✓ {rel_path}: {message}")
            updated_count += 1
        else:
            if args.apply or message.startswith("Error:"):
                print(f"- {rel_path}: {message}")
            skipped_count += 1
    
    print()
    print(f"Summary: {updated_count} files {'would be ' if dry_run else ''}updated, {skipped_count} files skipped")
    
    if dry_run and updated_count > 0:
        print("Run with --apply to make the changes")


if __name__ == "__main__":
    main()