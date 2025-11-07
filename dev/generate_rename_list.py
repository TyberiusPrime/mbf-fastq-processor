#!/usr/bin/env python3
"""
Generate rename list for test reorganization.
"""

from pathlib import Path
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

def suggest_new_path(test_path, config):
    """Suggest a new path for a test case."""
    path_str = str(test_path)
    parts = path_str.split('/')
    old_category = parts[0]

    # Determine if this is single-step or integration
    # Single-step: focused on testing one feature
    # Integration: tests multiple features or complex workflows

    is_single_step = True

    # Cock et al. tests - edge cases
    if 'cock_et_al' in path_str:
        if 'reject' in path_str:
            return f"single-step/error_handling/malformed_fastq/{'/'.join(parts[3:])}"
        else:
            return f"single-step/edge_cases/challenging_formats/{'/'.join(parts[3:])}"

    # Error handling and validation
    if old_category in ['input_validation', 'outside_error_conditions']:
        if 'bam' in path_str and ('missing' in path_str or 'disk_full' in path_str or 'output' in path_str):
            subcat = 'bam'
        elif any(x in path_str for x in ['read1', 'read2', 'index1', 'index2', 'input_file', 'missing', 'permission', 'repeated_filename']):
            subcat = 'input_files'
        elif 'output' in path_str or 'stdout' in path_str or 'segment' in path_str:
            subcat = 'output_config'
        elif 'barcode' in path_str or ('demultiplex' in path_str and old_category == 'input_validation'):
            subcat = 'demultiplex'
        elif any(x in path_str for x in ['extract', 'tag', 'umi', 'regex']):
            subcat = 'extraction'
        elif 'filter' in path_str:
            subcat = 'filter'
        elif 'swap' in path_str:
            subcat = 'swap'
        elif 'compression' in path_str or 'gzip' in path_str or 'zstd' in path_str:
            subcat = 'compression'
        elif 'convert_phred' in path_str:
            subcat = 'quality_scores'
        elif any(x in path_str for x in ['broken', 'truncated', 'newline', 'mismatched', 'quality', 'invalid_base']):
            subcat = 'malformed_fastq'
        elif 'cli' in path_str or 'help' in path_str or 'old_cli' in path_str:
            subcat = 'cli'
        elif 'dna_validation' in path_str or 'iupac' in path_str:
            subcat = 'dna_validation'
        elif 'paired_end' in path_str or 'read_pairing' in path_str:
            subcat = 'paired_end'
        elif 'report' in path_str and 'no_report' in path_str:
            subcat = 'reports'
        elif 'eval' in path_str or 'expr' in path_str:
            subcat = 'eval'
        elif 'store_tag' in path_str or 'store_tags' in path_str:
            subcat = 'tag_storage'
        elif 'interleaved' in path_str or 'stdin' in path_str:
            subcat = 'io'
        elif 'show_step_template' in path_str or 'two_mistakes' in path_str:
            subcat = 'error_messages'
        else:
            subcat = 'misc'
        return f"single-step/error_handling/{subcat}/{'/'.join(parts[1:])}"

    # Feature-based organization for working tests
    if old_category == 'demultiplex':
        return f"single-step/demultiplex/{'/'.join(parts[1:])}"

    if old_category == 'extraction':
        if 'demultiplex' in parts:
            return f"single-step/extraction/with_demultiplex/{'/'.join(parts[1:])}"
        return f"single-step/extraction/{'/'.join(parts[1:])}"

    if old_category == 'calc':
        return f"single-step/calc/{'/'.join(parts[1:])}"

    if old_category == 'convert':
        return f"single-step/convert/{'/'.join(parts[1:])}"

    if old_category == 'edits':
        return f"single-step/edits/{'/'.join(parts[1:])}"

    if old_category == 'fileformats':
        return f"single-step/fileformats/{'/'.join(parts[1:])}"

    if old_category == 'hamming_correct':
        return f"single-step/hamming/{'/'.join(parts[1:])}"

    if old_category == 'head_early_termination':
        return f"single-step/head/{'/'.join(parts[1:])}"

    if old_category == 'reports':
        return f"single-step/reports/{'/'.join(parts[1:])}"

    if old_category == 'memory':
        return f"single-step/performance/{'/'.join(parts[1:])}"

    if old_category == 'output':
        return f"single-step/output/{'/'.join(parts[1:])}"

    if old_category == 'validation':
        return f"single-step/validation/{'/'.join(parts[1:])}"

    # Handle integration_tests - split between single-step and integration
    if old_category == 'integration_tests':
        test_name = parts[-1]

        # Dedup tests
        if test_name.startswith('dedup') or 'dedup/' in path_str:
            rest = parts[1:] if parts[1] != 'dedup' else parts[2:]
            return f"single-step/dedup/{'/'.join(rest)}"

        # Filter tests
        if test_name.startswith('filter') or 'filter' in test_name:
            return f"single-step/filter/{'/'.join(parts[1:])}"

        # Trim tests
        if test_name.startswith('trim') or test_name.startswith('cut') or 'trim' in test_name or 'cut' in test_name:
            return f"single-step/trim/{'/'.join(parts[1:])}"

        # Quantify tests
        if test_name.startswith('quantify') or 'quantify' in test_name:
            return f"single-step/calc/{'/'.join(parts[1:])}"

        # Demultiplex tests
        if 'demultiplex' in test_name or 'head_with_index_and_demultiplex' in test_name:
            return f"single-step/demultiplex/{'/'.join(parts[1:])}"

        # BAM/external file tests
        if 'bam' in test_name or 'filter_other_file' in test_name:
            return f"single-step/fileformats/{'/'.join(parts[1:])}"

        # Expression evaluation
        if test_name.startswith('eval') or 'eval' in test_name or 'expr' in test_name:
            return f"single-step/eval/{'/'.join(parts[1:])}"

        # I/O format tests
        if 'interleaved' in test_name or 'stdin' in test_name or 'stdout' in test_name:
            return f"single-step/io/{'/'.join(parts[1:])}"

        # Hash validation
        if 'hash' in test_name:
            return f"single-step/validation/{'/'.join(parts[1:])}"

        # Compression tests
        if 'compress' in test_name or 'gzip' in test_name or 'zstd' in test_name or 'gz_input' in test_name:
            return f"single-step/compression/{'/'.join(parts[1:])}"

        # Inspect tests
        if 'inspect' in test_name or 'inspect/' in path_str:
            return f"single-step/inspect/{'/'.join(parts[1:])}"

        # Output tests
        if 'output' in test_name or 'output/' in path_str:
            return f"single-step/output/{'/'.join(parts[1:])}"

        # Extraction tests
        if 'extract_regex' in path_str or 'extract_iupac' in path_str:
            return f"single-step/extraction/{'/'.join(parts[1:])}"

        # Swap tests
        if 'swap' in test_name:
            return f"single-step/swap/{'/'.join(parts[1:])}"

        # Transform tests
        if 'reverse_complement' in test_name or 'rename' in test_name or 'prefix' in test_name or 'postfix' in test_name:
            return f"single-step/transform/{'/'.join(parts[1:])}"

        # Sampling tests
        if 'subsample' in test_name or 'skip' in test_name:
            return f"single-step/sampling/{'/'.join(parts[1:])}"

        # Quality tests
        if 'low_complexity' in test_name or 'quality_base' in test_name or 'convert_phred' in test_name:
            return f"single-step/quality/{'/'.join(parts[1:])}"

        # Max length
        if 'max_len' in test_name:
            return f"single-step/transform/{'/'.join(parts[1:])}"

        # Integration tests (complex multi-feature)
        if test_name in ['fastp_416', 'fastp_491', 'fastp_606', 'old_cli_format']:
            return f"integration/compatibility/{'/'.join(parts[1:])}"

        if test_name in ['ten_segments_creative_transforms', 'order_maintained_in_single_core_transforms']:
            return f"integration/complex/{'/'.join(parts[1:])}"

        if test_name in ['very_long_reads', 'mega_long_reads', 'gzip_blocks_spliting_reads']:
            return f"integration/edge_cases/{'/'.join(parts[1:])}"

        # Basic functionality - integration
        if test_name in ['noop', 'noop_minimal', 'allow_overwrites']:
            return f"integration/basic/{'/'.join(parts[1:])}"

        # Named pipes - integration
        if 'named_pipes' in path_str:
            return f"integration/io/{'/'.join(parts[1:])}"

        # Default remaining to integration
        return f"integration/misc/{'/'.join(parts[1:])}"

    # Fallback
    return path_str

def main():
    test_cases = find_all_test_cases()
    renames = []

    for test_path in test_cases:
        config = load_config(test_path)
        new_path = suggest_new_path(test_path, config)
        if str(test_path) != new_path:
            renames.append((str(test_path), new_path))

    # Write rename list
    with open('rename_list.txt', 'w') as f:
        for old_path, new_path in renames:
            f.write(f"{old_path}\n")
            f.write(f"{new_path}\n")
            f.write("\n")

    print(f"Generated rename_list.txt with {len(renames)} renames")

    # Print summary
    from collections import defaultdict
    by_new_top = defaultdict(int)
    by_old_top = defaultdict(lambda: defaultdict(int))

    for old_path, new_path in renames:
        old_top = old_path.split('/')[0]
        new_top = new_path.split('/')[0]
        by_new_top[new_top] += 1
        by_old_top[old_top][new_top] += 1

    print("\nNew structure:")
    for cat in sorted(by_new_top.keys()):
        print(f"  {cat:20s} {by_new_top[cat]:4d} tests")

if __name__ == '__main__':
    main()
