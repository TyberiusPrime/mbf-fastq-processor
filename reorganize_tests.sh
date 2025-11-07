#!/bin/bash
# Test case reorganization script
# Generated automatically - review before executing!
#
# This script renames test case directories according to the new organization.
# Review the changes carefully before running.
#
# Usage: bash reorganize_tests.sh
#

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Test Case Reorganization Script"
echo "================================"
echo ""
echo "This will reorganize 455 test cases."
echo ""
read -p "Are you sure you want to continue? (yes/no) " -r
echo
if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Aborting."
    exit 1
fi

cd test_cases


# calc/ (5 tests)
echo "Processing calc/..."


# convert/ (7 tests)
echo "Processing convert/..."


# demultiplex/ (17 tests)
echo "Processing demultiplex/..."


# edits/ (9 tests)
echo "Processing edits/..."


# extraction/ (76 tests)
echo "Processing extraction/..."

# Extraction with demultiplexing
if [ -d "extraction/store_tag_in_fastq/demultiplex" ]; then
    mkdir -p "$(dirname 'extraction/with_demultiplex/store_tag_in_fastq/demultiplex')"
    echo "  extraction/store_tag_in_fastq/demultiplex → extraction/with_demultiplex/store_tag_in_fastq/demultiplex"
    mv "extraction/store_tag_in_fastq/demultiplex" "extraction/with_demultiplex/store_tag_in_fastq/demultiplex"
fi


# fileformats/ (7 tests)
echo "Processing fileformats/..."


# hamming_correct/ (4 tests)
echo "Processing hamming_correct/..."

# Hamming distance correction: basic_correction
if [ -d "hamming_correct/basic_correction" ]; then
    mkdir -p "$(dirname 'hamming/basic_correction')"
    echo "  hamming_correct/basic_correction → hamming/basic_correction"
    mv "hamming_correct/basic_correction" "hamming/basic_correction"
fi

# Hamming distance correction: basic_correction_empty
if [ -d "hamming_correct/basic_correction_empty" ]; then
    mkdir -p "$(dirname 'hamming/basic_correction_empty')"
    echo "  hamming_correct/basic_correction_empty → hamming/basic_correction_empty"
    mv "hamming_correct/basic_correction_empty" "hamming/basic_correction_empty"
fi

# Hamming distance correction: basic_correction_keep
if [ -d "hamming_correct/basic_correction_keep" ]; then
    mkdir -p "$(dirname 'hamming/basic_correction_keep')"
    echo "  hamming_correct/basic_correction_keep → hamming/basic_correction_keep"
    mv "hamming_correct/basic_correction_keep" "hamming/basic_correction_keep"
fi

# Hamming distance correction: string_tag_correction
if [ -d "hamming_correct/string_tag_correction" ]; then
    mkdir -p "$(dirname 'hamming/string_tag_correction')"
    echo "  hamming_correct/string_tag_correction → hamming/string_tag_correction"
    mv "hamming_correct/string_tag_correction" "hamming/string_tag_correction"
fi


# head_early_termination/ (11 tests)
echo "Processing head_early_termination/..."

# Head/early termination: head_after_quantify
if [ -d "head_early_termination/head_after_quantify" ]; then
    mkdir -p "$(dirname 'head/head_after_quantify')"
    echo "  head_early_termination/head_after_quantify → head/head_after_quantify"
    mv "head_early_termination/head_after_quantify" "head/head_after_quantify"
fi

# Head/early termination: head_after_report
if [ -d "head_early_termination/head_after_report" ]; then
    mkdir -p "$(dirname 'head/head_after_report')"
    echo "  head_early_termination/head_after_report → head/head_after_report"
    mv "head_early_termination/head_after_report" "head/head_after_report"
fi

# Head/early termination: head_before_quantify
if [ -d "head_early_termination/head_before_quantify" ]; then
    mkdir -p "$(dirname 'head/head_before_quantify')"
    echo "  head_early_termination/head_before_quantify → head/head_before_quantify"
    mv "head_early_termination/head_before_quantify" "head/head_before_quantify"
fi

# Head/early termination: head_before_report
if [ -d "head_early_termination/head_before_report" ]; then
    mkdir -p "$(dirname 'head/head_before_report')"
    echo "  head_early_termination/head_before_report → head/head_before_report"
    mv "head_early_termination/head_before_report" "head/head_before_report"
fi

# Head/early termination: head_stops_reading
if [ -d "head_early_termination/head_stops_reading" ]; then
    mkdir -p "$(dirname 'head/head_stops_reading')"
    echo "  head_early_termination/head_stops_reading → head/head_stops_reading"
    mv "head_early_termination/head_stops_reading" "head/head_stops_reading"
fi

# Head/early termination: head_stops_reading_multiple
if [ -d "head_early_termination/head_stops_reading_multiple" ]; then
    mkdir -p "$(dirname 'head/head_stops_reading_multiple')"
    echo "  head_early_termination/head_stops_reading_multiple → head/head_stops_reading_multiple"
    mv "head_early_termination/head_stops_reading_multiple" "head/head_stops_reading_multiple"
fi

# Head/early termination: multi_stage_head
if [ -d "head_early_termination/multi_stage_head" ]; then
    mkdir -p "$(dirname 'head/multi_stage_head')"
    echo "  head_early_termination/multi_stage_head → head/multi_stage_head"
    mv "head_early_termination/multi_stage_head" "head/multi_stage_head"
fi

# Head/early termination: multi_stage_head_report_bottom
if [ -d "head_early_termination/multi_stage_head_report_bottom" ]; then
    mkdir -p "$(dirname 'head/multi_stage_head_report_bottom')"
    echo "  head_early_termination/multi_stage_head_report_bottom → head/multi_stage_head_report_bottom"
    mv "head_early_termination/multi_stage_head_report_bottom" "head/multi_stage_head_report_bottom"
fi

# Head/early termination: multi_stage_head_report_middle
if [ -d "head_early_termination/multi_stage_head_report_middle" ]; then
    mkdir -p "$(dirname 'head/multi_stage_head_report_middle')"
    echo "  head_early_termination/multi_stage_head_report_middle → head/multi_stage_head_report_middle"
    mv "head_early_termination/multi_stage_head_report_middle" "head/multi_stage_head_report_middle"
fi

# Head/early termination: multi_stage_head_report_middle_bottom
if [ -d "head_early_termination/multi_stage_head_report_middle_bottom" ]; then
    mkdir -p "$(dirname 'head/multi_stage_head_report_middle_bottom')"
    echo "  head_early_termination/multi_stage_head_report_middle_bottom → head/multi_stage_head_report_middle_bottom"
    mv "head_early_termination/multi_stage_head_report_middle_bottom" "head/multi_stage_head_report_middle_bottom"
fi

# Head/early termination: multi_stage_head_report_top
if [ -d "head_early_termination/multi_stage_head_report_top" ]; then
    mkdir -p "$(dirname 'head/multi_stage_head_report_top')"
    echo "  head_early_termination/multi_stage_head_report_top → head/multi_stage_head_report_top"
    mv "head_early_termination/multi_stage_head_report_top" "head/multi_stage_head_report_top"
fi


# input_validation/ (169 tests)
echo "Processing input_validation/..."

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/illumina/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/illumina/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/illumina/cat → edge_cases/challenging_formats/illumina/cat"
    mv "input_validation/cock_et_al_testdata/challenging/illumina/cat" "edge_cases/challenging_formats/illumina/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/illumina/to_sanger" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/illumina/to_sanger')"
    echo "  input_validation/cock_et_al_testdata/challenging/illumina/to_sanger → edge_cases/challenging_formats/illumina/to_sanger"
    mv "input_validation/cock_et_al_testdata/challenging/illumina/to_sanger" "edge_cases/challenging_formats/illumina/to_sanger"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/illumina/to_solexa" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/illumina/to_solexa')"
    echo "  input_validation/cock_et_al_testdata/challenging/illumina/to_solexa → edge_cases/challenging_formats/illumina/to_solexa"
    mv "input_validation/cock_et_al_testdata/challenging/illumina/to_solexa" "edge_cases/challenging_formats/illumina/to_solexa"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/longreads/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/longreads/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/longreads/cat → edge_cases/challenging_formats/longreads/cat"
    mv "input_validation/cock_et_al_testdata/challenging/longreads/cat" "edge_cases/challenging_formats/longreads/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_dna/as_illumina" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_dna/as_illumina')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_dna/as_illumina → edge_cases/challenging_formats/misc_dna/as_illumina"
    mv "input_validation/cock_et_al_testdata/challenging/misc_dna/as_illumina" "edge_cases/challenging_formats/misc_dna/as_illumina"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_dna/as_solexa" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_dna/as_solexa')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_dna/as_solexa → edge_cases/challenging_formats/misc_dna/as_solexa"
    mv "input_validation/cock_et_al_testdata/challenging/misc_dna/as_solexa" "edge_cases/challenging_formats/misc_dna/as_solexa"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_dna/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_dna/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_dna/cat → edge_cases/challenging_formats/misc_dna/cat"
    mv "input_validation/cock_et_al_testdata/challenging/misc_dna/cat" "edge_cases/challenging_formats/misc_dna/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_rna/as_illumina" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_rna/as_illumina')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_rna/as_illumina → edge_cases/challenging_formats/misc_rna/as_illumina"
    mv "input_validation/cock_et_al_testdata/challenging/misc_rna/as_illumina" "edge_cases/challenging_formats/misc_rna/as_illumina"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_rna/as_solexa" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_rna/as_solexa')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_rna/as_solexa → edge_cases/challenging_formats/misc_rna/as_solexa"
    mv "input_validation/cock_et_al_testdata/challenging/misc_rna/as_solexa" "edge_cases/challenging_formats/misc_rna/as_solexa"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/misc_rna/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/misc_rna/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/misc_rna/cat → edge_cases/challenging_formats/misc_rna/cat"
    mv "input_validation/cock_et_al_testdata/challenging/misc_rna/cat" "edge_cases/challenging_formats/misc_rna/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_illumina" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/sanger_full_range/as_illumina')"
    echo "  input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_illumina → edge_cases/challenging_formats/sanger_full_range/as_illumina"
    mv "input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_illumina" "edge_cases/challenging_formats/sanger_full_range/as_illumina"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_solexa" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/sanger_full_range/as_solexa')"
    echo "  input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_solexa → edge_cases/challenging_formats/sanger_full_range/as_solexa"
    mv "input_validation/cock_et_al_testdata/challenging/sanger_full_range/as_solexa" "edge_cases/challenging_formats/sanger_full_range/as_solexa"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/sanger_full_range/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/sanger_full_range/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/sanger_full_range/cat → edge_cases/challenging_formats/sanger_full_range/cat"
    mv "input_validation/cock_et_al_testdata/challenging/sanger_full_range/cat" "edge_cases/challenging_formats/sanger_full_range/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/solexa/as_illumina" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/solexa/as_illumina')"
    echo "  input_validation/cock_et_al_testdata/challenging/solexa/as_illumina → edge_cases/challenging_formats/solexa/as_illumina"
    mv "input_validation/cock_et_al_testdata/challenging/solexa/as_illumina" "edge_cases/challenging_formats/solexa/as_illumina"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/solexa/as_sanger" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/solexa/as_sanger')"
    echo "  input_validation/cock_et_al_testdata/challenging/solexa/as_sanger → edge_cases/challenging_formats/solexa/as_sanger"
    mv "input_validation/cock_et_al_testdata/challenging/solexa/as_sanger" "edge_cases/challenging_formats/solexa/as_sanger"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/solexa/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/solexa/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/solexa/cat → edge_cases/challenging_formats/solexa/cat"
    mv "input_validation/cock_et_al_testdata/challenging/solexa/cat" "edge_cases/challenging_formats/solexa/cat"
fi

# Cock et al. challenging format tests
if [ -d "input_validation/cock_et_al_testdata/challenging/wrapping/cat" ]; then
    mkdir -p "$(dirname 'edge_cases/challenging_formats/wrapping/cat')"
    echo "  input_validation/cock_et_al_testdata/challenging/wrapping/cat → edge_cases/challenging_formats/wrapping/cat"
    mv "input_validation/cock_et_al_testdata/challenging/wrapping/cat" "edge_cases/challenging_formats/wrapping/cat"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_diff_ids" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_diff_ids')"
    echo "  input_validation/cock_et_al_testdata/reject/error_diff_ids → error_handling/malformed_fastq/error_diff_ids"
    mv "input_validation/cock_et_al_testdata/reject/error_diff_ids" "error_handling/malformed_fastq/error_diff_ids"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_double_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_double_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_double_qual → error_handling/malformed_fastq/error_double_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_double_qual" "error_handling/malformed_fastq/error_double_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_double_seq" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_double_seq')"
    echo "  input_validation/cock_et_al_testdata/reject/error_double_seq → error_handling/malformed_fastq/error_double_seq"
    mv "input_validation/cock_et_al_testdata/reject/error_double_seq" "error_handling/malformed_fastq/error_double_seq"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_long_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_long_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_long_qual → error_handling/malformed_fastq/error_long_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_long_qual" "error_handling/malformed_fastq/error_long_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_no_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_no_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_no_qual → error_handling/malformed_fastq/error_no_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_no_qual" "error_handling/malformed_fastq/error_no_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_del" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_del')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_del → error_handling/malformed_fastq/error_qual_del"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_del" "error_handling/malformed_fastq/error_qual_del"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_escape" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_escape')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_escape → error_handling/malformed_fastq/error_qual_escape"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_escape" "error_handling/malformed_fastq/error_qual_escape"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_null" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_null')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_null → error_handling/malformed_fastq/error_qual_null"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_null" "error_handling/malformed_fastq/error_qual_null"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_space" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_space')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_space → error_handling/malformed_fastq/error_qual_space"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_space" "error_handling/malformed_fastq/error_qual_space"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_tab" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_tab')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_tab → error_handling/malformed_fastq/error_qual_tab"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_tab" "error_handling/malformed_fastq/error_qual_tab"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_unit_sep" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_unit_sep')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_unit_sep → error_handling/malformed_fastq/error_qual_unit_sep"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_unit_sep" "error_handling/malformed_fastq/error_qual_unit_sep"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_qual_vtab" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_qual_vtab')"
    echo "  input_validation/cock_et_al_testdata/reject/error_qual_vtab → error_handling/malformed_fastq/error_qual_vtab"
    mv "input_validation/cock_et_al_testdata/reject/error_qual_vtab" "error_handling/malformed_fastq/error_qual_vtab"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_short_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_short_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_short_qual → error_handling/malformed_fastq/error_short_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_short_qual" "error_handling/malformed_fastq/error_short_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_spaces" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_spaces')"
    echo "  input_validation/cock_et_al_testdata/reject/error_spaces → error_handling/malformed_fastq/error_spaces"
    mv "input_validation/cock_et_al_testdata/reject/error_spaces" "error_handling/malformed_fastq/error_spaces"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_tabs" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_tabs')"
    echo "  input_validation/cock_et_al_testdata/reject/error_tabs → error_handling/malformed_fastq/error_tabs"
    mv "input_validation/cock_et_al_testdata/reject/error_tabs" "error_handling/malformed_fastq/error_tabs"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_at_plus" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_at_plus')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_at_plus → error_handling/malformed_fastq/error_trunc_at_plus"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_at_plus" "error_handling/malformed_fastq/error_trunc_at_plus"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_at_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_at_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_at_qual → error_handling/malformed_fastq/error_trunc_at_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_at_qual" "error_handling/malformed_fastq/error_trunc_at_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_at_seq" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_at_seq')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_at_seq → error_handling/malformed_fastq/error_trunc_at_seq"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_at_seq" "error_handling/malformed_fastq/error_trunc_at_seq"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_in_plus" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_in_plus')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_in_plus → error_handling/malformed_fastq/error_trunc_in_plus"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_in_plus" "error_handling/malformed_fastq/error_trunc_in_plus"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_in_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_in_qual')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_in_qual → error_handling/malformed_fastq/error_trunc_in_qual"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_in_qual" "error_handling/malformed_fastq/error_trunc_in_qual"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_in_seq" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_in_seq')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_in_seq → error_handling/malformed_fastq/error_trunc_in_seq"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_in_seq" "error_handling/malformed_fastq/error_trunc_in_seq"
fi

# Cock et al. rejection tests (malformed FASTQ)
if [ -d "input_validation/cock_et_al_testdata/reject/error_trunc_in_title" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/error_trunc_in_title')"
    echo "  input_validation/cock_et_al_testdata/reject/error_trunc_in_title → error_handling/malformed_fastq/error_trunc_in_title"
    mv "input_validation/cock_et_al_testdata/reject/error_trunc_in_title" "error_handling/malformed_fastq/error_trunc_in_title"
fi

# Output configuration errors
if [ -d "input_validation/output/interleave/duplicated_target" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/output/interleave/duplicated_target')"
    echo "  input_validation/output/interleave/duplicated_target → error_handling/output_config/output/interleave/duplicated_target"
    mv "input_validation/output/interleave/duplicated_target" "error_handling/output_config/output/interleave/duplicated_target"
fi

# Output configuration errors
if [ -d "input_validation/output/interleave/just_one_target" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/output/interleave/just_one_target')"
    echo "  input_validation/output/interleave/just_one_target → error_handling/output_config/output/interleave/just_one_target"
    mv "input_validation/output/interleave/just_one_target" "error_handling/output_config/output/interleave/just_one_target"
fi

# Input file validation errors
if [ -d "input_validation/output/interleave/missing_target" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/output/interleave/missing_target')"
    echo "  input_validation/output/interleave/missing_target → error_handling/input_files/output/interleave/missing_target"
    mv "input_validation/output/interleave/missing_target" "error_handling/input_files/output/interleave/missing_target"
fi

# BAM file errors
if [ -d "input_validation/bam_missing_input_settings/both_false" ]; then
    mkdir -p "$(dirname 'error_handling/bam/bam_missing_input_settings/both_false')"
    echo "  input_validation/bam_missing_input_settings/both_false → error_handling/bam/bam_missing_input_settings/both_false"
    mv "input_validation/bam_missing_input_settings/both_false" "error_handling/bam/bam_missing_input_settings/both_false"
fi

# BAM file errors
if [ -d "input_validation/bam_missing_input_settings/mapped" ]; then
    mkdir -p "$(dirname 'error_handling/bam/bam_missing_input_settings/mapped')"
    echo "  input_validation/bam_missing_input_settings/mapped → error_handling/bam/bam_missing_input_settings/mapped"
    mv "input_validation/bam_missing_input_settings/mapped" "error_handling/bam/bam_missing_input_settings/mapped"
fi

# BAM file errors
if [ -d "input_validation/bam_missing_input_settings/unmapped" ]; then
    mkdir -p "$(dirname 'error_handling/bam/bam_missing_input_settings/unmapped')"
    echo "  input_validation/bam_missing_input_settings/unmapped → error_handling/bam/bam_missing_input_settings/unmapped"
    mv "input_validation/bam_missing_input_settings/unmapped" "error_handling/bam/bam_missing_input_settings/unmapped"
fi

# Demultiplexing validation errors
if [ -d "input_validation/barcodes/different_barcode_lengths" ]; then
    mkdir -p "$(dirname 'error_handling/demultiplex/barcodes/different_barcode_lengths')"
    echo "  input_validation/barcodes/different_barcode_lengths → error_handling/demultiplex/barcodes/different_barcode_lengths"
    mv "input_validation/barcodes/different_barcode_lengths" "error_handling/demultiplex/barcodes/different_barcode_lengths"
fi

# Demultiplexing validation errors
if [ -d "input_validation/barcodes/different_files" ]; then
    mkdir -p "$(dirname 'error_handling/demultiplex/barcodes/different_files')"
    echo "  input_validation/barcodes/different_files → error_handling/demultiplex/barcodes/different_files"
    mv "input_validation/barcodes/different_files" "error_handling/demultiplex/barcodes/different_files"
fi

# Demultiplexing validation errors
if [ -d "input_validation/barcodes/non_iupac" ]; then
    mkdir -p "$(dirname 'error_handling/demultiplex/barcodes/non_iupac')"
    echo "  input_validation/barcodes/non_iupac → error_handling/demultiplex/barcodes/non_iupac"
    mv "input_validation/barcodes/non_iupac" "error_handling/demultiplex/barcodes/non_iupac"
fi

# Demultiplexing validation errors
if [ -d "input_validation/barcodes/same_files" ]; then
    mkdir -p "$(dirname 'error_handling/demultiplex/barcodes/same_files')"
    echo "  input_validation/barcodes/same_files → error_handling/demultiplex/barcodes/same_files"
    mv "input_validation/barcodes/same_files" "error_handling/demultiplex/barcodes/same_files"
fi

# Extraction/tag validation errors
if [ -d "input_validation/eval_expr/len_from_not_a_len_tag" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/eval_expr/len_from_not_a_len_tag')"
    echo "  input_validation/eval_expr/len_from_not_a_len_tag → error_handling/extraction/eval_expr/len_from_not_a_len_tag"
    mv "input_validation/eval_expr/len_from_not_a_len_tag" "error_handling/extraction/eval_expr/len_from_not_a_len_tag"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_regex/from_name_followed_by_uppercase" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_regex/from_name_followed_by_uppercase')"
    echo "  input_validation/extract_regex/from_name_followed_by_uppercase → error_handling/extraction/extract_regex/from_name_followed_by_uppercase"
    mv "input_validation/extract_regex/from_name_followed_by_uppercase" "error_handling/extraction/extract_regex/from_name_followed_by_uppercase"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_regex/label_starts_with_name" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_regex/label_starts_with_name')"
    echo "  input_validation/extract_regex/label_starts_with_name → error_handling/extraction/extract_regex/label_starts_with_name"
    mv "input_validation/extract_regex/label_starts_with_name" "error_handling/extraction/extract_regex/label_starts_with_name"
fi

# Output configuration errors
if [ -d "input_validation/extract_regex/name_invalid_segment" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/extract_regex/name_invalid_segment')"
    echo "  input_validation/extract_regex/name_invalid_segment → error_handling/output_config/extract_regex/name_invalid_segment"
    mv "input_validation/extract_regex/name_invalid_segment" "error_handling/output_config/extract_regex/name_invalid_segment"
fi

# Output configuration errors
if [ -d "input_validation/extract_regex/name_no_segment_specified" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/extract_regex/name_no_segment_specified')"
    echo "  input_validation/extract_regex/name_no_segment_specified → error_handling/output_config/extract_regex/name_no_segment_specified"
    mv "input_validation/extract_regex/name_no_segment_specified" "error_handling/output_config/extract_regex/name_no_segment_specified"
fi

# Compression validation errors
if [ -d "input_validation/invalid_compression_levels/inspect_gzip_level_too_high" ]; then
    mkdir -p "$(dirname 'error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high')"
    echo "  input_validation/invalid_compression_levels/inspect_gzip_level_too_high → error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high"
    mv "input_validation/invalid_compression_levels/inspect_gzip_level_too_high" "error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high"
fi

# Compression validation errors
if [ -d "input_validation/invalid_compression_levels/inspect_zstd_level_zero" ]; then
    mkdir -p "$(dirname 'error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero')"
    echo "  input_validation/invalid_compression_levels/inspect_zstd_level_zero → error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero"
    mv "input_validation/invalid_compression_levels/inspect_zstd_level_zero" "error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero"
fi

# Output configuration errors
if [ -d "input_validation/invalid_compression_levels/output_gzip_level_too_high" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high')"
    echo "  input_validation/invalid_compression_levels/output_gzip_level_too_high → error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high"
    mv "input_validation/invalid_compression_levels/output_gzip_level_too_high" "error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high"
fi

# Output configuration errors
if [ -d "input_validation/invalid_compression_levels/output_zstd_level_too_high" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high')"
    echo "  input_validation/invalid_compression_levels/output_zstd_level_too_high → error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high"
    mv "input_validation/invalid_compression_levels/output_zstd_level_too_high" "error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high"
fi

# Output configuration errors
if [ -d "input_validation/invalid_compression_levels/output_zstd_level_zero" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/invalid_compression_levels/output_zstd_level_zero')"
    echo "  input_validation/invalid_compression_levels/output_zstd_level_zero → error_handling/output_config/invalid_compression_levels/output_zstd_level_zero"
    mv "input_validation/invalid_compression_levels/output_zstd_level_zero" "error_handling/output_config/invalid_compression_levels/output_zstd_level_zero"
fi

# Compression validation errors
if [ -d "input_validation/invalid_compression_levels/raw_with_compression_level" ]; then
    mkdir -p "$(dirname 'error_handling/compression/invalid_compression_levels/raw_with_compression_level')"
    echo "  input_validation/invalid_compression_levels/raw_with_compression_level → error_handling/compression/invalid_compression_levels/raw_with_compression_level"
    mv "input_validation/invalid_compression_levels/raw_with_compression_level" "error_handling/compression/invalid_compression_levels/raw_with_compression_level"
fi

# Output configuration errors
if [ -d "input_validation/invalid_segment_names/all" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/invalid_segment_names/all')"
    echo "  input_validation/invalid_segment_names/all → error_handling/output_config/invalid_segment_names/all"
    mv "input_validation/invalid_segment_names/all" "error_handling/output_config/invalid_segment_names/all"
fi

# Output configuration errors
if [ -d "input_validation/invalid_segment_names/internal" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/invalid_segment_names/internal')"
    echo "  input_validation/invalid_segment_names/internal → error_handling/output_config/invalid_segment_names/internal"
    mv "input_validation/invalid_segment_names/internal" "error_handling/output_config/invalid_segment_names/internal"
fi

# Output configuration errors
if [ -d "input_validation/no_output_no_reports/empty_output" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/no_output_no_reports/empty_output')"
    echo "  input_validation/no_output_no_reports/empty_output → error_handling/output_config/no_output_no_reports/empty_output"
    mv "input_validation/no_output_no_reports/empty_output" "error_handling/output_config/no_output_no_reports/empty_output"
fi

# Output configuration errors
if [ -d "input_validation/no_output_no_reports/format_raw" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/no_output_no_reports/format_raw')"
    echo "  input_validation/no_output_no_reports/format_raw → error_handling/output_config/no_output_no_reports/format_raw"
    mv "input_validation/no_output_no_reports/format_raw" "error_handling/output_config/no_output_no_reports/format_raw"
fi

# Output configuration errors
if [ -d "input_validation/output/chunked_fifo" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/output/chunked_fifo')"
    echo "  input_validation/output/chunked_fifo → error_handling/output_config/output/chunked_fifo"
    mv "input_validation/output/chunked_fifo" "error_handling/output_config/output/chunked_fifo"
fi

# Output configuration errors
if [ -d "input_validation/output/chunked_stdout" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/output/chunked_stdout')"
    echo "  input_validation/output/chunked_stdout → error_handling/output_config/output/chunked_stdout"
    mv "input_validation/output/chunked_stdout" "error_handling/output_config/output/chunked_stdout"
fi

# Input file validation errors
if [ -d "input_validation/paired_end_unqueal_read_count/read1_more_than_read2" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2')"
    echo "  input_validation/paired_end_unqueal_read_count/read1_more_than_read2 → error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2"
    mv "input_validation/paired_end_unqueal_read_count/read1_more_than_read2" "error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2"
fi

# Input file validation errors
if [ -d "input_validation/paired_end_unqueal_read_count/read2_more_than_read1" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1')"
    echo "  input_validation/paired_end_unqueal_read_count/read2_more_than_read1 → error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1"
    mv "input_validation/paired_end_unqueal_read_count/read2_more_than_read1" "error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1"
fi

# Paired-end validation errors
if [ -d "input_validation/paired_end_unqueal_read_count/read3_more_than_1_2" ]; then
    mkdir -p "$(dirname 'error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2')"
    echo "  input_validation/paired_end_unqueal_read_count/read3_more_than_1_2 → error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2"
    mv "input_validation/paired_end_unqueal_read_count/read3_more_than_1_2" "error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2"
fi

# Extraction/tag validation errors
if [ -d "input_validation/store_tag_in_comment/insert_char_in_value" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/store_tag_in_comment/insert_char_in_value')"
    echo "  input_validation/store_tag_in_comment/insert_char_in_value → error_handling/extraction/store_tag_in_comment/insert_char_in_value"
    mv "input_validation/store_tag_in_comment/insert_char_in_value" "error_handling/extraction/store_tag_in_comment/insert_char_in_value"
fi

# Extraction/tag validation errors
if [ -d "input_validation/store_tag_in_comment/seperator_in_label" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/store_tag_in_comment/seperator_in_label')"
    echo "  input_validation/store_tag_in_comment/seperator_in_label → error_handling/extraction/store_tag_in_comment/seperator_in_label"
    mv "input_validation/store_tag_in_comment/seperator_in_label" "error_handling/extraction/store_tag_in_comment/seperator_in_label"
fi

# Extraction/tag validation errors
if [ -d "input_validation/store_tag_in_comment/seperator_in_value" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/store_tag_in_comment/seperator_in_value')"
    echo "  input_validation/store_tag_in_comment/seperator_in_value → error_handling/extraction/store_tag_in_comment/seperator_in_value"
    mv "input_validation/store_tag_in_comment/seperator_in_value" "error_handling/extraction/store_tag_in_comment/seperator_in_value"
fi

# Extraction/tag validation errors
if [ -d "input_validation/store_tags_in_table/same_infix_twice" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/store_tags_in_table/same_infix_twice')"
    echo "  input_validation/store_tags_in_table/same_infix_twice → error_handling/extraction/store_tags_in_table/same_infix_twice"
    mv "input_validation/store_tags_in_table/same_infix_twice" "error_handling/extraction/store_tags_in_table/same_infix_twice"
fi

# Extraction/tag validation errors
if [ -d "input_validation/store_tags_in_table/store_tags_in_table_no_tags_defined" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined')"
    echo "  input_validation/store_tags_in_table/store_tags_in_table_no_tags_defined → error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined"
    mv "input_validation/store_tags_in_table/store_tags_in_table_no_tags_defined" "error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined"
fi

# Output configuration errors
if [ -d "input_validation/swap/swap_auto_detect_too_few_segments" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/swap/swap_auto_detect_too_few_segments')"
    echo "  input_validation/swap/swap_auto_detect_too_few_segments → error_handling/output_config/swap/swap_auto_detect_too_few_segments"
    mv "input_validation/swap/swap_auto_detect_too_few_segments" "error_handling/output_config/swap/swap_auto_detect_too_few_segments"
fi

# Output configuration errors
if [ -d "input_validation/swap/swap_auto_detect_too_many_segments" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/swap/swap_auto_detect_too_many_segments')"
    echo "  input_validation/swap/swap_auto_detect_too_many_segments → error_handling/output_config/swap/swap_auto_detect_too_many_segments"
    mv "input_validation/swap/swap_auto_detect_too_many_segments" "error_handling/output_config/swap/swap_auto_detect_too_many_segments"
fi

# Input file validation errors
if [ -d "input_validation/swap/swap_missing_segment_a" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/swap/swap_missing_segment_a')"
    echo "  input_validation/swap/swap_missing_segment_a → error_handling/input_files/swap/swap_missing_segment_a"
    mv "input_validation/swap/swap_missing_segment_a" "error_handling/input_files/swap/swap_missing_segment_a"
fi

# Input file validation errors
if [ -d "input_validation/swap/swap_missing_segment_b" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/swap/swap_missing_segment_b')"
    echo "  input_validation/swap/swap_missing_segment_b → error_handling/input_files/swap/swap_missing_segment_b"
    mv "input_validation/swap/swap_missing_segment_b" "error_handling/input_files/swap/swap_missing_segment_b"
fi

# Swap validation errors
if [ -d "input_validation/swap/swap_partial_specification_a_only" ]; then
    mkdir -p "$(dirname 'error_handling/swap/swap/swap_partial_specification_a_only')"
    echo "  input_validation/swap/swap_partial_specification_a_only → error_handling/swap/swap/swap_partial_specification_a_only"
    mv "input_validation/swap/swap_partial_specification_a_only" "error_handling/swap/swap/swap_partial_specification_a_only"
fi

# Swap validation errors
if [ -d "input_validation/swap/swap_partial_specification_b_only" ]; then
    mkdir -p "$(dirname 'error_handling/swap/swap/swap_partial_specification_b_only')"
    echo "  input_validation/swap/swap_partial_specification_b_only → error_handling/swap/swap/swap_partial_specification_b_only"
    mv "input_validation/swap/swap_partial_specification_b_only" "error_handling/swap/swap/swap_partial_specification_b_only"
fi

# Output configuration errors
if [ -d "input_validation/swap/swap_same_segment" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/swap/swap_same_segment')"
    echo "  input_validation/swap/swap_same_segment → error_handling/output_config/swap/swap_same_segment"
    mv "input_validation/swap/swap_same_segment" "error_handling/output_config/swap/swap_same_segment"
fi

# Extraction/tag validation errors
if [ -d "input_validation/tag_name/tag_name_not_len" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/tag_name/tag_name_not_len')"
    echo "  input_validation/tag_name/tag_name_not_len → error_handling/extraction/tag_name/tag_name_not_len"
    mv "input_validation/tag_name/tag_name_not_len" "error_handling/extraction/tag_name/tag_name_not_len"
fi

# BAM file errors
if [ -d "input_validation/bam_output_uncompressed_hash" ]; then
    mkdir -p "$(dirname 'error_handling/bam/bam_output_uncompressed_hash')"
    echo "  input_validation/bam_output_uncompressed_hash → error_handling/bam/bam_output_uncompressed_hash"
    mv "input_validation/bam_output_uncompressed_hash" "error_handling/bam/bam_output_uncompressed_hash"
fi

# Output configuration errors
if [ -d "input_validation/barcode_outputs_not_named_no_barcode" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/barcode_outputs_not_named_no_barcode')"
    echo "  input_validation/barcode_outputs_not_named_no_barcode → error_handling/output_config/barcode_outputs_not_named_no_barcode"
    mv "input_validation/barcode_outputs_not_named_no_barcode" "error_handling/output_config/barcode_outputs_not_named_no_barcode"
fi

# Malformed FASTQ files
if [ -d "input_validation/broken_newline" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/broken_newline')"
    echo "  input_validation/broken_newline → error_handling/malformed_fastq/broken_newline"
    mv "input_validation/broken_newline" "error_handling/malformed_fastq/broken_newline"
fi

# Malformed FASTQ files
if [ -d "input_validation/broken_newline2" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/broken_newline2')"
    echo "  input_validation/broken_newline2 → error_handling/malformed_fastq/broken_newline2"
    mv "input_validation/broken_newline2" "error_handling/malformed_fastq/broken_newline2"
fi

# Malformed FASTQ files
if [ -d "input_validation/broken_panics" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/broken_panics')"
    echo "  input_validation/broken_panics → error_handling/malformed_fastq/broken_panics"
    mv "input_validation/broken_panics" "error_handling/malformed_fastq/broken_panics"
fi

# CLI validation errors
if [ -d "input_validation/cli_invalid_command" ]; then
    mkdir -p "$(dirname 'error_handling/cli/cli_invalid_command')"
    echo "  input_validation/cli_invalid_command → error_handling/cli/cli_invalid_command"
    mv "input_validation/cli_invalid_command" "error_handling/cli/cli_invalid_command"
fi

# Compression validation errors
if [ -d "input_validation/compression_detection_wrong_extension" ]; then
    mkdir -p "$(dirname 'error_handling/compression/compression_detection_wrong_extension')"
    echo "  input_validation/compression_detection_wrong_extension → error_handling/compression/compression_detection_wrong_extension"
    mv "input_validation/compression_detection_wrong_extension" "error_handling/compression/compression_detection_wrong_extension"
fi

# Quality score conversion errors
if [ -d "input_validation/convert_phred_raises" ]; then
    mkdir -p "$(dirname 'error_handling/quality_scores/convert_phred_raises')"
    echo "  input_validation/convert_phred_raises → error_handling/quality_scores/convert_phred_raises"
    mv "input_validation/convert_phred_raises" "error_handling/quality_scores/convert_phred_raises"
fi

# DNA validation errors
if [ -d "input_validation/dna_validation_count_oligos_non_agtc" ]; then
    mkdir -p "$(dirname 'error_handling/dna_validation/dna_validation_count_oligos_non_agtc')"
    echo "  input_validation/dna_validation_count_oligos_non_agtc → error_handling/dna_validation/dna_validation_count_oligos_non_agtc"
    mv "input_validation/dna_validation_count_oligos_non_agtc" "error_handling/dna_validation/dna_validation_count_oligos_non_agtc"
fi

# DNA validation errors
if [ -d "input_validation/dna_validation_count_oligos_non_empty" ]; then
    mkdir -p "$(dirname 'error_handling/dna_validation/dna_validation_count_oligos_non_empty')"
    echo "  input_validation/dna_validation_count_oligos_non_empty → error_handling/dna_validation/dna_validation_count_oligos_non_empty"
    mv "input_validation/dna_validation_count_oligos_non_empty" "error_handling/dna_validation/dna_validation_count_oligos_non_empty"
fi

# Miscellaneous validation errors
if [ -d "input_validation/empty_list_of_files" ]; then
    mkdir -p "$(dirname 'error_handling/misc/empty_list_of_files')"
    echo "  input_validation/empty_list_of_files → error_handling/misc/empty_list_of_files"
    mv "input_validation/empty_list_of_files" "error_handling/misc/empty_list_of_files"
fi

# Miscellaneous validation errors
if [ -d "input_validation/empty_name_input" ]; then
    mkdir -p "$(dirname 'error_handling/misc/empty_name_input')"
    echo "  input_validation/empty_name_input → error_handling/misc/empty_name_input"
    mv "input_validation/empty_name_input" "error_handling/misc/empty_name_input"
fi

# Output configuration errors
if [ -d "input_validation/empty_output" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/empty_output')"
    echo "  input_validation/empty_output → error_handling/output_config/empty_output"
    mv "input_validation/empty_output" "error_handling/output_config/empty_output"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_base_content_absolute_with_ignore" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_base_content_absolute_with_ignore')"
    echo "  input_validation/extract_base_content_absolute_with_ignore → error_handling/extraction/extract_base_content_absolute_with_ignore"
    mv "input_validation/extract_base_content_absolute_with_ignore" "error_handling/extraction/extract_base_content_absolute_with_ignore"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_base_content_empty_count" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_base_content_empty_count')"
    echo "  input_validation/extract_base_content_empty_count → error_handling/extraction/extract_base_content_empty_count"
    mv "input_validation/extract_base_content_empty_count" "error_handling/extraction/extract_base_content_empty_count"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_base_content_invalid_letters" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_base_content_invalid_letters')"
    echo "  input_validation/extract_base_content_invalid_letters → error_handling/extraction/extract_base_content_invalid_letters"
    mv "input_validation/extract_base_content_invalid_letters" "error_handling/extraction/extract_base_content_invalid_letters"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_gc_panic_on_store_in_seq" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_gc_panic_on_store_in_seq')"
    echo "  input_validation/extract_gc_panic_on_store_in_seq → error_handling/extraction/extract_gc_panic_on_store_in_seq"
    mv "input_validation/extract_gc_panic_on_store_in_seq" "error_handling/extraction/extract_gc_panic_on_store_in_seq"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_iupac_suffix_min_length_too_high" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_iupac_suffix_min_length_too_high')"
    echo "  input_validation/extract_iupac_suffix_min_length_too_high → error_handling/extraction/extract_iupac_suffix_min_length_too_high"
    mv "input_validation/extract_iupac_suffix_min_length_too_high" "error_handling/extraction/extract_iupac_suffix_min_length_too_high"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_iupac_suffix_too_many_mismatches" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_iupac_suffix_too_many_mismatches')"
    echo "  input_validation/extract_iupac_suffix_too_many_mismatches → error_handling/extraction/extract_iupac_suffix_too_many_mismatches"
    mv "input_validation/extract_iupac_suffix_too_many_mismatches" "error_handling/extraction/extract_iupac_suffix_too_many_mismatches"
fi

# Extraction/tag validation errors
if [ -d "input_validation/extract_tag_from_i1_i2_no_i1_i2" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2')"
    echo "  input_validation/extract_tag_from_i1_i2_no_i1_i2 → error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2"
    mv "input_validation/extract_tag_from_i1_i2_no_i1_i2" "error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2"
fi

# Output configuration errors
if [ -d "input_validation/extract_tag_i1_i2_but_not_output" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/extract_tag_i1_i2_but_not_output')"
    echo "  input_validation/extract_tag_i1_i2_but_not_output → error_handling/output_config/extract_tag_i1_i2_but_not_output"
    mv "input_validation/extract_tag_i1_i2_but_not_output" "error_handling/output_config/extract_tag_i1_i2_but_not_output"
fi

# Input file validation errors
if [ -d "input_validation/fake_fasta_missing" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/fake_fasta_missing')"
    echo "  input_validation/fake_fasta_missing → error_handling/input_files/fake_fasta_missing"
    mv "input_validation/fake_fasta_missing" "error_handling/input_files/fake_fasta_missing"
fi

# Extraction/tag validation errors
if [ -d "input_validation/filter_by_tag_numeric_rejection" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/filter_by_tag_numeric_rejection')"
    echo "  input_validation/filter_by_tag_numeric_rejection → error_handling/extraction/filter_by_tag_numeric_rejection"
    mv "input_validation/filter_by_tag_numeric_rejection" "error_handling/extraction/filter_by_tag_numeric_rejection"
fi

# Input file validation errors
if [ -d "input_validation/filter_missing_tag" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/filter_missing_tag')"
    echo "  input_validation/filter_missing_tag → error_handling/input_files/filter_missing_tag"
    mv "input_validation/filter_missing_tag" "error_handling/input_files/filter_missing_tag"
fi

# Extraction/tag validation errors
if [ -d "input_validation/filter_no_such_tag" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/filter_no_such_tag')"
    echo "  input_validation/filter_no_such_tag → error_handling/extraction/filter_no_such_tag"
    mv "input_validation/filter_no_such_tag" "error_handling/extraction/filter_no_such_tag"
fi

# CLI validation errors
if [ -d "input_validation/help" ]; then
    mkdir -p "$(dirname 'error_handling/cli/help')"
    echo "  input_validation/help → error_handling/cli/help"
    mv "input_validation/help" "error_handling/cli/help"
fi

# Input file validation errors
if [ -d "input_validation/index1_file_does_not_exist" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/index1_file_does_not_exist')"
    echo "  input_validation/index1_file_does_not_exist → error_handling/input_files/index1_file_does_not_exist"
    mv "input_validation/index1_file_does_not_exist" "error_handling/input_files/index1_file_does_not_exist"
fi

# Input file validation errors
if [ -d "input_validation/index2_file_does_not_exist" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/index2_file_does_not_exist')"
    echo "  input_validation/index2_file_does_not_exist → error_handling/input_files/index2_file_does_not_exist"
    mv "input_validation/index2_file_does_not_exist" "error_handling/input_files/index2_file_does_not_exist"
fi

# Input file validation errors
if [ -d "input_validation/input_file_is_output_file" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/input_file_is_output_file')"
    echo "  input_validation/input_file_is_output_file → error_handling/input_files/input_file_is_output_file"
    mv "input_validation/input_file_is_output_file" "error_handling/input_files/input_file_is_output_file"
fi

# Output configuration errors
if [ -d "input_validation/input_interleaved_multiple_segment_files" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/input_interleaved_multiple_segment_files')"
    echo "  input_validation/input_interleaved_multiple_segment_files → error_handling/output_config/input_interleaved_multiple_segment_files"
    mv "input_validation/input_interleaved_multiple_segment_files" "error_handling/output_config/input_interleaved_multiple_segment_files"
fi

# Malformed FASTQ files
if [ -d "input_validation/invalid_base" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/invalid_base')"
    echo "  input_validation/invalid_base → error_handling/malformed_fastq/invalid_base"
    mv "input_validation/invalid_base" "error_handling/malformed_fastq/invalid_base"
fi

# Malformed FASTQ files
if [ -d "input_validation/invalid_base_or_dot" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/invalid_base_or_dot')"
    echo "  input_validation/invalid_base_or_dot → error_handling/malformed_fastq/invalid_base_or_dot"
    mv "input_validation/invalid_base_or_dot" "error_handling/malformed_fastq/invalid_base_or_dot"
fi

# Malformed FASTQ files
if [ -d "input_validation/invalid_base_or_dot_too_long" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/invalid_base_or_dot_too_long')"
    echo "  input_validation/invalid_base_or_dot_too_long → error_handling/malformed_fastq/invalid_base_or_dot_too_long"
    mv "input_validation/invalid_base_or_dot_too_long" "error_handling/malformed_fastq/invalid_base_or_dot_too_long"
fi

# Malformed FASTQ files
if [ -d "input_validation/mismatched_seq_qual_len_1st_read_qual_too_long" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long')"
    echo "  input_validation/mismatched_seq_qual_len_1st_read_qual_too_long → error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long"
    mv "input_validation/mismatched_seq_qual_len_1st_read_qual_too_long" "error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long"
fi

# Malformed FASTQ files
if [ -d "input_validation/mismatched_seq_qual_len_1st_read_qual_too_short" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short')"
    echo "  input_validation/mismatched_seq_qual_len_1st_read_qual_too_short → error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short"
    mv "input_validation/mismatched_seq_qual_len_1st_read_qual_too_short" "error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short"
fi

# Malformed FASTQ files
if [ -d "input_validation/mismatched_seq_qual_len_2nd_read_qual_too_long" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long')"
    echo "  input_validation/mismatched_seq_qual_len_2nd_read_qual_too_long → error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long"
    mv "input_validation/mismatched_seq_qual_len_2nd_read_qual_too_long" "error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long"
fi

# Malformed FASTQ files
if [ -d "input_validation/mismatched_seq_qual_len_2nd_read_qual_too_short" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short')"
    echo "  input_validation/mismatched_seq_qual_len_2nd_read_qual_too_short → error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short"
    mv "input_validation/mismatched_seq_qual_len_2nd_read_qual_too_short" "error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short"
fi

# Input file validation errors
if [ -d "input_validation/missing_input_file" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/missing_input_file')"
    echo "  input_validation/missing_input_file → error_handling/input_files/missing_input_file"
    mv "input_validation/missing_input_file" "error_handling/input_files/missing_input_file"
fi

# Miscellaneous validation errors
if [ -d "input_validation/mixed_input_formats" ]; then
    mkdir -p "$(dirname 'error_handling/misc/mixed_input_formats')"
    echo "  input_validation/mixed_input_formats → error_handling/misc/mixed_input_formats"
    mv "input_validation/mixed_input_formats" "error_handling/misc/mixed_input_formats"
fi

# Malformed FASTQ files
if [ -d "input_validation/no_newline_and_truncated_qual" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/no_newline_and_truncated_qual')"
    echo "  input_validation/no_newline_and_truncated_qual → error_handling/malformed_fastq/no_newline_and_truncated_qual"
    mv "input_validation/no_newline_and_truncated_qual" "error_handling/malformed_fastq/no_newline_and_truncated_qual"
fi

# Malformed FASTQ files
if [ -d "input_validation/no_newline_at_end_ok" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/no_newline_at_end_ok')"
    echo "  input_validation/no_newline_at_end_ok → error_handling/malformed_fastq/no_newline_at_end_ok"
    mv "input_validation/no_newline_at_end_ok" "error_handling/malformed_fastq/no_newline_at_end_ok"
fi

# Output configuration errors
if [ -d "input_validation/no_segments" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/no_segments')"
    echo "  input_validation/no_segments → error_handling/output_config/no_segments"
    mv "input_validation/no_segments" "error_handling/output_config/no_segments"
fi

# Extraction/tag validation errors
if [ -d "input_validation/numeric_filter_wrong_tag_type" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/numeric_filter_wrong_tag_type')"
    echo "  input_validation/numeric_filter_wrong_tag_type → error_handling/extraction/numeric_filter_wrong_tag_type"
    mv "input_validation/numeric_filter_wrong_tag_type" "error_handling/extraction/numeric_filter_wrong_tag_type"
fi

# CLI validation errors
if [ -d "input_validation/old_cli_not_existant_file" ]; then
    mkdir -p "$(dirname 'error_handling/cli/old_cli_not_existant_file')"
    echo "  input_validation/old_cli_not_existant_file → error_handling/cli/old_cli_not_existant_file"
    mv "input_validation/old_cli_not_existant_file" "error_handling/cli/old_cli_not_existant_file"
fi

# Input file validation errors
if [ -d "input_validation/permission_denied_input_file" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/permission_denied_input_file')"
    echo "  input_validation/permission_denied_input_file → error_handling/input_files/permission_denied_input_file"
    mv "input_validation/permission_denied_input_file" "error_handling/input_files/permission_denied_input_file"
fi

# Input file validation errors
if [ -d "input_validation/permission_denied_read1" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/permission_denied_read1')"
    echo "  input_validation/permission_denied_read1 → error_handling/input_files/permission_denied_read1"
    mv "input_validation/permission_denied_read1" "error_handling/input_files/permission_denied_read1"
fi

# Miscellaneous validation errors
if [ -d "input_validation/postfix_len_mismatch" ]; then
    mkdir -p "$(dirname 'error_handling/misc/postfix_len_mismatch')"
    echo "  input_validation/postfix_len_mismatch → error_handling/misc/postfix_len_mismatch"
    mv "input_validation/postfix_len_mismatch" "error_handling/misc/postfix_len_mismatch"
fi

# Miscellaneous validation errors
if [ -d "input_validation/prefix_len_mismatch" ]; then
    mkdir -p "$(dirname 'error_handling/misc/prefix_len_mismatch')"
    echo "  input_validation/prefix_len_mismatch → error_handling/misc/prefix_len_mismatch"
    mv "input_validation/prefix_len_mismatch" "error_handling/misc/prefix_len_mismatch"
fi

# Malformed FASTQ files
if [ -d "input_validation/quality_starts_with_at" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/quality_starts_with_at')"
    echo "  input_validation/quality_starts_with_at → error_handling/malformed_fastq/quality_starts_with_at"
    mv "input_validation/quality_starts_with_at" "error_handling/malformed_fastq/quality_starts_with_at"
fi

# Input file validation errors
if [ -d "input_validation/read1_empty_list" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read1_empty_list')"
    echo "  input_validation/read1_empty_list → error_handling/input_files/read1_empty_list"
    mv "input_validation/read1_empty_list" "error_handling/input_files/read1_empty_list"
fi

# Input file validation errors
if [ -d "input_validation/read1_len_neq_index1_len" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read1_len_neq_index1_len')"
    echo "  input_validation/read1_len_neq_index1_len → error_handling/input_files/read1_len_neq_index1_len"
    mv "input_validation/read1_len_neq_index1_len" "error_handling/input_files/read1_len_neq_index1_len"
fi

# Input file validation errors
if [ -d "input_validation/read1_len_neq_index2_len" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read1_len_neq_index2_len')"
    echo "  input_validation/read1_len_neq_index2_len → error_handling/input_files/read1_len_neq_index2_len"
    mv "input_validation/read1_len_neq_index2_len" "error_handling/input_files/read1_len_neq_index2_len"
fi

# Input file validation errors
if [ -d "input_validation/read1_len_neq_read2_len" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read1_len_neq_read2_len')"
    echo "  input_validation/read1_len_neq_read2_len → error_handling/input_files/read1_len_neq_read2_len"
    mv "input_validation/read1_len_neq_read2_len" "error_handling/input_files/read1_len_neq_read2_len"
fi

# Input file validation errors
if [ -d "input_validation/read1_not_a_string" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read1_not_a_string')"
    echo "  input_validation/read1_not_a_string → error_handling/input_files/read1_not_a_string"
    mv "input_validation/read1_not_a_string" "error_handling/input_files/read1_not_a_string"
fi

# Input file validation errors
if [ -d "input_validation/read2_file_does_not_exist" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read2_file_does_not_exist')"
    echo "  input_validation/read2_file_does_not_exist → error_handling/input_files/read2_file_does_not_exist"
    mv "input_validation/read2_file_does_not_exist" "error_handling/input_files/read2_file_does_not_exist"
fi

# Input file validation errors
if [ -d "input_validation/read2_not_a_string" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/read2_not_a_string')"
    echo "  input_validation/read2_not_a_string → error_handling/input_files/read2_not_a_string"
    mv "input_validation/read2_not_a_string" "error_handling/input_files/read2_not_a_string"
fi

# Miscellaneous validation errors
if [ -d "input_validation/read_with_comment_in_line_3" ]; then
    mkdir -p "$(dirname 'error_handling/misc/read_with_comment_in_line_3')"
    echo "  input_validation/read_with_comment_in_line_3 → error_handling/misc/read_with_comment_in_line_3"
    mv "input_validation/read_with_comment_in_line_3" "error_handling/misc/read_with_comment_in_line_3"
fi

# Input file validation errors
if [ -d "input_validation/repeated_filenames" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/repeated_filenames')"
    echo "  input_validation/repeated_filenames → error_handling/input_files/repeated_filenames"
    mv "input_validation/repeated_filenames" "error_handling/input_files/repeated_filenames"
fi

# Input file validation errors
if [ -d "input_validation/repeated_filenames_index1" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/repeated_filenames_index1')"
    echo "  input_validation/repeated_filenames_index1 → error_handling/input_files/repeated_filenames_index1"
    mv "input_validation/repeated_filenames_index1" "error_handling/input_files/repeated_filenames_index1"
fi

# Input file validation errors
if [ -d "input_validation/repeated_filenames_index2" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/repeated_filenames_index2')"
    echo "  input_validation/repeated_filenames_index2 → error_handling/input_files/repeated_filenames_index2"
    mv "input_validation/repeated_filenames_index2" "error_handling/input_files/repeated_filenames_index2"
fi

# Input file validation errors
if [ -d "input_validation/repeated_filenames_one_key" ]; then
    mkdir -p "$(dirname 'error_handling/input_files/repeated_filenames_one_key')"
    echo "  input_validation/repeated_filenames_one_key → error_handling/input_files/repeated_filenames_one_key"
    mv "input_validation/repeated_filenames_one_key" "error_handling/input_files/repeated_filenames_one_key"
fi

# Report validation errors
if [ -d "input_validation/report_but_no_report_step_html" ]; then
    mkdir -p "$(dirname 'error_handling/reports/report_but_no_report_step_html')"
    echo "  input_validation/report_but_no_report_step_html → error_handling/reports/report_but_no_report_step_html"
    mv "input_validation/report_but_no_report_step_html" "error_handling/reports/report_but_no_report_step_html"
fi

# Report validation errors
if [ -d "input_validation/report_but_no_report_step_json" ]; then
    mkdir -p "$(dirname 'error_handling/reports/report_but_no_report_step_json')"
    echo "  input_validation/report_but_no_report_step_json → error_handling/reports/report_but_no_report_step_json"
    mv "input_validation/report_but_no_report_step_json" "error_handling/reports/report_but_no_report_step_json"
fi

# Miscellaneous validation errors
if [ -d "input_validation/report_names_distinct" ]; then
    mkdir -p "$(dirname 'error_handling/misc/report_names_distinct')"
    echo "  input_validation/report_names_distinct → error_handling/misc/report_names_distinct"
    mv "input_validation/report_names_distinct" "error_handling/misc/report_names_distinct"
fi

# Output configuration errors
if [ -d "input_validation/report_without_output_flags" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/report_without_output_flags')"
    echo "  input_validation/report_without_output_flags → error_handling/output_config/report_without_output_flags"
    mv "input_validation/report_without_output_flags" "error_handling/output_config/report_without_output_flags"
fi

# Output configuration errors
if [ -d "input_validation/segment_defaults_multiple_segments_fails" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_defaults_multiple_segments_fails')"
    echo "  input_validation/segment_defaults_multiple_segments_fails → error_handling/output_config/segment_defaults_multiple_segments_fails"
    mv "input_validation/segment_defaults_multiple_segments_fails" "error_handling/output_config/segment_defaults_multiple_segments_fails"
fi

# Output configuration errors
if [ -d "input_validation/segment_duplicated_interleave" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_duplicated_interleave')"
    echo "  input_validation/segment_duplicated_interleave → error_handling/output_config/segment_duplicated_interleave"
    mv "input_validation/segment_duplicated_interleave" "error_handling/output_config/segment_duplicated_interleave"
fi

# Output configuration errors
if [ -d "input_validation/segment_name_duplicated_after_trim" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_name_duplicated_after_trim')"
    echo "  input_validation/segment_name_duplicated_after_trim → error_handling/output_config/segment_name_duplicated_after_trim"
    mv "input_validation/segment_name_duplicated_after_trim" "error_handling/output_config/segment_name_duplicated_after_trim"
fi

# Output configuration errors
if [ -d "input_validation/segment_name_empty" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_name_empty')"
    echo "  input_validation/segment_name_empty → error_handling/output_config/segment_name_empty"
    mv "input_validation/segment_name_empty" "error_handling/output_config/segment_name_empty"
fi

# Output configuration errors
if [ -d "input_validation/segment_name_invalid_path" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_name_invalid_path')"
    echo "  input_validation/segment_name_invalid_path → error_handling/output_config/segment_name_invalid_path"
    mv "input_validation/segment_name_invalid_path" "error_handling/output_config/segment_name_invalid_path"
fi

# Output configuration errors
if [ -d "input_validation/segment_name_invalid_path2" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_name_invalid_path2')"
    echo "  input_validation/segment_name_invalid_path2 → error_handling/output_config/segment_name_invalid_path2"
    mv "input_validation/segment_name_invalid_path2" "error_handling/output_config/segment_name_invalid_path2"
fi

# Output configuration errors
if [ -d "input_validation/segment_name_whitespace_only" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/segment_name_whitespace_only')"
    echo "  input_validation/segment_name_whitespace_only → error_handling/output_config/segment_name_whitespace_only"
    mv "input_validation/segment_name_whitespace_only" "error_handling/output_config/segment_name_whitespace_only"
fi

# Error message tests
if [ -d "input_validation/show_step_template_on_error" ]; then
    mkdir -p "$(dirname 'error_handling/error_messages/show_step_template_on_error')"
    echo "  input_validation/show_step_template_on_error → error_handling/error_messages/show_step_template_on_error"
    mv "input_validation/show_step_template_on_error" "error_handling/error_messages/show_step_template_on_error"
fi

# I/O validation errors
if [ -d "input_validation/stdin_multiple_files" ]; then
    mkdir -p "$(dirname 'error_handling/io/stdin_multiple_files')"
    echo "  input_validation/stdin_multiple_files → error_handling/io/stdin_multiple_files"
    mv "input_validation/stdin_multiple_files" "error_handling/io/stdin_multiple_files"
fi

# Output configuration errors
if [ -d "input_validation/stdin_multiple_segments" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/stdin_multiple_segments')"
    echo "  input_validation/stdin_multiple_segments → error_handling/output_config/stdin_multiple_segments"
    mv "input_validation/stdin_multiple_segments" "error_handling/output_config/stdin_multiple_segments"
fi

# Output configuration errors
if [ -d "input_validation/stdout_conflict" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/stdout_conflict')"
    echo "  input_validation/stdout_conflict → error_handling/output_config/stdout_conflict"
    mv "input_validation/stdout_conflict" "error_handling/output_config/stdout_conflict"
fi

# Extraction/tag validation errors
if [ -d "input_validation/trim_tag_multi_locations" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/trim_tag_multi_locations')"
    echo "  input_validation/trim_tag_multi_locations → error_handling/extraction/trim_tag_multi_locations"
    mv "input_validation/trim_tag_multi_locations" "error_handling/extraction/trim_tag_multi_locations"
fi

# Malformed FASTQ files
if [ -d "input_validation/truncated_after_at" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/truncated_after_at')"
    echo "  input_validation/truncated_after_at → error_handling/malformed_fastq/truncated_after_at"
    mv "input_validation/truncated_after_at" "error_handling/malformed_fastq/truncated_after_at"
fi

# Error message tests
if [ -d "input_validation/two_mistakes_eserde" ]; then
    mkdir -p "$(dirname 'error_handling/error_messages/two_mistakes_eserde')"
    echo "  input_validation/two_mistakes_eserde → error_handling/error_messages/two_mistakes_eserde"
    mv "input_validation/two_mistakes_eserde" "error_handling/error_messages/two_mistakes_eserde"
fi

# Error message tests
if [ -d "input_validation/two_mistakes_post_deserialization" ]; then
    mkdir -p "$(dirname 'error_handling/error_messages/two_mistakes_post_deserialization')"
    echo "  input_validation/two_mistakes_post_deserialization → error_handling/error_messages/two_mistakes_post_deserialization"
    mv "input_validation/two_mistakes_post_deserialization" "error_handling/error_messages/two_mistakes_post_deserialization"
fi

# Miscellaneous validation errors
if [ -d "input_validation/u8_from_char_number_to_large" ]; then
    mkdir -p "$(dirname 'error_handling/misc/u8_from_char_number_to_large')"
    echo "  input_validation/u8_from_char_number_to_large → error_handling/misc/u8_from_char_number_to_large"
    mv "input_validation/u8_from_char_number_to_large" "error_handling/misc/u8_from_char_number_to_large"
fi

# Miscellaneous validation errors
if [ -d "input_validation/u8_from_char_too_many_chars" ]; then
    mkdir -p "$(dirname 'error_handling/misc/u8_from_char_too_many_chars')"
    echo "  input_validation/u8_from_char_too_many_chars → error_handling/misc/u8_from_char_too_many_chars"
    mv "input_validation/u8_from_char_too_many_chars" "error_handling/misc/u8_from_char_too_many_chars"
fi

# Extraction/tag validation errors
if [ -d "input_validation/unused_extract_tag" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/unused_extract_tag')"
    echo "  input_validation/unused_extract_tag → error_handling/extraction/unused_extract_tag"
    mv "input_validation/unused_extract_tag" "error_handling/extraction/unused_extract_tag"
fi

# Output configuration errors
if [ -d "input_validation/unwritable_output_dir" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/unwritable_output_dir')"
    echo "  input_validation/unwritable_output_dir → error_handling/output_config/unwritable_output_dir"
    mv "input_validation/unwritable_output_dir" "error_handling/output_config/unwritable_output_dir"
fi

# Output configuration errors
if [ -d "input_validation/validate_name_needs_multiple_segments" ]; then
    mkdir -p "$(dirname 'error_handling/output_config/validate_name_needs_multiple_segments')"
    echo "  input_validation/validate_name_needs_multiple_segments → error_handling/output_config/validate_name_needs_multiple_segments"
    mv "input_validation/validate_name_needs_multiple_segments" "error_handling/output_config/validate_name_needs_multiple_segments"
fi

# Extraction/tag validation errors
if [ -d "input_validation/validate_regex_fail" ]; then
    mkdir -p "$(dirname 'error_handling/extraction/validate_regex_fail')"
    echo "  input_validation/validate_regex_fail → error_handling/extraction/validate_regex_fail"
    mv "input_validation/validate_regex_fail" "error_handling/extraction/validate_regex_fail"
fi

# Malformed FASTQ files
if [ -d "input_validation/windows_newlines" ]; then
    mkdir -p "$(dirname 'error_handling/malformed_fastq/windows_newlines')"
    echo "  input_validation/windows_newlines → error_handling/malformed_fastq/windows_newlines"
    mv "input_validation/windows_newlines" "error_handling/malformed_fastq/windows_newlines"
fi


# integration_tests/ (124 tests)
echo "Processing integration_tests/..."

# BAM/external file test
if [ -d "integration_tests/output/chunked/bam" ]; then
    mkdir -p "$(dirname 'fileformats/output/chunked/bam')"
    echo "  integration_tests/output/chunked/bam → fileformats/output/chunked/bam"
    mv "integration_tests/output/chunked/bam" "fileformats/output/chunked/bam"
fi

# Output configuration test
if [ -d "integration_tests/output/chunked/fastq" ]; then
    mkdir -p "$(dirname 'output/output/chunked/fastq')"
    echo "  integration_tests/output/chunked/fastq → output/output/chunked/fastq"
    mv "integration_tests/output/chunked/fastq" "output/output/chunked/fastq"
fi

# Output configuration test
if [ -d "integration_tests/output/chunked/fastq_exceeding_10k_chunks" ]; then
    mkdir -p "$(dirname 'output/output/chunked/fastq_exceeding_10k_chunks')"
    echo "  integration_tests/output/chunked/fastq_exceeding_10k_chunks → output/output/chunked/fastq_exceeding_10k_chunks"
    mv "integration_tests/output/chunked/fastq_exceeding_10k_chunks" "output/output/chunked/fastq_exceeding_10k_chunks"
fi

# Compression test
if [ -d "integration_tests/output/chunked/fastq_gzip" ]; then
    mkdir -p "$(dirname 'compression/output/chunked/fastq_gzip')"
    echo "  integration_tests/output/chunked/fastq_gzip → compression/output/chunked/fastq_gzip"
    mv "integration_tests/output/chunked/fastq_gzip" "compression/output/chunked/fastq_gzip"
fi

# General integration test: both
if [ -d "integration_tests/cut_end_named_pipes/both" ]; then
    mkdir -p "$(dirname 'integration/cut_end_named_pipes/both')"
    echo "  integration_tests/cut_end_named_pipes/both → integration/cut_end_named_pipes/both"
    mv "integration_tests/cut_end_named_pipes/both" "integration/cut_end_named_pipes/both"
fi

# General integration test: input_pipe
if [ -d "integration_tests/cut_end_named_pipes/input_pipe" ]; then
    mkdir -p "$(dirname 'integration/cut_end_named_pipes/input_pipe')"
    echo "  integration_tests/cut_end_named_pipes/input_pipe → integration/cut_end_named_pipes/input_pipe"
    mv "integration_tests/cut_end_named_pipes/input_pipe" "integration/cut_end_named_pipes/input_pipe"
fi

# Output configuration test
if [ -d "integration_tests/cut_end_named_pipes/output_pipe" ]; then
    mkdir -p "$(dirname 'output/cut_end_named_pipes/output_pipe')"
    echo "  integration_tests/cut_end_named_pipes/output_pipe → output/cut_end_named_pipes/output_pipe"
    mv "integration_tests/cut_end_named_pipes/output_pipe" "output/cut_end_named_pipes/output_pipe"
fi

# Deduplication test
if [ -d "integration_tests/dedup/basic" ]; then
    mkdir -p "$(dirname 'dedup/basic')"
    echo "  integration_tests/dedup/basic → dedup/basic"
    mv "integration_tests/dedup/basic" "dedup/basic"
fi

# Deduplication test
if [ -d "integration_tests/dedup/dedup_exact" ]; then
    mkdir -p "$(dirname 'dedup/dedup_exact')"
    echo "  integration_tests/dedup/dedup_exact → dedup/dedup_exact"
    mv "integration_tests/dedup/dedup_exact" "dedup/dedup_exact"
fi

# Deduplication test
if [ -d "integration_tests/dedup/dedup_keep_duplicates" ]; then
    mkdir -p "$(dirname 'dedup/dedup_keep_duplicates')"
    echo "  integration_tests/dedup/dedup_keep_duplicates → dedup/dedup_keep_duplicates"
    mv "integration_tests/dedup/dedup_keep_duplicates" "dedup/dedup_keep_duplicates"
fi

# Deduplication test
if [ -d "integration_tests/dedup/dedup_read2" ]; then
    mkdir -p "$(dirname 'dedup/dedup_read2')"
    echo "  integration_tests/dedup/dedup_read2 → dedup/dedup_read2"
    mv "integration_tests/dedup/dedup_read2" "dedup/dedup_read2"
fi

# Deduplication test
if [ -d "integration_tests/dedup/dedup_read_combo" ]; then
    mkdir -p "$(dirname 'dedup/dedup_read_combo')"
    echo "  integration_tests/dedup/dedup_read_combo → dedup/dedup_read_combo"
    mv "integration_tests/dedup/dedup_read_combo" "dedup/dedup_read_combo"
fi

# Deduplication test
if [ -d "integration_tests/dedup/dedup_read_combo_incl_index" ]; then
    mkdir -p "$(dirname 'dedup/dedup_read_combo_incl_index')"
    echo "  integration_tests/dedup/dedup_read_combo_incl_index → dedup/dedup_read_combo_incl_index"
    mv "integration_tests/dedup/dedup_read_combo_incl_index" "dedup/dedup_read_combo_incl_index"
fi

# Deduplication test
if [ -d "integration_tests/dedup/exact_name" ]; then
    mkdir -p "$(dirname 'dedup/exact_name')"
    echo "  integration_tests/dedup/exact_name → dedup/exact_name"
    mv "integration_tests/dedup/exact_name" "dedup/exact_name"
fi

# Deduplication test
if [ -d "integration_tests/dedup/exact_tag" ]; then
    mkdir -p "$(dirname 'dedup/exact_tag')"
    echo "  integration_tests/dedup/exact_tag → dedup/exact_tag"
    mv "integration_tests/dedup/exact_tag" "dedup/exact_tag"
fi

# Expression evaluation test
if [ -d "integration_tests/eval_expr/eval_expression_basic" ]; then
    mkdir -p "$(dirname 'eval/eval_expr/eval_expression_basic')"
    echo "  integration_tests/eval_expr/eval_expression_basic → eval/eval_expr/eval_expression_basic"
    mv "integration_tests/eval_expr/eval_expression_basic" "eval/eval_expr/eval_expression_basic"
fi

# Expression evaluation test
if [ -d "integration_tests/eval_expr/eval_expression_bool" ]; then
    mkdir -p "$(dirname 'eval/eval_expr/eval_expression_bool')"
    echo "  integration_tests/eval_expr/eval_expression_bool → eval/eval_expr/eval_expression_bool"
    mv "integration_tests/eval_expr/eval_expression_bool" "eval/eval_expr/eval_expression_bool"
fi

# Expression evaluation test
if [ -d "integration_tests/eval_expr/eval_expression_complex" ]; then
    mkdir -p "$(dirname 'eval/eval_expr/eval_expression_complex')"
    echo "  integration_tests/eval_expr/eval_expression_complex → eval/eval_expr/eval_expression_complex"
    mv "integration_tests/eval_expr/eval_expression_complex" "eval/eval_expr/eval_expression_complex"
fi

# General integration test: location
if [ -d "integration_tests/eval_expr/location" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/location')"
    echo "  integration_tests/eval_expr/location → integration/eval_expr/location"
    mv "integration_tests/eval_expr/location" "integration/eval_expr/location"
fi

# General integration test: location_len
if [ -d "integration_tests/eval_expr/location_len" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/location_len')"
    echo "  integration_tests/eval_expr/location_len → integration/eval_expr/location_len"
    mv "integration_tests/eval_expr/location_len" "integration/eval_expr/location_len"
fi

# General integration test: segment_len
if [ -d "integration_tests/eval_expr/segment_len" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/segment_len')"
    echo "  integration_tests/eval_expr/segment_len → integration/eval_expr/segment_len"
    mv "integration_tests/eval_expr/segment_len" "integration/eval_expr/segment_len"
fi

# General integration test: str
if [ -d "integration_tests/eval_expr/str" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/str')"
    echo "  integration_tests/eval_expr/str → integration/eval_expr/str"
    mv "integration_tests/eval_expr/str" "integration/eval_expr/str"
fi

# General integration test: str_len
if [ -d "integration_tests/eval_expr/str_len" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/str_len')"
    echo "  integration_tests/eval_expr/str_len → integration/eval_expr/str_len"
    mv "integration_tests/eval_expr/str_len" "integration/eval_expr/str_len"
fi

# General integration test: threshold
if [ -d "integration_tests/eval_expr/threshold" ]; then
    mkdir -p "$(dirname 'integration/eval_expr/threshold')"
    echo "  integration_tests/eval_expr/threshold → integration/eval_expr/threshold"
    mv "integration_tests/eval_expr/threshold" "integration/eval_expr/threshold"
fi

# Extraction test
if [ -d "integration_tests/extract_regex/extract_regex_from_name" ]; then
    mkdir -p "$(dirname 'extraction/extract_regex/extract_regex_from_name')"
    echo "  integration_tests/extract_regex/extract_regex_from_name → extraction/extract_regex/extract_regex_from_name"
    mv "integration_tests/extract_regex/extract_regex_from_name" "extraction/extract_regex/extract_regex_from_name"
fi

# Extraction test
if [ -d "integration_tests/extract_regex/extract_regex_from_name_multi_segment" ]; then
    mkdir -p "$(dirname 'extraction/extract_regex/extract_regex_from_name_multi_segment')"
    echo "  integration_tests/extract_regex/extract_regex_from_name_multi_segment → extraction/extract_regex/extract_regex_from_name_multi_segment"
    mv "integration_tests/extract_regex/extract_regex_from_name_multi_segment" "extraction/extract_regex/extract_regex_from_name_multi_segment"
fi

# Extraction test
if [ -d "integration_tests/extract_regex/extract_regex_from_name_no_replacement" ]; then
    mkdir -p "$(dirname 'extraction/extract_regex/extract_regex_from_name_no_replacement')"
    echo "  integration_tests/extract_regex/extract_regex_from_name_no_replacement → extraction/extract_regex/extract_regex_from_name_no_replacement"
    mv "integration_tests/extract_regex/extract_regex_from_name_no_replacement" "extraction/extract_regex/extract_regex_from_name_no_replacement"
fi

# Extraction test
if [ -d "integration_tests/extract_regex/extract_regex_no_replacement" ]; then
    mkdir -p "$(dirname 'extraction/extract_regex/extract_regex_no_replacement')"
    echo "  integration_tests/extract_regex/extract_regex_no_replacement → extraction/extract_regex/extract_regex_no_replacement"
    mv "integration_tests/extract_regex/extract_regex_no_replacement" "extraction/extract_regex/extract_regex_no_replacement"
fi

# General integration test: above
if [ -d "integration_tests/filter_qualified_bases/above" ]; then
    mkdir -p "$(dirname 'integration/filter_qualified_bases/above')"
    echo "  integration_tests/filter_qualified_bases/above → integration/filter_qualified_bases/above"
    mv "integration_tests/filter_qualified_bases/above" "integration/filter_qualified_bases/above"
fi

# General integration test: above_or_equal
if [ -d "integration_tests/filter_qualified_bases/above_or_equal" ]; then
    mkdir -p "$(dirname 'integration/filter_qualified_bases/above_or_equal')"
    echo "  integration_tests/filter_qualified_bases/above_or_equal → integration/filter_qualified_bases/above_or_equal"
    mv "integration_tests/filter_qualified_bases/above_or_equal" "integration/filter_qualified_bases/above_or_equal"
fi

# General integration test: below
if [ -d "integration_tests/filter_qualified_bases/below" ]; then
    mkdir -p "$(dirname 'integration/filter_qualified_bases/below')"
    echo "  integration_tests/filter_qualified_bases/below → integration/filter_qualified_bases/below"
    mv "integration_tests/filter_qualified_bases/below" "integration/filter_qualified_bases/below"
fi

# General integration test: below_or_equal
if [ -d "integration_tests/filter_qualified_bases/below_or_equal" ]; then
    mkdir -p "$(dirname 'integration/filter_qualified_bases/below_or_equal')"
    echo "  integration_tests/filter_qualified_bases/below_or_equal → integration/filter_qualified_bases/below_or_equal"
    mv "integration_tests/filter_qualified_bases/below_or_equal" "integration/filter_qualified_bases/below_or_equal"
fi

# I/O format test
if [ -d "integration_tests/inspect/inspect_all_interleaved" ]; then
    mkdir -p "$(dirname 'io/inspect/inspect_all_interleaved')"
    echo "  integration_tests/inspect/inspect_all_interleaved → io/inspect/inspect_all_interleaved"
    mv "integration_tests/inspect/inspect_all_interleaved" "io/inspect/inspect_all_interleaved"
fi

# I/O format test
if [ -d "integration_tests/inspect/inspect_all_interleaved_reversed" ]; then
    mkdir -p "$(dirname 'io/inspect/inspect_all_interleaved_reversed')"
    echo "  integration_tests/inspect/inspect_all_interleaved_reversed → io/inspect/inspect_all_interleaved_reversed"
    mv "integration_tests/inspect/inspect_all_interleaved_reversed" "io/inspect/inspect_all_interleaved_reversed"
fi

# Compression test
if [ -d "integration_tests/inspect/inspect_compression_zstd_level" ]; then
    mkdir -p "$(dirname 'compression/inspect/inspect_compression_zstd_level')"
    echo "  integration_tests/inspect/inspect_compression_zstd_level → compression/inspect/inspect_compression_zstd_level"
    mv "integration_tests/inspect/inspect_compression_zstd_level" "compression/inspect/inspect_compression_zstd_level"
fi

# File inspection test
if [ -d "integration_tests/inspect/inspect_index1" ]; then
    mkdir -p "$(dirname 'inspect/inspect/inspect_index1')"
    echo "  integration_tests/inspect/inspect_index1 → inspect/inspect/inspect_index1"
    mv "integration_tests/inspect/inspect_index1" "inspect/inspect/inspect_index1"
fi

# File inspection test
if [ -d "integration_tests/inspect/inspect_index2" ]; then
    mkdir -p "$(dirname 'inspect/inspect/inspect_index2')"
    echo "  integration_tests/inspect/inspect_index2 → inspect/inspect/inspect_index2"
    mv "integration_tests/inspect/inspect_index2" "inspect/inspect/inspect_index2"
fi

# File inspection test
if [ -d "integration_tests/inspect/inspect_read1" ]; then
    mkdir -p "$(dirname 'inspect/inspect/inspect_read1')"
    echo "  integration_tests/inspect/inspect_read1 → inspect/inspect/inspect_read1"
    mv "integration_tests/inspect/inspect_read1" "inspect/inspect/inspect_read1"
fi

# Compression test
if [ -d "integration_tests/inspect/inspect_read1_compressed" ]; then
    mkdir -p "$(dirname 'compression/inspect/inspect_read1_compressed')"
    echo "  integration_tests/inspect/inspect_read1_compressed → compression/inspect/inspect_read1_compressed"
    mv "integration_tests/inspect/inspect_read1_compressed" "compression/inspect/inspect_read1_compressed"
fi

# File inspection test
if [ -d "integration_tests/inspect/inspect_read2" ]; then
    mkdir -p "$(dirname 'inspect/inspect/inspect_read2')"
    echo "  integration_tests/inspect/inspect_read2 → inspect/inspect/inspect_read2"
    mv "integration_tests/inspect/inspect_read2" "inspect/inspect/inspect_read2"
fi

# BAM/external file test
if [ -d "integration_tests/output/bam" ]; then
    mkdir -p "$(dirname 'fileformats/output/bam')"
    echo "  integration_tests/output/bam → fileformats/output/bam"
    mv "integration_tests/output/bam" "fileformats/output/bam"
fi

# Output configuration test
if [ -d "integration_tests/output/fastq" ]; then
    mkdir -p "$(dirname 'output/output/fastq')"
    echo "  integration_tests/output/fastq → output/output/fastq"
    mv "integration_tests/output/fastq" "output/output/fastq"
fi

# I/O format test
if [ -d "integration_tests/stdin/stdin_interleaved" ]; then
    mkdir -p "$(dirname 'io/stdin/stdin_interleaved')"
    echo "  integration_tests/stdin/stdin_interleaved → io/stdin/stdin_interleaved"
    mv "integration_tests/stdin/stdin_interleaved" "io/stdin/stdin_interleaved"
fi

# I/O format test
if [ -d "integration_tests/stdin/stdin_regular" ]; then
    mkdir -p "$(dirname 'io/stdin/stdin_regular')"
    echo "  integration_tests/stdin/stdin_regular → io/stdin/stdin_regular"
    mv "integration_tests/stdin/stdin_regular" "io/stdin/stdin_regular"
fi

# Basic functionality test
if [ -d "integration_tests/allow_overwrites" ]; then
    mkdir -p "$(dirname 'basic/allow_overwrites')"
    echo "  integration_tests/allow_overwrites → basic/allow_overwrites"
    mv "integration_tests/allow_overwrites" "basic/allow_overwrites"
fi

# Quality/complexity test
if [ -d "integration_tests/convert_phred" ]; then
    mkdir -p "$(dirname 'quality/convert_phred')"
    echo "  integration_tests/convert_phred → quality/convert_phred"
    mv "integration_tests/convert_phred" "quality/convert_phred"
fi

# Quality/complexity test
if [ -d "integration_tests/convert_phred_multi" ]; then
    mkdir -p "$(dirname 'quality/convert_phred_multi')"
    echo "  integration_tests/convert_phred_multi → quality/convert_phred_multi"
    mv "integration_tests/convert_phred_multi" "quality/convert_phred_multi"
fi

# Trimming/cutting test: cut_end
if [ -d "integration_tests/cut_end" ]; then
    mkdir -p "$(dirname 'trim/cut_end')"
    echo "  integration_tests/cut_end → trim/cut_end"
    mv "integration_tests/cut_end" "trim/cut_end"
fi

# Trimming/cutting test: cut_start
if [ -d "integration_tests/cut_start" ]; then
    mkdir -p "$(dirname 'trim/cut_start')"
    echo "  integration_tests/cut_start → trim/cut_start"
    mv "integration_tests/cut_start" "trim/cut_start"
fi

# Extraction test
if [ -d "integration_tests/extract_iupac_suffix" ]; then
    mkdir -p "$(dirname 'extraction/extract_iupac_suffix')"
    echo "  integration_tests/extract_iupac_suffix → extraction/extract_iupac_suffix"
    mv "integration_tests/extract_iupac_suffix" "extraction/extract_iupac_suffix"
fi

# fastp compatibility test
if [ -d "integration_tests/fastp_416" ]; then
    mkdir -p "$(dirname 'compatibility/fastp_416')"
    echo "  integration_tests/fastp_416 → compatibility/fastp_416"
    mv "integration_tests/fastp_416" "compatibility/fastp_416"
fi

# fastp compatibility test
if [ -d "integration_tests/fastp_491" ]; then
    mkdir -p "$(dirname 'compatibility/fastp_491')"
    echo "  integration_tests/fastp_491 → compatibility/fastp_491"
    mv "integration_tests/fastp_491" "compatibility/fastp_491"
fi

# fastp compatibility test
if [ -d "integration_tests/fastp_606" ]; then
    mkdir -p "$(dirname 'compatibility/fastp_606')"
    echo "  integration_tests/fastp_606 → compatibility/fastp_606"
    mv "integration_tests/fastp_606" "compatibility/fastp_606"
fi

# Filtering test: filter_empty
if [ -d "integration_tests/filter_empty" ]; then
    mkdir -p "$(dirname 'filter/filter_empty')"
    echo "  integration_tests/filter_empty → filter/filter_empty"
    mv "integration_tests/filter_empty" "filter/filter_empty"
fi

# Filtering test: filter_empty_all
if [ -d "integration_tests/filter_empty_all" ]; then
    mkdir -p "$(dirname 'filter/filter_empty_all')"
    echo "  integration_tests/filter_empty_all → filter/filter_empty_all"
    mv "integration_tests/filter_empty_all" "filter/filter_empty_all"
fi

# Filtering test: filter_empty_segments
if [ -d "integration_tests/filter_empty_segments" ]; then
    mkdir -p "$(dirname 'filter/filter_empty_segments')"
    echo "  integration_tests/filter_empty_segments → filter/filter_empty_segments"
    mv "integration_tests/filter_empty_segments" "filter/filter_empty_segments"
fi

# Filtering test: filter_max_len
if [ -d "integration_tests/filter_max_len" ]; then
    mkdir -p "$(dirname 'filter/filter_max_len')"
    echo "  integration_tests/filter_max_len → filter/filter_max_len"
    mv "integration_tests/filter_max_len" "filter/filter_max_len"
fi

# Filtering test: filter_min_len
if [ -d "integration_tests/filter_min_len" ]; then
    mkdir -p "$(dirname 'filter/filter_min_len')"
    echo "  integration_tests/filter_min_len → filter/filter_min_len"
    mv "integration_tests/filter_min_len" "filter/filter_min_len"
fi

# Filtering test: filter_other_file_by_name_distinct_separators
if [ -d "integration_tests/filter_other_file_by_name_distinct_separators" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_distinct_separators')"
    echo "  integration_tests/filter_other_file_by_name_distinct_separators → filter/filter_other_file_by_name_distinct_separators"
    mv "integration_tests/filter_other_file_by_name_distinct_separators" "filter/filter_other_file_by_name_distinct_separators"
fi

# Filtering test: filter_other_file_by_name_keep
if [ -d "integration_tests/filter_other_file_by_name_keep" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_keep')"
    echo "  integration_tests/filter_other_file_by_name_keep → filter/filter_other_file_by_name_keep"
    mv "integration_tests/filter_other_file_by_name_keep" "filter/filter_other_file_by_name_keep"
fi

# Filtering test: filter_other_file_by_name_remove
if [ -d "integration_tests/filter_other_file_by_name_remove" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove')"
    echo "  integration_tests/filter_other_file_by_name_remove → filter/filter_other_file_by_name_remove"
    mv "integration_tests/filter_other_file_by_name_remove" "filter/filter_other_file_by_name_remove"
fi

# Filtering test: filter_other_file_by_name_remove_bam
if [ -d "integration_tests/filter_other_file_by_name_remove_bam" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove_bam')"
    echo "  integration_tests/filter_other_file_by_name_remove_bam → filter/filter_other_file_by_name_remove_bam"
    mv "integration_tests/filter_other_file_by_name_remove_bam" "filter/filter_other_file_by_name_remove_bam"
fi

# Filtering test: filter_other_file_by_name_remove_bam_approximate
if [ -d "integration_tests/filter_other_file_by_name_remove_bam_approximate" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove_bam_approximate')"
    echo "  integration_tests/filter_other_file_by_name_remove_bam_approximate → filter/filter_other_file_by_name_remove_bam_approximate"
    mv "integration_tests/filter_other_file_by_name_remove_bam_approximate" "filter/filter_other_file_by_name_remove_bam_approximate"
fi

# Filtering test: filter_other_file_by_name_remove_bam_approximate_no_bai
if [ -d "integration_tests/filter_other_file_by_name_remove_bam_approximate_no_bai" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove_bam_approximate_no_bai')"
    echo "  integration_tests/filter_other_file_by_name_remove_bam_approximate_no_bai → filter/filter_other_file_by_name_remove_bam_approximate_no_bai"
    mv "integration_tests/filter_other_file_by_name_remove_bam_approximate_no_bai" "filter/filter_other_file_by_name_remove_bam_approximate_no_bai"
fi

# Filtering test: filter_other_file_by_name_remove_bam_unaligned
if [ -d "integration_tests/filter_other_file_by_name_remove_bam_unaligned" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove_bam_unaligned')"
    echo "  integration_tests/filter_other_file_by_name_remove_bam_unaligned → filter/filter_other_file_by_name_remove_bam_unaligned"
    mv "integration_tests/filter_other_file_by_name_remove_bam_unaligned" "filter/filter_other_file_by_name_remove_bam_unaligned"
fi

# Filtering test: filter_other_file_by_name_remove_bam_unaligned_no_ignore
if [ -d "integration_tests/filter_other_file_by_name_remove_bam_unaligned_no_ignore" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_name_remove_bam_unaligned_no_ignore')"
    echo "  integration_tests/filter_other_file_by_name_remove_bam_unaligned_no_ignore → filter/filter_other_file_by_name_remove_bam_unaligned_no_ignore"
    mv "integration_tests/filter_other_file_by_name_remove_bam_unaligned_no_ignore" "filter/filter_other_file_by_name_remove_bam_unaligned_no_ignore"
fi

# Filtering test: filter_other_file_by_seq_keep
if [ -d "integration_tests/filter_other_file_by_seq_keep" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_seq_keep')"
    echo "  integration_tests/filter_other_file_by_seq_keep → filter/filter_other_file_by_seq_keep"
    mv "integration_tests/filter_other_file_by_seq_keep" "filter/filter_other_file_by_seq_keep"
fi

# Filtering test: filter_other_file_by_seq_remove
if [ -d "integration_tests/filter_other_file_by_seq_remove" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_seq_remove')"
    echo "  integration_tests/filter_other_file_by_seq_remove → filter/filter_other_file_by_seq_remove"
    mv "integration_tests/filter_other_file_by_seq_remove" "filter/filter_other_file_by_seq_remove"
fi

# Filtering test: filter_other_file_by_seq_remove_bam
if [ -d "integration_tests/filter_other_file_by_seq_remove_bam" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_seq_remove_bam')"
    echo "  integration_tests/filter_other_file_by_seq_remove_bam → filter/filter_other_file_by_seq_remove_bam"
    mv "integration_tests/filter_other_file_by_seq_remove_bam" "filter/filter_other_file_by_seq_remove_bam"
fi

# Filtering test: filter_other_file_by_seq_remove_bam_unaligned
if [ -d "integration_tests/filter_other_file_by_seq_remove_bam_unaligned" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_seq_remove_bam_unaligned')"
    echo "  integration_tests/filter_other_file_by_seq_remove_bam_unaligned → filter/filter_other_file_by_seq_remove_bam_unaligned"
    mv "integration_tests/filter_other_file_by_seq_remove_bam_unaligned" "filter/filter_other_file_by_seq_remove_bam_unaligned"
fi

# Filtering test: filter_other_file_by_seq_remove_bam_unaligned_no_ignore
if [ -d "integration_tests/filter_other_file_by_seq_remove_bam_unaligned_no_ignore" ]; then
    mkdir -p "$(dirname 'filter/filter_other_file_by_seq_remove_bam_unaligned_no_ignore')"
    echo "  integration_tests/filter_other_file_by_seq_remove_bam_unaligned_no_ignore → filter/filter_other_file_by_seq_remove_bam_unaligned_no_ignore"
    mv "integration_tests/filter_other_file_by_seq_remove_bam_unaligned_no_ignore" "filter/filter_other_file_by_seq_remove_bam_unaligned_no_ignore"
fi

# Filtering test: filter_too_many_n
if [ -d "integration_tests/filter_too_many_n" ]; then
    mkdir -p "$(dirname 'filter/filter_too_many_n')"
    echo "  integration_tests/filter_too_many_n → filter/filter_too_many_n"
    mv "integration_tests/filter_too_many_n" "filter/filter_too_many_n"
fi

# Filtering test: filter_too_many_n_all
if [ -d "integration_tests/filter_too_many_n_all" ]; then
    mkdir -p "$(dirname 'filter/filter_too_many_n_all')"
    echo "  integration_tests/filter_too_many_n_all → filter/filter_too_many_n_all"
    mv "integration_tests/filter_too_many_n_all" "filter/filter_too_many_n_all"
fi

# Filtering test: filter_too_many_n_segments_vs_all
if [ -d "integration_tests/filter_too_many_n_segments_vs_all" ]; then
    mkdir -p "$(dirname 'filter/filter_too_many_n_segments_vs_all')"
    echo "  integration_tests/filter_too_many_n_segments_vs_all → filter/filter_too_many_n_segments_vs_all"
    mv "integration_tests/filter_too_many_n_segments_vs_all" "filter/filter_too_many_n_segments_vs_all"
fi

# Compression test
if [ -d "integration_tests/gz_input" ]; then
    mkdir -p "$(dirname 'compression/gz_input')"
    echo "  integration_tests/gz_input → compression/gz_input"
    mv "integration_tests/gz_input" "compression/gz_input"
fi

# Compression test
if [ -d "integration_tests/gzip_blocks_spliting_reads" ]; then
    mkdir -p "$(dirname 'compression/gzip_blocks_spliting_reads')"
    echo "  integration_tests/gzip_blocks_spliting_reads → compression/gzip_blocks_spliting_reads"
    mv "integration_tests/gzip_blocks_spliting_reads" "compression/gzip_blocks_spliting_reads"
fi

# Hash validation test
if [ -d "integration_tests/hash_output_both" ]; then
    mkdir -p "$(dirname 'validation/hash/hash_output_both')"
    echo "  integration_tests/hash_output_both → validation/hash/hash_output_both"
    mv "integration_tests/hash_output_both" "validation/hash/hash_output_both"
fi

# Hash validation test
if [ -d "integration_tests/hash_output_compressed" ]; then
    mkdir -p "$(dirname 'validation/hash/hash_output_compressed')"
    echo "  integration_tests/hash_output_compressed → validation/hash/hash_output_compressed"
    mv "integration_tests/hash_output_compressed" "validation/hash/hash_output_compressed"
fi

# General integration test: head_with_index
if [ -d "integration_tests/head_with_index" ]; then
    mkdir -p "$(dirname 'integration/head_with_index')"
    echo "  integration_tests/head_with_index → integration/head_with_index"
    mv "integration_tests/head_with_index" "integration/head_with_index"
fi

# Demultiplexing integration test
if [ -d "integration_tests/head_with_index_and_demultiplex" ]; then
    mkdir -p "$(dirname 'demultiplex/head_with_index_and_demultiplex')"
    echo "  integration_tests/head_with_index_and_demultiplex → demultiplex/head_with_index_and_demultiplex"
    mv "integration_tests/head_with_index_and_demultiplex" "demultiplex/head_with_index_and_demultiplex"
fi

# I/O format test
if [ -d "integration_tests/input_interleaved" ]; then
    mkdir -p "$(dirname 'io/input_interleaved')"
    echo "  integration_tests/input_interleaved → io/input_interleaved"
    mv "integration_tests/input_interleaved" "io/input_interleaved"
fi

# I/O format test
if [ -d "integration_tests/input_interleaved_test_premature_termination" ]; then
    mkdir -p "$(dirname 'io/input_interleaved_test_premature_termination')"
    echo "  integration_tests/input_interleaved_test_premature_termination → io/input_interleaved_test_premature_termination"
    mv "integration_tests/input_interleaved_test_premature_termination" "io/input_interleaved_test_premature_termination"
fi

# I/O format test
if [ -d "integration_tests/interleaved_must_have_even_block_size" ]; then
    mkdir -p "$(dirname 'io/interleaved_must_have_even_block_size')"
    echo "  integration_tests/interleaved_must_have_even_block_size → io/interleaved_must_have_even_block_size"
    mv "integration_tests/interleaved_must_have_even_block_size" "io/interleaved_must_have_even_block_size"
fi

# I/O format test
if [ -d "integration_tests/interleaved_output" ]; then
    mkdir -p "$(dirname 'io/interleaved_output')"
    echo "  integration_tests/interleaved_output → io/interleaved_output"
    mv "integration_tests/interleaved_output" "io/interleaved_output"
fi

# Demultiplexing integration test
if [ -d "integration_tests/interleaved_output_demultiplex" ]; then
    mkdir -p "$(dirname 'demultiplex/interleaved_output_demultiplex')"
    echo "  integration_tests/interleaved_output_demultiplex → demultiplex/interleaved_output_demultiplex"
    mv "integration_tests/interleaved_output_demultiplex" "demultiplex/interleaved_output_demultiplex"
fi

# Filtering test: low_complexity_filter
if [ -d "integration_tests/low_complexity_filter" ]; then
    mkdir -p "$(dirname 'filter/low_complexity_filter')"
    echo "  integration_tests/low_complexity_filter → filter/low_complexity_filter"
    mv "integration_tests/low_complexity_filter" "filter/low_complexity_filter"
fi

# Length constraint test
if [ -d "integration_tests/max_len" ]; then
    mkdir -p "$(dirname 'transform/max_len')"
    echo "  integration_tests/max_len → transform/max_len"
    mv "integration_tests/max_len" "transform/max_len"
fi

# Long read test
if [ -d "integration_tests/mega_long_reads" ]; then
    mkdir -p "$(dirname 'edge_cases/mega_long_reads')"
    echo "  integration_tests/mega_long_reads → edge_cases/mega_long_reads"
    mv "integration_tests/mega_long_reads" "edge_cases/mega_long_reads"
fi

# Basic functionality test
if [ -d "integration_tests/noop" ]; then
    mkdir -p "$(dirname 'basic/noop')"
    echo "  integration_tests/noop → basic/noop"
    mv "integration_tests/noop" "basic/noop"
fi

# Basic functionality test
if [ -d "integration_tests/noop_minimal" ]; then
    mkdir -p "$(dirname 'basic/noop_minimal')"
    echo "  integration_tests/noop_minimal → basic/noop_minimal"
    mv "integration_tests/noop_minimal" "basic/noop_minimal"
fi

# Legacy CLI test
if [ -d "integration_tests/old_cli_format" ]; then
    mkdir -p "$(dirname 'compatibility/old_cli_format')"
    echo "  integration_tests/old_cli_format → compatibility/old_cli_format"
    mv "integration_tests/old_cli_format" "compatibility/old_cli_format"
fi

# Correctness test
if [ -d "integration_tests/order_maintained_in_single_core_transforms" ]; then
    mkdir -p "$(dirname 'correctness/order_maintained_in_single_core_transforms')"
    echo "  integration_tests/order_maintained_in_single_core_transforms → correctness/order_maintained_in_single_core_transforms"
    mv "integration_tests/order_maintained_in_single_core_transforms" "correctness/order_maintained_in_single_core_transforms"
fi

# Compression test
if [ -d "integration_tests/output_compression_gzip_level" ]; then
    mkdir -p "$(dirname 'compression/output_compression_gzip_level')"
    echo "  integration_tests/output_compression_gzip_level → compression/output_compression_gzip_level"
    mv "integration_tests/output_compression_gzip_level" "compression/output_compression_gzip_level"
fi

# Output configuration test
if [ -d "integration_tests/output_different_suffix" ]; then
    mkdir -p "$(dirname 'output/output_different_suffix')"
    echo "  integration_tests/output_different_suffix → output/output_different_suffix"
    mv "integration_tests/output_different_suffix" "output/output_different_suffix"
fi

# Output configuration test
if [ -d "integration_tests/output_neither_r1_nor_r2" ]; then
    mkdir -p "$(dirname 'output/output_neither_r1_nor_r2')"
    echo "  integration_tests/output_neither_r1_nor_r2 → output/output_neither_r1_nor_r2"
    mv "integration_tests/output_neither_r1_nor_r2" "output/output_neither_r1_nor_r2"
fi

# Output configuration test
if [ -d "integration_tests/output_neither_r1_nor_r2_but_index" ]; then
    mkdir -p "$(dirname 'output/output_neither_r1_nor_r2_but_index')"
    echo "  integration_tests/output_neither_r1_nor_r2_but_index → output/output_neither_r1_nor_r2_but_index"
    mv "integration_tests/output_neither_r1_nor_r2_but_index" "output/output_neither_r1_nor_r2_but_index"
fi

# Output configuration test
if [ -d "integration_tests/output_neither_r1_nor_r2_but_index2" ]; then
    mkdir -p "$(dirname 'output/output_neither_r1_nor_r2_but_index2')"
    echo "  integration_tests/output_neither_r1_nor_r2_but_index2 → output/output_neither_r1_nor_r2_but_index2"
    mv "integration_tests/output_neither_r1_nor_r2_but_index2" "output/output_neither_r1_nor_r2_but_index2"
fi

# Output configuration test
if [ -d "integration_tests/output_r1_only" ]; then
    mkdir -p "$(dirname 'output/output_r1_only')"
    echo "  integration_tests/output_r1_only → output/output_r1_only"
    mv "integration_tests/output_r1_only" "output/output_r1_only"
fi

# Output configuration test
if [ -d "integration_tests/output_r2_only" ]; then
    mkdir -p "$(dirname 'output/output_r2_only')"
    echo "  integration_tests/output_r2_only → output/output_r2_only"
    mv "integration_tests/output_r2_only" "output/output_r2_only"
fi

# Prefix/postfix test
if [ -d "integration_tests/prefix_and_postfix" ]; then
    mkdir -p "$(dirname 'transform/prefix_and_postfix')"
    echo "  integration_tests/prefix_and_postfix → transform/prefix_and_postfix"
    mv "integration_tests/prefix_and_postfix" "transform/prefix_and_postfix"
fi

# Quality/complexity test
if [ -d "integration_tests/quality_base_replacement" ]; then
    mkdir -p "$(dirname 'quality/quality_base_replacement')"
    echo "  integration_tests/quality_base_replacement → quality/quality_base_replacement"
    mv "integration_tests/quality_base_replacement" "quality/quality_base_replacement"
fi

# Quantification test
if [ -d "integration_tests/quantify_regions_multi" ]; then
    mkdir -p "$(dirname 'calc/quantify_regions_multi')"
    echo "  integration_tests/quantify_regions_multi → calc/quantify_regions_multi"
    mv "integration_tests/quantify_regions_multi" "calc/quantify_regions_multi"
fi

# Quantification test
if [ -d "integration_tests/quantify_regions_simple" ]; then
    mkdir -p "$(dirname 'calc/quantify_regions_simple')"
    echo "  integration_tests/quantify_regions_simple → calc/quantify_regions_simple"
    mv "integration_tests/quantify_regions_simple" "calc/quantify_regions_simple"
fi

# Renaming test
if [ -d "integration_tests/rename_read_index_placeholder" ]; then
    mkdir -p "$(dirname 'transform/rename_read_index_placeholder')"
    echo "  integration_tests/rename_read_index_placeholder → transform/rename_read_index_placeholder"
    mv "integration_tests/rename_read_index_placeholder" "transform/rename_read_index_placeholder"
fi

# Renaming test
if [ -d "integration_tests/rename_regex" ]; then
    mkdir -p "$(dirname 'transform/rename_regex')"
    echo "  integration_tests/rename_regex → transform/rename_regex"
    mv "integration_tests/rename_regex" "transform/rename_regex"
fi

# Renaming test
if [ -d "integration_tests/rename_regex_gets_longer" ]; then
    mkdir -p "$(dirname 'transform/rename_regex_gets_longer')"
    echo "  integration_tests/rename_regex_gets_longer → transform/rename_regex_gets_longer"
    mv "integration_tests/rename_regex_gets_longer" "transform/rename_regex_gets_longer"
fi

# Renaming test
if [ -d "integration_tests/rename_regex_shorter" ]; then
    mkdir -p "$(dirname 'transform/rename_regex_shorter')"
    echo "  integration_tests/rename_regex_shorter → transform/rename_regex_shorter"
    mv "integration_tests/rename_regex_shorter" "transform/rename_regex_shorter"
fi

# Reverse complement test
if [ -d "integration_tests/reverse_complement" ]; then
    mkdir -p "$(dirname 'transform/reverse_complement')"
    echo "  integration_tests/reverse_complement → transform/reverse_complement"
    mv "integration_tests/reverse_complement" "transform/reverse_complement"
fi

# Sampling test
if [ -d "integration_tests/skip" ]; then
    mkdir -p "$(dirname 'sampling/skip')"
    echo "  integration_tests/skip → sampling/skip"
    mv "integration_tests/skip" "sampling/skip"
fi

# I/O format test
if [ -d "integration_tests/stdout_output_interleaved" ]; then
    mkdir -p "$(dirname 'io/stdout_output_interleaved')"
    echo "  integration_tests/stdout_output_interleaved → io/stdout_output_interleaved"
    mv "integration_tests/stdout_output_interleaved" "io/stdout_output_interleaved"
fi

# Sampling test
if [ -d "integration_tests/subsample" ]; then
    mkdir -p "$(dirname 'sampling/subsample')"
    echo "  integration_tests/subsample → sampling/subsample"
    mv "integration_tests/subsample" "sampling/subsample"
fi

# Swap test
if [ -d "integration_tests/swap_auto_detect_two_segments" ]; then
    mkdir -p "$(dirname 'swap/swap_auto_detect_two_segments')"
    echo "  integration_tests/swap_auto_detect_two_segments → swap/swap_auto_detect_two_segments"
    mv "integration_tests/swap_auto_detect_two_segments" "swap/swap_auto_detect_two_segments"
fi

# Multi-segment test
if [ -d "integration_tests/ten_segments_creative_transforms" ]; then
    mkdir -p "$(dirname 'edge_cases/ten_segments_creative_transforms')"
    echo "  integration_tests/ten_segments_creative_transforms → edge_cases/ten_segments_creative_transforms"
    mv "integration_tests/ten_segments_creative_transforms" "edge_cases/ten_segments_creative_transforms"
fi

# Trimming/cutting test: trim_poly_tail_detail
if [ -d "integration_tests/trim_poly_tail_detail" ]; then
    mkdir -p "$(dirname 'trim/trim_poly_tail_detail')"
    echo "  integration_tests/trim_poly_tail_detail → trim/trim_poly_tail_detail"
    mv "integration_tests/trim_poly_tail_detail" "trim/trim_poly_tail_detail"
fi

# Trimming/cutting test: trim_poly_tail_detail_g
if [ -d "integration_tests/trim_poly_tail_detail_g" ]; then
    mkdir -p "$(dirname 'trim/trim_poly_tail_detail_g')"
    echo "  integration_tests/trim_poly_tail_detail_g → trim/trim_poly_tail_detail_g"
    mv "integration_tests/trim_poly_tail_detail_g" "trim/trim_poly_tail_detail_g"
fi

# Trimming/cutting test: trim_poly_tail_long
if [ -d "integration_tests/trim_poly_tail_long" ]; then
    mkdir -p "$(dirname 'trim/trim_poly_tail_long')"
    echo "  integration_tests/trim_poly_tail_long → trim/trim_poly_tail_long"
    mv "integration_tests/trim_poly_tail_long" "trim/trim_poly_tail_long"
fi

# Trimming/cutting test: trim_poly_tail_n
if [ -d "integration_tests/trim_poly_tail_n" ]; then
    mkdir -p "$(dirname 'trim/trim_poly_tail_n')"
    echo "  integration_tests/trim_poly_tail_n → trim/trim_poly_tail_n"
    mv "integration_tests/trim_poly_tail_n" "trim/trim_poly_tail_n"
fi

# Trimming/cutting test: trim_qual_end
if [ -d "integration_tests/trim_qual_end" ]; then
    mkdir -p "$(dirname 'trim/trim_qual_end')"
    echo "  integration_tests/trim_qual_end → trim/trim_qual_end"
    mv "integration_tests/trim_qual_end" "trim/trim_qual_end"
fi

# Trimming/cutting test: trim_qual_start
if [ -d "integration_tests/trim_qual_start" ]; then
    mkdir -p "$(dirname 'trim/trim_qual_start')"
    echo "  integration_tests/trim_qual_start → trim/trim_qual_start"
    mv "integration_tests/trim_qual_start" "trim/trim_qual_start"
fi

# Long read test
if [ -d "integration_tests/very_long_reads" ]; then
    mkdir -p "$(dirname 'edge_cases/very_long_reads')"
    echo "  integration_tests/very_long_reads → edge_cases/very_long_reads"
    mv "integration_tests/very_long_reads" "edge_cases/very_long_reads"
fi

# Compression test
if [ -d "integration_tests/zstd_input" ]; then
    mkdir -p "$(dirname 'compression/zstd_input')"
    echo "  integration_tests/zstd_input → compression/zstd_input"
    mv "integration_tests/zstd_input" "compression/zstd_input"
fi

# Compression test
if [ -d "integration_tests/zstd_input_gzip_output" ]; then
    mkdir -p "$(dirname 'compression/zstd_input_gzip_output')"
    echo "  integration_tests/zstd_input_gzip_output → compression/zstd_input_gzip_output"
    mv "integration_tests/zstd_input_gzip_output" "compression/zstd_input_gzip_output"
fi

# Compression test
if [ -d "integration_tests/zstd_input_read_swap" ]; then
    mkdir -p "$(dirname 'compression/zstd_input_read_swap')"
    echo "  integration_tests/zstd_input_read_swap → compression/zstd_input_read_swap"
    mv "integration_tests/zstd_input_read_swap" "compression/zstd_input_read_swap"
fi

# Compression test
if [ -d "integration_tests/zstd_input_zst_output" ]; then
    mkdir -p "$(dirname 'compression/zstd_input_zst_output')"
    echo "  integration_tests/zstd_input_zst_output → compression/zstd_input_zst_output"
    mv "integration_tests/zstd_input_zst_output" "compression/zstd_input_zst_output"
fi


# memory/ (1 tests)
echo "Processing memory/..."

# Memory/performance test: duplicate_input_allocation
if [ -d "memory/duplicate_input_allocation" ]; then
    mkdir -p "$(dirname 'performance/duplicate_input_allocation')"
    echo "  memory/duplicate_input_allocation → performance/duplicate_input_allocation"
    mv "memory/duplicate_input_allocation" "performance/duplicate_input_allocation"
fi


# output/ (2 tests)
echo "Processing output/..."


# outside_error_conditions/ (2 tests)
echo "Processing outside_error_conditions/..."

# BAM file errors
if [ -d "outside_error_conditions/disk_full_bam" ]; then
    mkdir -p "$(dirname 'error_handling/bam/disk_full_bam')"
    echo "  outside_error_conditions/disk_full_bam → error_handling/bam/disk_full_bam"
    mv "outside_error_conditions/disk_full_bam" "error_handling/bam/disk_full_bam"
fi

# Miscellaneous validation errors
if [ -d "outside_error_conditions/disk_full_fastq" ]; then
    mkdir -p "$(dirname 'error_handling/misc/disk_full_fastq')"
    echo "  outside_error_conditions/disk_full_fastq → error_handling/misc/disk_full_fastq"
    mv "outside_error_conditions/disk_full_fastq" "error_handling/misc/disk_full_fastq"
fi


# reports/ (10 tests)
echo "Processing reports/..."


# validation/ (11 tests)
echo "Processing validation/..."


echo ""
echo "${GREEN}Reorganization complete!${NC}"
echo "Moved 312 test directories."
echo ""
echo "${YELLOW}Next steps:${NC}"
echo "1. Run: dev/update_tests.py"
echo "2. Run: cargo test"
echo "3. Commit the changes"
