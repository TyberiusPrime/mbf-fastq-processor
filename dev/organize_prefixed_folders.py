#!/usr/bin/env python3
"""
Script to organize folders with a common prefix into a parent folder.
Takes all folders matching a prefix, creates a parent folder with that prefix,
moves the folders into it (stripping prefix and optional '_' separator),
and fixes symlinks to be one level deeper.
"""

import os
import sys
from pathlib import Path


def find_prefixed_folders(directory, prefix):
    """Find all folders that start with the given prefix."""
    folders = []
    for item in directory.iterdir():
        if item.is_dir() and item.name.startswith(prefix):
            folders.append(item)
    return folders


def strip_prefix_and_separator(folder_name, prefix):
    """Strip prefix and optional '_' separator from folder name."""
    if folder_name.startswith(prefix + '_'):
        return folder_name[len(prefix) + 1:]
    elif folder_name.startswith(prefix):
        return folder_name[len(prefix):]
    else:
        return folder_name


def fix_symlinks_in_folder(folder_path, depth_increase=1):
    """Fix symlinks in a folder to account for being moved deeper."""
    for item in folder_path.rglob('*'):
        if item.is_symlink():
            target = item.readlink()
            if not target.is_absolute():
                # Relative symlink - need to add '../' for each level deeper
                new_target = Path('../' * depth_increase) / target
                print(f"Fixing symlink {item.relative_to(folder_path)}: {target} -> {new_target}")
                item.unlink()
                item.symlink_to(new_target)


def main():
    if len(sys.argv) != 2:
        print("Usage: python organize_prefixed_folders.py <prefix>")
        sys.exit(1)
    
    prefix = sys.argv[1]
    current_dir = Path('.')
    
    # Find all folders with the given prefix
    prefixed_folders = find_prefixed_folders(current_dir, prefix)
    
    if not prefixed_folders:
        print(f"No folders found with prefix '{prefix}'")
        return
    
    print(f"Found {len(prefixed_folders)} folders with prefix '{prefix}':")
    for folder in prefixed_folders:
        print(f"  {folder.name}")
    
    # Handle existing parent folder with the prefix name
    parent_folder = current_dir / prefix
    if parent_folder.exists():
        print(f"Parent folder '{prefix}' already exists, renaming to '{prefix}/basic'")
        basic_folder = parent_folder / 'basic'
        temp_folder = current_dir / f"{prefix}_temp"
        
        # Move existing folder to temp location, create new parent, then move to basic
        parent_folder.rename(temp_folder)
        parent_folder.mkdir()
        temp_folder.rename(basic_folder)
        
        # Fix symlinks in the renamed folder
        fix_symlinks_in_folder(basic_folder)
        print(f"Moved existing '{prefix}' -> '{prefix}/basic'")
    else:
        parent_folder.mkdir()
        print(f"Created parent folder '{prefix}'")
    
    # Move each folder into the parent, stripping prefix
    for folder in prefixed_folders:
        new_name = strip_prefix_and_separator(folder.name, prefix)
        new_path = parent_folder / new_name
        
        if new_path.exists():
            print(f"Warning: Target folder '{new_path}' already exists, skipping '{folder.name}'")
            continue
        
        print(f"Moving '{folder.name}' -> '{prefix}/{new_name}'")
        folder.rename(new_path)
        
        # Fix symlinks in the moved folder
        fix_symlinks_in_folder(new_path)
    
    print("Done!")


if __name__ == "__main__":
    main()