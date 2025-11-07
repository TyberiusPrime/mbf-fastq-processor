#!/usr/bin/env python3
"""
Migrate shared input files to sample_data folder.
"""

import shutil
from pathlib import Path
import os

def read_migration_plan(filename):
    """Read migration plan from migrate_to_sample_data.txt."""
    groups = []

    with open(filename, 'r') as f:
        lines = [l.rstrip('\n') for l in f]

    i = 0
    while i < len(lines):
        if lines[i].startswith('#'):
            # Comment line with count
            i += 1
            if i >= len(lines):
                break

            canonical = lines[i]
            i += 1
            if i >= len(lines):
                break

            target = lines[i]
            i += 1

            duplicates = []
            while i < len(lines) and lines[i] and not lines[i].startswith('#'):
                duplicates.append(lines[i])
                i += 1

            groups.append({
                'canonical': canonical,
                'target': target,
                'duplicates': duplicates
            })

            # Skip empty line
            if i < len(lines) and not lines[i]:
                i += 1
        else:
            i += 1

    return groups

def read_symlinks(filename):
    """Read symlink migrations."""
    symlinks = []

    with open(filename, 'r') as f:
        lines = [l.rstrip('\n') for l in f]

    i = 0
    while i + 1 < len(lines):
        if lines[i] and lines[i + 1]:
            symlinks.append((lines[i], lines[i + 1]))
            i += 3  # Skip empty line
        else:
            i += 1

    return symlinks

def main():
    base_dir = Path('test_cases')

    print("Reading migration plan...")
    groups = read_migration_plan('migrate_to_sample_data.txt')
    print(f"Found {len(groups)} file groups to migrate")

    print("Reading symlink plan...")
    symlinks = read_symlinks('symlinks_to_migrate.txt')
    print(f"Found {len(symlinks)} symlinks to update")

    # Create sample_data directory
    sample_data_dir = base_dir / 'sample_data'
    print(f"\nCreating {sample_data_dir}/...")
    sample_data_dir.mkdir(exist_ok=True)

    # Migrate file groups
    print("\nMigrating file groups to sample_data...")
    for group in groups:
        target_path = base_dir / group['target']
        canonical_path = base_dir / group['canonical']

        # Create parent directory
        target_path.parent.mkdir(parents=True, exist_ok=True)

        # Move canonical file to sample_data
        if canonical_path.exists():
            print(f"  {group['canonical']} -> {group['target']}")
            shutil.copy2(canonical_path, target_path)
            canonical_path.unlink()

            # Create symlink from old location to new
            rel_target = os.path.relpath(target_path, canonical_path.parent)
            canonical_path.symlink_to(rel_target)

        # Replace duplicates with symlinks
        for dup in group['duplicates']:
            dup_path = base_dir / dup
            if dup_path.exists():
                print(f"  {dup} -> {group['target']}")
                dup_path.unlink()

                # Create symlink to sample_data
                rel_target = os.path.relpath(target_path, dup_path.parent)
                dup_path.symlink_to(rel_target)

    # Update existing symlinks
    print("\nUpdating existing symlinks...")
    for sym, old_target in symlinks:
        sym_path = base_dir / sym
        old_target_path = base_dir / old_target

        if sym_path.is_symlink():
            print(f"  Updating {sym}")
            sym_path.unlink()

            # Find new target (should be in sample_data now)
            # Look for matching file in migration groups
            new_target = None
            for group in groups:
                if group['canonical'] == old_target or old_target in group['duplicates']:
                    new_target = group['target']
                    break

            if new_target:
                new_target_path = base_dir / new_target
                rel_target = os.path.relpath(new_target_path, sym_path.parent)
                sym_path.symlink_to(rel_target)
                print(f"    -> {new_target}")
            else:
                # Symlink target wasn't migrated, recreate original
                rel_target = os.path.relpath(old_target_path, sym_path.parent)
                sym_path.symlink_to(rel_target)
                print(f"    -> {old_target} (unchanged)")

    print("\nDone!")
    print(f"  Migrated {len(groups)} file groups to sample_data/")
    print(f"  Updated {len(symlinks)} symlinks")

    # Count files in sample_data
    sample_files = list((base_dir / 'sample_data').rglob('input_*'))
    print(f"  sample_data/ now contains {len(sample_files)} files")

if __name__ == '__main__':
    main()
