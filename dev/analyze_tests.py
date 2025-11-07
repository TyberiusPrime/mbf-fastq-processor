#!/usr/bin/env python3
"""
Analyze all integration tests and propose reorganization.
"""

import os
import re
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

def is_expected_panic(test_path):
    """Check if test is expected to panic by checking Cargo.toml."""
    # Read Cargo.toml to see if this test has expected_panic attribute
    test_name = str(test_path).replace("/", "_")

    # Check integration_tests.rs for the test function
    integration_tests = Path("tests/integration_tests.rs")
    if integration_tests.exists():
        content = integration_tests.read_text()
        # Look for test function with this name
        pattern = rf'#\[test\].*?fn\s+{re.escape(test_name)}\(\)'
        if re.search(pattern, content, re.DOTALL):
            # Check if there's #[should_panic] before it
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if f'fn {test_name}()' in line:
                    # Check previous lines for should_panic
                    for j in range(max(0, i-5), i):
                        if 'should_panic' in lines[j]:
                            return True
    return False

def categorize_test(test_path, config):
    """Categorize a test based on its path, name, and config."""
    path_str = str(test_path)
    parts = path_str.split('/')

    categories = []
    features = []

    # Check if it's a validation/error test
    if 'input_validation' in path_str or 'outside_error_conditions' in path_str:
        categories.append('validation')
    elif 'expected_panic' in path_str or 'panic' in path_str:
        categories.append('validation')

    # Check for specific features in path
    if 'demultiplex' in path_str:
        features.append('demultiplex')
    if 'extract' in path_str:
        features.append('extract')
    if 'filter' in path_str:
        features.append('filter')
    if 'trim' in path_str or 'cut' in path_str:
        features.append('trim')
    if 'report' in path_str:
        features.append('report')
    if 'bam' in path_str or 'fasta' in path_str or 'fastq' in path_str or 'fileformat' in path_str:
        features.append('fileformat')
    if 'head' in path_str:
        features.append('head')
    if 'dedup' in path_str:
        features.append('dedup')
    if 'calc' in path_str or 'quantify' in path_str:
        features.append('calc')
    if 'convert' in path_str:
        features.append('convert')
    if 'hamming' in path_str:
        features.append('hamming')
    if 'umi' in path_str:
        features.append('umi')
    if 'merge' in path_str:
        features.append('merge')
    if 'stdin' in path_str or 'stdout' in path_str:
        features.append('io')
    if 'interleaved' in path_str:
        features.append('interleaved')
    if 'compression' in path_str or 'gzip' in path_str or 'zstd' in path_str:
        features.append('compression')
    if 'swap' in path_str:
        features.append('swap')
    if 'reverse_complement' in path_str:
        features.append('reverse_complement')
    if 'subsample' in path_str or 'skip' in path_str:
        features.append('sampling')
    if 'output' in parts[0]:
        features.append('output')
    if 'memory' in path_str:
        features.append('memory')
    if 'eval' in path_str or 'expr' in path_str:
        features.append('eval')

    # Check steps in config
    if config and 'step' in config:
        steps = config['step'] if isinstance(config['step'], list) else [config['step']]
        for step in steps:
            step_type = step.keys()
            for key in step_type:
                if key not in ['description', 'name']:
                    features.append(key)

    return categories, features

def load_config(test_path):
    """Load and parse the input.toml config."""
    config_file = Path("test_cases") / test_path / "input.toml"
    try:
        with open(config_file, 'r') as f:
            return toml.load(f)
    except Exception as e:
        return None

def analyze_all_tests():
    """Analyze all tests and generate report."""
    test_cases = find_all_test_cases()

    analysis = []

    for test_path in test_cases:
        config = load_config(test_path)
        is_panic = is_expected_panic(test_path)
        categories, features = categorize_test(test_path, config)

        analysis.append({
            'path': str(test_path),
            'old_category': str(test_path).split('/')[0],
            'is_panic': is_panic,
            'categories': categories,
            'features': list(set(features)),
            'config': config
        })

    return analysis

def suggest_new_organization(analysis):
    """Suggest new organization based on analysis."""
    suggestions = []

    for item in analysis:
        path = item['path']
        old_cat = item['old_category']
        features = item['features']
        is_panic = item['is_panic']

        # Determine primary category
        if 'validation' in item['categories'] or is_panic:
            new_category = 'error_handling'
        elif old_cat in ['input_validation', 'outside_error_conditions']:
            new_category = 'error_handling'
        elif 'cock_et_al' in path:
            new_category = 'external_datasets'
        elif old_cat == 'memory':
            new_category = 'performance'
        elif old_cat == 'head_early_termination':
            new_category = 'head'
        elif old_cat == 'hamming_correct':
            new_category = 'hamming'
        elif old_cat == 'reports':
            new_category = 'reports'
        elif old_cat == 'validation':
            new_category = 'validation'
        elif 'demultiplex' in features:
            new_category = 'demultiplex'
        elif 'extract' in features and old_cat == 'extraction':
            new_category = 'extraction'
        elif 'calc' in features or 'quantify' in features:
            new_category = 'calc'
        elif 'convert' in features and old_cat == 'convert':
            new_category = 'convert'
        elif 'edits' in path or old_cat == 'edits':
            new_category = 'edits'
        elif 'fileformats' in old_cat or 'fileformat' in features:
            new_category = 'fileformats'
        elif 'output' in old_cat:
            new_category = 'output'
        elif 'dedup' in features:
            new_category = 'dedup'
        elif 'filter' in features and old_cat == 'integration_tests':
            new_category = 'filter'
        elif 'trim' in features and old_cat == 'integration_tests':
            new_category = 'trim'
        elif old_cat == 'integration_tests':
            # Keep integration tests but organize better
            new_category = 'integration'
        else:
            new_category = old_cat

        # Build subcategory from remaining path
        path_parts = path.split('/')[1:]  # Remove old top-level category

        if new_category == 'error_handling':
            # Organize errors by what they're testing
            if 'bam' in path:
                subcat = 'bam'
            elif 'input' in path or 'read' in path or 'index' in path:
                subcat = 'input'
            elif 'output' in path:
                subcat = 'output'
            elif 'barcode' in path or 'demultiplex' in path:
                subcat = 'demultiplex'
            elif 'extract' in path or 'tag' in path:
                subcat = 'extraction'
            elif 'filter' in path:
                subcat = 'filter'
            elif 'segment' in path:
                subcat = 'segments'
            elif 'swap' in path:
                subcat = 'swap'
            elif 'compression' in path:
                subcat = 'compression'
            elif 'cock_et_al' in path and 'reject' in path:
                subcat = 'malformed_fastq'
            elif 'cock_et_al' in path and 'challenging' in path:
                subcat = 'format_edge_cases'
            elif 'report' in path:
                subcat = 'reports'
            elif 'eval' in path:
                subcat = 'eval'
            elif 'store_tag' in path:
                subcat = 'tag_storage'
            elif 'cli' in path or 'help' in path:
                subcat = 'cli'
            elif 'dna_validation' in path:
                subcat = 'dna_validation'
            else:
                subcat = 'misc'

            new_path = f"{new_category}/{subcat}/{'/'.join(path_parts)}"
        elif new_category == 'external_datasets':
            # Keep structure for cock_et_al
            new_path = f"{new_category}/{'/'.join(path_parts)}"
        else:
            new_path = f"{new_category}/{'/'.join(path_parts)}"

        suggestions.append({
            'old_path': path,
            'new_path': new_path,
            'reason': f"Category: {new_category}, Features: {', '.join(features[:3]) if features else 'general'}"
        })

    return suggestions

if __name__ == '__main__':
    print("Analyzing test cases...")
    analysis = analyze_all_tests()
    print(f"Found {len(analysis)} test cases\n")

    suggestions = suggest_new_organization(analysis)

    # Group by old category for easier review
    by_old_cat = defaultdict(list)
    for s in suggestions:
        old_cat = s['old_path'].split('/')[0]
        by_old_cat[old_cat].append(s)

    print("=" * 80)
    print("SUGGESTED REORGANIZATION")
    print("=" * 80)
    print()

    for old_cat in sorted(by_old_cat.keys()):
        items = by_old_cat[old_cat]
        print(f"\n{old_cat}/ ({len(items)} tests)")
        print("-" * 80)

        # Group by new top-level category
        by_new = defaultdict(list)
        for item in items:
            new_top = item['new_path'].split('/')[0]
            by_new[new_top].append(item)

        for new_cat in sorted(by_new.keys()):
            new_items = by_new[new_cat]
            print(f"  → {new_cat}/ ({len(new_items)} tests)")
            for item in new_items[:3]:  # Show first 3 examples
                print(f"      {item['old_path']} → {item['new_path']}")
            if len(new_items) > 3:
                print(f"      ... and {len(new_items) - 3} more")

    print("\n\n" + "=" * 80)
    print("SUMMARY OF NEW ORGANIZATION")
    print("=" * 80)

    by_new_top = defaultdict(int)
    for s in suggestions:
        new_top = s['new_path'].split('/')[0]
        by_new_top[new_top] += 1

    for cat in sorted(by_new_top.keys()):
        print(f"  {cat:30s} {by_new_top[cat]:4d} tests")

    print(f"\n  {'TOTAL':30s} {len(suggestions):4d} tests")
