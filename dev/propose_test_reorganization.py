#!/usr/bin/env python3
"""
Generate a comprehensive reorganization plan for test cases.
Creates a shell script with all rename commands for review.
"""

import os
from pathlib import Path
from collections import defaultdict
import toml

def find_all_test_cases():
    """Find all test cases with input.toml files."""
    test_cases = []
    test_cases_dir = Path("test_cases")

    for input_file in test_cases_dir.rglob("input.toml"):
        test_dir = input_file.parent
        rel_path = test_dir.relative_to(test_cases_dir)
        test_cases.append(rel_path)

    return sorted(test_cases)

def load_config(test_path):
    """Load and parse the input.toml config."""
    config_file = Path("test_cases") / test_path / "input.toml"
    try:
        with open(config_file, 'r') as f:
            return toml.load(f)
    except Exception:
        return None

def extract_primary_feature(path_str, config):
    """Determine the primary feature being tested."""
    parts = path_str.split('/')
    old_category = parts[0]

    # Parse config to understand what's being tested
    steps = []
    if config and 'step' in config:
        step_list = config['step'] if isinstance(config['step'], list) else [config['step']]
        for step in step_list:
            # Get the step type (the key that's not 'description' or 'name')
            for key in step.keys():
                if key not in ['description', 'name']:
                    steps.append(key)

    return old_category, parts, steps

def suggest_new_path(test_path, config):
    """Suggest a new path for a test case."""
    path_str = str(test_path)
    old_category, parts, steps = extract_primary_feature(path_str, config)

    # Build reason string for transparency
    reasons = []

    # Handle special cases first
    if 'cock_et_al' in path_str:
        if 'reject' in path_str:
            new_cat = 'error_handling/malformed_fastq'
            reasons.append("Cock et al. rejection tests (malformed FASTQ)")
        else:
            new_cat = 'edge_cases/challenging_formats'
            reasons.append("Cock et al. challenging format tests")
        return f"{new_cat}/{'/'.join(parts[3:])}", reasons

    # Error handling and validation
    if old_category in ['input_validation', 'outside_error_conditions']:
        # Determine subcategory
        if 'bam' in path_str and ('missing' in path_str or 'disk_full' in path_str or 'output' in path_str):
            subcat = 'bam'
            reasons.append("BAM file errors")
        elif any(x in path_str for x in ['read1', 'read2', 'index1', 'index2', 'input_file', 'missing', 'permission', 'repeated_filename']):
            subcat = 'input_files'
            reasons.append("Input file validation errors")
        elif 'output' in path_str or 'stdout' in path_str or 'segment' in path_str:
            subcat = 'output_config'
            reasons.append("Output configuration errors")
        elif 'barcode' in path_str or ('demultiplex' in path_str and old_category == 'input_validation'):
            subcat = 'demultiplex'
            reasons.append("Demultiplexing validation errors")
        elif any(x in path_str for x in ['extract', 'tag', 'umi', 'regex']):
            subcat = 'extraction'
            reasons.append("Extraction/tag validation errors")
        elif 'filter' in path_str:
            subcat = 'filter'
            reasons.append("Filter validation errors")
        elif 'swap' in path_str:
            subcat = 'swap'
            reasons.append("Swap validation errors")
        elif 'compression' in path_str or 'gzip' in path_str or 'zstd' in path_str:
            subcat = 'compression'
            reasons.append("Compression validation errors")
        elif 'convert_phred' in path_str:
            subcat = 'quality_scores'
            reasons.append("Quality score conversion errors")
        elif any(x in path_str for x in ['broken', 'truncated', 'newline', 'mismatched', 'quality', 'invalid_base']):
            subcat = 'malformed_fastq'
            reasons.append("Malformed FASTQ files")
        elif 'cli' in path_str or 'help' in path_str or 'old_cli' in path_str:
            subcat = 'cli'
            reasons.append("CLI validation errors")
        elif 'dna_validation' in path_str or 'invalid_base' in path_str or 'iupac' in path_str:
            subcat = 'dna_validation'
            reasons.append("DNA validation errors")
        elif 'paired_end' in path_str or 'read_pairing' in path_str:
            subcat = 'paired_end'
            reasons.append("Paired-end validation errors")
        elif 'report' in path_str and 'no_report' in path_str:
            subcat = 'reports'
            reasons.append("Report validation errors")
        elif 'eval' in path_str or 'expr' in path_str:
            subcat = 'eval'
            reasons.append("Expression evaluation errors")
        elif 'store_tag' in path_str or 'store_tags' in path_str:
            subcat = 'tag_storage'
            reasons.append("Tag storage validation errors")
        elif 'interleaved' in path_str or 'stdin' in path_str:
            subcat = 'io'
            reasons.append("I/O validation errors")
        elif 'show_step_template' in path_str or 'two_mistakes' in path_str:
            subcat = 'error_messages'
            reasons.append("Error message tests")
        else:
            subcat = 'misc'
            reasons.append("Miscellaneous validation errors")

        return f"error_handling/{subcat}/{'/'.join(parts[1:])}", reasons

    # Feature-based organization for working tests
    if old_category == 'demultiplex':
        reasons.append(f"Demultiplexing: {parts[-1]}")
        return f"demultiplex/{'/'.join(parts[1:])}", reasons

    if old_category == 'extraction':
        # Some extraction tests are really about demultiplexing output
        if 'demultiplex' in parts:
            reasons.append(f"Extraction with demultiplexing")
            return f"extraction/with_demultiplex/{'/'.join(parts[1:])}", reasons
        else:
            reasons.append(f"Tag/region extraction: {parts[-1]}")
            return f"extraction/{'/'.join(parts[1:])}", reasons

    if old_category == 'calc':
        reasons.append(f"Calculation/quantification: {parts[-1]}")
        return f"calc/{'/'.join(parts[1:])}", reasons

    if old_category == 'convert':
        reasons.append(f"Data conversion: {parts[-1]}")
        return f"convert/{'/'.join(parts[1:])}", reasons

    if old_category == 'edits':
        reasons.append(f"Sequence editing: {parts[-1]}")
        return f"edits/{'/'.join(parts[1:])}", reasons

    if old_category == 'fileformats':
        reasons.append(f"File format conversion: {parts[-1]}")
        return f"fileformats/{'/'.join(parts[1:])}", reasons

    if old_category == 'hamming_correct':
        reasons.append(f"Hamming distance correction: {parts[-1]}")
        return f"hamming/{'/'.join(parts[1:])}", reasons

    if old_category == 'head_early_termination':
        reasons.append(f"Head/early termination: {parts[-1]}")
        return f"head/{'/'.join(parts[1:])}", reasons

    if old_category == 'reports':
        reasons.append(f"Report generation: {parts[-1]}")
        return f"reports/{'/'.join(parts[1:])}", reasons

    if old_category == 'memory':
        reasons.append(f"Memory/performance test: {parts[-1]}")
        return f"performance/{'/'.join(parts[1:])}", reasons

    if old_category == 'output':
        reasons.append(f"Output formatting: {parts[-1]}")
        return f"output/{'/'.join(parts[1:])}", reasons

    if old_category == 'validation':
        reasons.append(f"Runtime validation: {parts[-1]}")
        return f"validation/{'/'.join(parts[1:])}", reasons

    # Handle integration_tests - split by primary feature
    if old_category == 'integration_tests':
        test_name = parts[-1]

        # Check for specific features
        if test_name.startswith('dedup') or 'dedup/' in path_str:
            reasons.append(f"Deduplication test")
            rest = parts[1:] if parts[1] != 'dedup' else parts[2:]
            return f"dedup/{'/'.join(rest)}", reasons

        if test_name.startswith('filter') or 'filter' in test_name:
            reasons.append(f"Filtering test: {test_name}")
            return f"filter/{'/'.join(parts[1:])}", reasons

        if test_name.startswith('trim') or test_name.startswith('cut') or 'trim' in test_name or 'cut' in test_name:
            reasons.append(f"Trimming/cutting test: {test_name}")
            return f"trim/{'/'.join(parts[1:])}", reasons

        if test_name.startswith('quantify') or 'quantify' in test_name:
            reasons.append(f"Quantification test")
            return f"calc/{'/'.join(parts[1:])}", reasons

        if 'demultiplex' in test_name or 'head_with_index_and_demultiplex' in test_name:
            reasons.append(f"Demultiplexing integration test")
            return f"demultiplex/{'/'.join(parts[1:])}", reasons

        if 'bam' in test_name or 'filter_other_file' in test_name:
            reasons.append(f"BAM/external file test")
            return f"fileformats/{'/'.join(parts[1:])}", reasons

        if test_name.startswith('eval') or 'eval' in test_name or 'expr' in test_name:
            reasons.append(f"Expression evaluation test")
            return f"eval/{'/'.join(parts[1:])}", reasons

        if 'interleaved' in test_name or 'stdin' in test_name or 'stdout' in test_name:
            reasons.append(f"I/O format test")
            return f"io/{'/'.join(parts[1:])}", reasons

        if 'hash' in test_name:
            reasons.append(f"Hash validation test")
            return f"validation/hash/{'/'.join(parts[1:])}", reasons

        if 'compress' in test_name or 'gzip' in test_name or 'zstd' in test_name or 'gz_input' in test_name:
            reasons.append(f"Compression test")
            return f"compression/{'/'.join(parts[1:])}", reasons

        if 'inspect' in test_name or 'inspect/' in path_str:
            reasons.append(f"File inspection test")
            return f"inspect/{'/'.join(parts[1:])}", reasons

        if 'output' in test_name or 'output/' in path_str:
            reasons.append(f"Output configuration test")
            return f"output/{'/'.join(parts[1:])}", reasons

        if 'extract_regex' in path_str or 'extract_iupac' in path_str:
            reasons.append(f"Extraction test")
            return f"extraction/{'/'.join(parts[1:])}", reasons

        if 'swap' in test_name:
            reasons.append(f"Swap test")
            return f"swap/{'/'.join(parts[1:])}", reasons

        if 'reverse_complement' in test_name:
            reasons.append(f"Reverse complement test")
            return f"transform/{'/'.join(parts[1:])}", reasons

        if 'subsample' in test_name or 'skip' in test_name:
            reasons.append(f"Sampling test")
            return f"sampling/{'/'.join(parts[1:])}", reasons

        if 'rename' in test_name:
            reasons.append(f"Renaming test")
            return f"transform/{'/'.join(parts[1:])}", reasons

        if 'fastp' in test_name:
            reasons.append(f"fastp compatibility test")
            return f"compatibility/{'/'.join(parts[1:])}", reasons

        if 'old_cli' in test_name:
            reasons.append(f"Legacy CLI test")
            return f"compatibility/{'/'.join(parts[1:])}", reasons

        if test_name in ['noop', 'noop_minimal', 'allow_overwrites']:
            reasons.append(f"Basic functionality test")
            return f"basic/{'/'.join(parts[1:])}", reasons

        if 'very_long' in test_name or 'mega_long' in test_name:
            reasons.append(f"Long read test")
            return f"edge_cases/{'/'.join(parts[1:])}", reasons

        if 'order_maintained' in test_name or 'gzip_blocks_spliting' in test_name:
            reasons.append(f"Correctness test")
            return f"correctness/{'/'.join(parts[1:])}", reasons

        if 'low_complexity' in test_name or 'quality_base' in test_name or 'convert_phred' in test_name:
            reasons.append(f"Quality/complexity test")
            return f"quality/{'/'.join(parts[1:])}", reasons

        if 'prefix' in test_name or 'postfix' in test_name:
            reasons.append(f"Prefix/postfix test")
            return f"transform/{'/'.join(parts[1:])}", reasons

        if 'max_len' in test_name:
            reasons.append(f"Length constraint test")
            return f"transform/{'/'.join(parts[1:])}", reasons

        if 'ten_segments' in test_name:
            reasons.append(f"Multi-segment test")
            return f"edge_cases/{'/'.join(parts[1:])}", reasons

        # Default for remaining integration tests
        reasons.append(f"General integration test: {test_name}")
        return f"integration/{'/'.join(parts[1:])}", reasons

    # Fallback
    reasons.append(f"Keeping original category")
    return path_str, reasons

def generate_reorganization_plan():
    """Generate complete reorganization plan."""
    test_cases = find_all_test_cases()
    plan = []

    for test_path in test_cases:
        config = load_config(test_path)
        new_path, reasons = suggest_new_path(test_path, config)

        plan.append({
            'old_path': str(test_path),
            'new_path': new_path,
            'reasons': reasons
        })

    return plan

def print_summary(plan):
    """Print summary of reorganization."""
    # Count changes by category
    category_changes = defaultdict(lambda: {'from': defaultdict(int), 'to': defaultdict(int)})

    for item in plan:
        old_cat = item['old_path'].split('/')[0]
        new_cat = item['new_path'].split('/')[0]
        category_changes[old_cat]['to'][new_cat] += 1

    print("=" * 80)
    print("REORGANIZATION SUMMARY")
    print("=" * 80)
    print()

    for old_cat in sorted(category_changes.keys()):
        destinations = category_changes[old_cat]['to']
        total = sum(destinations.values())
        print(f"\n{old_cat}/ ({total} tests)")
        print("-" * 60)
        for new_cat, count in sorted(destinations.items(), key=lambda x: -x[1]):
            print(f"  → {new_cat:30s} {count:3d} tests")

    # New category summary
    print("\n" + "=" * 80)
    print("NEW CATEGORY STRUCTURE")
    print("=" * 80)
    print()

    new_categories = defaultdict(int)
    for item in plan:
        new_cat = item['new_path'].split('/')[0]
        new_categories[new_cat] += 1

    for cat in sorted(new_categories.keys()):
        print(f"  {cat:30s} {new_categories[cat]:4d} tests")

    print(f"\n  {'TOTAL':30s} {len(plan):4d} tests")

def generate_rename_script(plan, output_file):
    """Generate bash script with rename commands."""
    with open(output_file, 'w') as f:
        f.write("#!/bin/bash\n")
        f.write("# Test case reorganization script\n")
        f.write("# Generated automatically - review before executing!\n")
        f.write("#\n")
        f.write("# This script renames test case directories according to the new organization.\n")
        f.write("# Review the changes carefully before running.\n")
        f.write("#\n")
        f.write("# Usage: bash reorganize_tests.sh\n")
        f.write("#\n\n")

        f.write("set -euo pipefail\n\n")

        f.write("# Color output\n")
        f.write("RED='\\033[0;31m'\n")
        f.write("GREEN='\\033[0;32m'\n")
        f.write("YELLOW='\\033[1;33m'\n")
        f.write("NC='\\033[0m' # No Color\n\n")

        f.write("echo \"Test Case Reorganization Script\"\n")
        f.write("echo \"================================\"\n")
        f.write("echo \"\"\n")
        f.write(f"echo \"This will reorganize {len(plan)} test cases.\"\n")
        f.write("echo \"\"\n")
        f.write("read -p \"Are you sure you want to continue? (yes/no) \" -r\n")
        f.write("echo\n")
        f.write("if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then\n")
        f.write("    echo \"Aborting.\"\n")
        f.write("    exit 1\n")
        f.write("fi\n\n")

        f.write("cd test_cases\n\n")

        # Group by old category for organization
        by_old_cat = defaultdict(list)
        for item in plan:
            old_cat = item['old_path'].split('/')[0]
            by_old_cat[old_cat].append(item)

        move_count = 0
        for old_cat in sorted(by_old_cat.keys()):
            items = by_old_cat[old_cat]

            # Sort to handle nested directories properly (move deepest first)
            items.sort(key=lambda x: x['old_path'].count('/'), reverse=True)

            f.write(f"\n# {old_cat}/ ({len(items)} tests)\n")
            f.write(f"echo \"Processing {old_cat}/...\"\n\n")

            for item in items:
                old_path = item['old_path']
                new_path = item['new_path']

                if old_path != new_path:
                    move_count += 1
                    reasons_str = '; '.join(item['reasons'])
                    f.write(f"# {reasons_str}\n")
                    f.write(f"if [ -d \"{old_path}\" ]; then\n")
                    f.write(f"    mkdir -p \"$(dirname '{new_path}')\"\n")
                    f.write(f"    echo \"  {old_path} → {new_path}\"\n")
                    f.write(f"    mv \"{old_path}\" \"{new_path}\"\n")
                    f.write(f"fi\n\n")

        f.write(f"\necho \"\"\n")
        f.write(f"echo \"${{GREEN}}Reorganization complete!${{NC}}\"\n")
        f.write(f"echo \"Moved {move_count} test directories.\"\n")
        f.write(f"echo \"\"\n")
        f.write(f"echo \"${{YELLOW}}Next steps:${{NC}}\"\n")
        f.write(f"echo \"1. Run: dev/update_tests.py\"\n")
        f.write(f"echo \"2. Run: cargo test\"\n")
        f.write(f"echo \"3. Commit the changes\"\n")

    os.chmod(output_file, 0o755)

def generate_detailed_list(plan, output_file):
    """Generate detailed text file with all renames and reasons."""
    with open(output_file, 'w') as f:
        f.write("DETAILED TEST REORGANIZATION PLAN\n")
        f.write("=" * 80)
        f.write("\n\n")

        f.write(f"Total test cases: {len(plan)}\n")
        move_count = sum(1 for item in plan if item['old_path'] != item['new_path'])
        f.write(f"To be moved: {move_count}\n")
        f.write(f"Staying in place: {len(plan) - move_count}\n")
        f.write("\n\n")

        # Group by old category
        by_old_cat = defaultdict(list)
        for item in plan:
            old_cat = item['old_path'].split('/')[0]
            by_old_cat[old_cat].append(item)

        for old_cat in sorted(by_old_cat.keys()):
            items = by_old_cat[old_cat]

            f.write("\n" + "=" * 80 + "\n")
            f.write(f"{old_cat}/ ({len(items)} tests)\n")
            f.write("=" * 80 + "\n\n")

            # Group by new category
            by_new_cat = defaultdict(list)
            for item in items:
                new_cat = item['new_path'].split('/')[0]
                by_new_cat[new_cat].append(item)

            for new_cat in sorted(by_new_cat.keys()):
                new_items = by_new_cat[new_cat]
                f.write(f"\n  → {new_cat}/ ({len(new_items)} tests)\n")
                f.write("  " + "-" * 76 + "\n")

                for item in new_items:
                    if item['old_path'] != item['new_path']:
                        f.write(f"\n  OLD: {item['old_path']}\n")
                        f.write(f"  NEW: {item['new_path']}\n")
                        f.write(f"  WHY: {'; '.join(item['reasons'])}\n")
                    else:
                        f.write(f"\n  UNCHANGED: {item['old_path']}\n")

if __name__ == '__main__':
    print("Generating reorganization plan...")
    plan = generate_reorganization_plan()

    print_summary(plan)

    # Generate the rename script
    script_file = "reorganize_tests.sh"
    print(f"\n\nGenerating rename script: {script_file}")
    generate_rename_script(plan, script_file)

    # Generate detailed list
    detail_file = "test_reorganization_plan.txt"
    print(f"Generating detailed plan: {detail_file}")
    generate_detailed_list(plan, detail_file)

    print(f"\n{'-' * 80}")
    print("Done!")
    print(f"\nReview the changes in: {detail_file}")
    print(f"To execute renames, run: ./{script_file}")
    print(f"\nWARNING: Review carefully before running the script!")
