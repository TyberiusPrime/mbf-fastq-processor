#!/usr/bin/env python3
"""
Find shared input files and organize them into sample_data.
"""

import hashlib
from pathlib import Path
from collections import defaultdict
import os

def md5_file(filepath):
    """Calculate MD5 hash of a file."""
    hash_md5 = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

def find_input_files():
    """Find all input_* files (except input.toml)."""
    files = []
    for f in Path('test_cases').rglob('input_*'):
        if f.name != 'input.toml' and (f.is_file() or f.is_symlink()):
            files.append(f)
    return files

def resolve_symlinks(files):
    """Resolve symlinks to their targets."""
    resolved = {}
    for f in files:
        if f.is_symlink():
            target = f.resolve()
            resolved[f] = target
        else:
            resolved[f] = f
    return resolved

def find_duplicates(files):
    """Find duplicate files by content hash."""
    hash_to_files = defaultdict(list)

    for f in files:
        # Ensure absolute path
        abs_f = f.resolve() if not f.is_absolute() else f
        if abs_f.is_file():
            file_hash = md5_file(abs_f)
            hash_to_files[file_hash].append(abs_f)

    # Only return groups with 2+ files
    duplicates = {h: fs for h, fs in hash_to_files.items() if len(fs) > 1}
    return duplicates

def categorize_file_group(files):
    """Determine category for a group of duplicate files."""
    # Look at file names and paths to determine what they are
    names = [f.name for f in files]
    paths = [str(f) for f in files]

    # Check file extensions
    if any('.bam' in n for n in names):
        return 'bam'
    if any('.fasta' in n or '.fa' in n for n in names):
        return 'fasta'
    if any('.zst' in n for n in names):
        return 'zstd'
    if any('.gz' in n for n in names):
        return 'gzip'

    # Check paths for clues
    path_str = ' '.join(paths)
    if 'demultiplex' in path_str:
        return 'demultiplex'
    if 'extract' in path_str:
        return 'extraction'
    if 'cock_et_al' in path_str:
        return 'cock_et_al'
    if 'filter' in path_str:
        return 'filter'
    if 'paired' in path_str or ('read1' in path_str and 'read2' in path_str):
        return 'paired_end'

    # Default
    return 'misc'

def suggest_filename(files, category):
    """Suggest a descriptive filename for the shared file."""
    # Prefer shorter paths (likely the canonical one)
    shortest = min(files, key=lambda f: len(str(f)))
    base_name = shortest.name

    # For very common files, add more context from path
    if len(files) > 10:
        # Get a distinctive part of the path
        path_parts = str(shortest).split('/')
        # Find a meaningful directory name
        for part in path_parts:
            if part not in ['test_cases', 'input_validation', 'integration_tests', 'extraction']:
                if part not in base_name:
                    # Use it as prefix
                    base_name = f"{part}_{base_name}"
                    break

    return base_name

def main():
    print("Finding input files...")
    files = find_input_files()
    print(f"Found {len(files)} input files")

    # Resolve symlinks
    print("\nResolving symlinks...")
    resolved = resolve_symlinks(files)
    symlinks = {f: t for f, t in resolved.items() if f != t}
    print(f"Found {len(symlinks)} symlinks")
    base = Path('test_cases').resolve()
    for sym, target in symlinks.items():
        print(f"  {sym.relative_to('test_cases')} -> {target.relative_to(base)}")

    # Get unique real files
    unique_files = set(resolved.values())
    print(f"\nAnalyzing {len(unique_files)} unique files for duplicates...")

    # Find duplicates
    duplicates = find_duplicates(unique_files)
    print(f"Found {len(duplicates)} groups of duplicate files")

    # Organize by category
    category_groups = defaultdict(list)
    for file_hash, file_group in duplicates.items():
        category = categorize_file_group(file_group)
        category_groups[category].append((file_hash, file_group))

    print("\nDuplicate groups by category:")
    base = Path('test_cases').resolve()
    for category in sorted(category_groups.keys()):
        groups = category_groups[category]
        total_files = sum(len(g[1]) for g in groups)
        print(f"\n  {category}: {len(groups)} groups, {total_files} files")
        for file_hash, file_group in groups[:3]:  # Show first 3
            print(f"    {len(file_group)} copies: {suggest_filename(file_group, category)}")
            for f in file_group[:2]:  # Show first 2 paths
                print(f"      - {f.relative_to(base)}")
            if len(file_group) > 2:
                print(f"      ... and {len(file_group) - 2} more")

    # Generate migration plan
    print("\n" + "="*80)
    print("Generating migration plan...")

    migration = []
    used_targets = set()

    for category in sorted(category_groups.keys()):
        groups = category_groups[category]
        for file_hash, file_group in groups:
            # Pick canonical file (shortest path)
            canonical = min(file_group, key=lambda f: len(str(f)))
            filename = suggest_filename(file_group, category)

            # Target in sample_data
            target = f"sample_data/{category}/{filename}"

            # Avoid naming conflicts
            counter = 1
            original_target = target
            while target in used_targets:
                # Add counter before extension
                parts = filename.rsplit('.', 1)
                if len(parts) == 2:
                    new_filename = f"{parts[0]}_{counter}.{parts[1]}"
                else:
                    new_filename = f"{filename}_{counter}"
                target = f"sample_data/{category}/{new_filename}"
                counter += 1

            used_targets.add(target)

            migration.append({
                'canonical': canonical,
                'duplicates': [f for f in file_group if f != canonical],
                'target': target,
                'category': category
            })

    # Write migration script
    base = Path('test_cases').resolve()
    with open('migrate_to_sample_data.txt', 'w') as f:
        for item in migration:
            f.write(f"# {len(item['duplicates']) + 1} copies\n")
            f.write(f"{item['canonical'].relative_to(base)}\n")
            f.write(f"{item['target']}\n")
            for dup in item['duplicates']:
                f.write(f"{dup.relative_to(base)}\n")
            f.write("\n")

    # Also handle existing symlinks
    base = Path('test_cases').resolve()
    with open('symlinks_to_migrate.txt', 'w') as f:
        for sym, target in symlinks.items():
            f.write(f"{sym.relative_to('test_cases')}\n")
            f.write(f"{target.relative_to(base)}\n")
            f.write("\n")

    print(f"\nWrote migration plan:")
    print(f"  migrate_to_sample_data.txt - {len(migration)} groups to migrate")
    print(f"  symlinks_to_migrate.txt - {len(symlinks)} symlinks to update")

    print("\nSummary:")
    print(f"  Total input files: {len(files)}")
    print(f"  Symlinks: {len(symlinks)}")
    print(f"  Unique files: {len(unique_files)}")
    print(f"  Duplicate groups: {len(duplicates)}")
    print(f"  Files to migrate: {sum(len(item['duplicates']) + 1 for item in migration)}")

if __name__ == '__main__':
    main()
