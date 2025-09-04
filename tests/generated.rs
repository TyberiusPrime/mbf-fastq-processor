
// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;

#[test]
fn test_case_demultiplex_simple_demultiplex_basics() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_basics"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_combined_outputs() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_combined_outputs"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_hamming() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_hamming"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_iupac() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_iupac"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_iupac_hamming() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_iupac_hamming"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_no_unmatched() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_no_unmatched"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_single_barcode() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_single_barcode"));
}

#[test]
fn test_case_demultiplex_simple_demultiplex_single_barcode_no_unmatched_output() {
    run_test(std::path::Path::new("test_cases/demultiplex/simple_demultiplex_single_barcode_no_unmatched_output"));
}

#[test]
fn test_case_edits_lowercase_sequence() {
    run_test(std::path::Path::new("test_cases/edits/lowercase_sequence"));
}

#[test]
fn test_case_edits_lowercase_tag() {
    run_test(std::path::Path::new("test_cases/edits/lowercase_tag"));
}

#[test]
fn test_case_edits_uppercase_sequence() {
    run_test(std::path::Path::new("test_cases/edits/uppercase_sequence"));
}

#[test]
fn test_case_edits_uppercase_tag() {
    run_test(std::path::Path::new("test_cases/edits/uppercase_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_cut_end_inside_tag() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/cut_end_inside_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_cut_start_inside_tag() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/cut_start_inside_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_extract_trim_end_false() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/extract_trim_end_false"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_extract_trim_end_true() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/extract_trim_end_true"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_extract_trim_start_false() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/extract_trim_start_false"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_extract_trim_start_true() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/extract_trim_start_true"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_max_len_after_tag() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/max_len_after_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_max_len_before_tag() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/max_len_before_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_max_len_inside_tag() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/max_len_inside_tag"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_prefix() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/prefix"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_rev_complement() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/rev_complement"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_swap_r1_and_r2() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/swap_r1_and_r2"));
}

#[test]
fn test_case_extraction_edits_altering_tag_locations_trim_quality_start() {
    run_test(std::path::Path::new("test_cases/extraction/edits_altering_tag_locations/trim_quality_start"));
}

#[test]
fn test_case_extraction_extract_anchor_hamming() {
    run_test(std::path::Path::new("test_cases/extraction/extract_anchor/hamming"));
}

#[test]
fn test_case_extraction_extract_anchor_leftmost_verification() {
    run_test(std::path::Path::new("test_cases/extraction/extract_anchor/leftmost_verification"));
}

#[test]
fn test_case_extraction_extract_anchor_simple() {
    run_test(std::path::Path::new("test_cases/extraction/extract_anchor/simple"));
}

#[test]
fn test_case_extraction_extract_anchor_too_far() {
    run_test(std::path::Path::new("test_cases/extraction/extract_anchor/too_far"));
}

#[test]
fn test_case_extraction_extract_anchor_too_far_left() {
    run_test(std::path::Path::new("test_cases/extraction/extract_anchor/too_far_left"));
}

#[test]
fn test_case_extraction_extract_filter_keep() {
    run_test(std::path::Path::new("test_cases/extraction/extract_filter_keep"));
}

#[test]
fn test_case_extraction_extract_filter_remove() {
    run_test(std::path::Path::new("test_cases/extraction/extract_filter_remove"));
}

#[test]
fn test_case_extraction_extract_growing() {
    run_test(std::path::Path::new("test_cases/extraction/extract_growing"));
}

#[test]
fn test_case_extraction_extract_growing_from_nothing() {
    run_test(std::path::Path::new("test_cases/extraction/extract_growing_from_nothing"));
}

#[test]
fn test_case_extraction_extract_highlight() {
    run_test(std::path::Path::new("test_cases/extraction/extract_highlight"));
}

#[test]
fn test_case_extraction_extract_highlight_regex() {
    run_test(std::path::Path::new("test_cases/extraction/extract_highlight_regex"));
}

#[test]
fn test_case_extraction_extract_label_must_not_be_empty() {
    run_test(std::path::Path::new("test_cases/extraction/extract_label_must_not_be_empty"));
}

#[test]
fn test_case_extraction_extract_length() {
    run_test(std::path::Path::new("test_cases/extraction/extract_length"));
}

#[test]
fn test_case_extraction_extract_length_panic_on_store_in_seq() {
    run_test(std::path::Path::new("test_cases/extraction/extract_length_panic_on_store_in_seq"));
}

#[test]
fn test_case_extraction_extract_regex() {
    run_test(std::path::Path::new("test_cases/extraction/extract_regex"));
}

#[test]
fn test_case_extraction_extract_regex_underscores() {
    run_test(std::path::Path::new("test_cases/extraction/extract_regex_underscores"));
}

#[test]
fn test_case_extraction_extract_regex_underscores_ok_works() {
    run_test(std::path::Path::new("test_cases/extraction/extract_regex_underscores/ok_works"));
}

#[test]
fn test_case_extraction_extract_region_and_replace_multiple() {
    run_test(std::path::Path::new("test_cases/extraction/extract_region_and_replace_multiple"));
}

#[test]
fn test_case_extraction_extract_region_beyond_read_len() {
    run_test(std::path::Path::new("test_cases/extraction/extract_region_beyond_read_len"));
}

#[test]
fn test_case_extraction_extract_region_beyond_read_len_and_trim() {
    run_test(std::path::Path::new("test_cases/extraction/extract_region_beyond_read_len_and_trim"));
}

#[test]
fn test_case_extraction_extract_region_trim_at_tag_conflict() {
    run_test(std::path::Path::new("test_cases/extraction/extract_region_trim_at_tag_conflict"));
}

#[test]
fn test_case_extraction_extract_shrinking() {
    run_test(std::path::Path::new("test_cases/extraction/extract_shrinking"));
}

#[test]
fn test_case_extraction_extract_tag() {
    run_test(std::path::Path::new("test_cases/extraction/extract_tag"));
}

#[test]
fn test_case_extraction_extract_tag_duplicate_name_panics() {
    run_test(std::path::Path::new("test_cases/extraction/extract_tag_duplicate_name_panics"));
}

#[test]
fn test_case_extraction_extract_tag_i1_i2() {
    run_test(std::path::Path::new("test_cases/extraction/extract_tag_i1_i2"));
}

#[test]
fn test_case_extraction_extract_tag_r2() {
    run_test(std::path::Path::new("test_cases/extraction/extract_tag_r2"));
}

#[test]
fn test_case_extraction_extract_tag_reserved_name_panics() {
    run_test(std::path::Path::new("test_cases/extraction/extract_tag_reserved_name_panics"));
}

#[test]
fn test_case_extraction_extract_trim_end_false() {
    run_test(std::path::Path::new("test_cases/extraction/extract_trim_end_false"));
}

#[test]
fn test_case_extraction_extract_trim_end_true() {
    run_test(std::path::Path::new("test_cases/extraction/extract_trim_end_true"));
}

#[test]
fn test_case_extraction_extract_trim_start_false() {
    run_test(std::path::Path::new("test_cases/extraction/extract_trim_start_false"));
}

#[test]
fn test_case_extraction_extract_trim_start_true() {
    run_test(std::path::Path::new("test_cases/extraction/extract_trim_start_true"));
}

#[test]
fn test_case_extraction_filter_no_such_tag() {
    run_test(std::path::Path::new("test_cases/extraction/filter_no_such_tag"));
}

#[test]
fn test_case_extraction_remove_nonexistant_tag() {
    run_test(std::path::Path::new("test_cases/extraction/remove_nonexistant_tag"));
}

#[test]
fn test_case_extraction_store_tags_in_tsv() {
    run_test(std::path::Path::new("test_cases/extraction/store_tags_in_tsv"));
}

#[test]
fn test_case_extraction_store_tags_in_tsv_gz() {
    run_test(std::path::Path::new("test_cases/extraction/store_tags_in_tsv_gz"));
}

#[test]
fn test_case_extraction_store_tags_in_tsv_validate_compression() {
    run_test(std::path::Path::new("test_cases/extraction/store_tags_in_tsv_validate_compression"));
}

#[test]
fn test_case_extraction_umi_extract() {
    run_test(std::path::Path::new("test_cases/extraction/umi_extract"));
}

#[test]
fn test_case_extraction_umi_extract_store_in_all_read_names() {
    run_test(std::path::Path::new("test_cases/extraction/umi_extract_store_in_all_read_names"));
}

#[test]
fn test_case_extraction_umi_extract_with_existing_comment() {
    run_test(std::path::Path::new("test_cases/extraction/umi_extract_with_existing_comment"));
}

#[test]
fn test_case_extraction_use_removed_tag() {
    run_test(std::path::Path::new("test_cases/extraction/use_removed_tag"));
}

#[test]
fn test_case_head_early_termination_head_after_quantify() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_after_quantify"));
}

#[test]
fn test_case_head_early_termination_head_after_report() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_after_report"));
}

#[test]
fn test_case_head_early_termination_head_before_quantify() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_before_quantify"));
}

#[test]
fn test_case_head_early_termination_head_before_report() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_before_report"));
}

#[test]
fn test_case_head_early_termination_head_stops_reading() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_stops_reading"));
}

#[test]
fn test_case_head_early_termination_head_stops_reading_multiple() {
    run_test(std::path::Path::new("test_cases/head_early_termination/head_stops_reading_multiple"));
}

#[test]
fn test_case_head_early_termination_multi_stage_head() {
    run_test(std::path::Path::new("test_cases/head_early_termination/multi_stage_head"));
}

#[test]
fn test_case_head_early_termination_multi_stage_head_report_bottom() {
    run_test(std::path::Path::new("test_cases/head_early_termination/multi_stage_head_report_bottom"));
}

#[test]
fn test_case_head_early_termination_multi_stage_head_report_middle() {
    run_test(std::path::Path::new("test_cases/head_early_termination/multi_stage_head_report_middle"));
}

#[test]
fn test_case_head_early_termination_multi_stage_head_report_middle_bottom() {
    run_test(std::path::Path::new("test_cases/head_early_termination/multi_stage_head_report_middle_bottom"));
}

#[test]
fn test_case_head_early_termination_multi_stage_head_report_top() {
    run_test(std::path::Path::new("test_cases/head_early_termination/multi_stage_head_report_top"));
}

#[test]
fn test_case_input_validation_adapter_mismatch_tail_min_length_too_high() {
    run_test(std::path::Path::new("test_cases/input_validation/adapter_mismatch_tail_min_length_too_high"));
}

#[test]
fn test_case_input_validation_adapter_mismatch_tail_too_many_mismatches() {
    run_test(std::path::Path::new("test_cases/input_validation/adapter_mismatch_tail_too_many_mismatches"));
}

#[test]
fn test_case_input_validation_barcode_outputs_not_named_no_barcode() {
    run_test(std::path::Path::new("test_cases/input_validation/barcode_outputs_not_named_no_barcode"));
}

#[test]
fn test_case_input_validation_broken_newline() {
    run_test(std::path::Path::new("test_cases/input_validation/broken_newline"));
}

#[test]
fn test_case_input_validation_broken_newline2() {
    run_test(std::path::Path::new("test_cases/input_validation/broken_newline2"));
}

#[test]
fn test_case_input_validation_broken_panics() {
    run_test(std::path::Path::new("test_cases/input_validation/broken_panics"));
}

#[test]
fn test_case_input_validation_convert_phred_raises() {
    run_test(std::path::Path::new("test_cases/input_validation/convert_phred_raises"));
}

#[test]
fn test_case_input_validation_dna_validation_count_oligos_non_agtc() {
    run_test(std::path::Path::new("test_cases/input_validation/dna_validation_count_oligos_non_agtc"));
}

#[test]
fn test_case_input_validation_dna_validation_count_oligos_non_empty() {
    run_test(std::path::Path::new("test_cases/input_validation/dna_validation_count_oligos_non_empty"));
}

#[test]
fn test_case_input_validation_empty_name_input() {
    run_test(std::path::Path::new("test_cases/input_validation/empty_name_input"));
}

#[test]
fn test_case_input_validation_existing_unwritable_output_file() {
    run_test(std::path::Path::new("test_cases/input_validation/existing_unwritable_output_file"));
}

#[test]
fn test_case_input_validation_extract_tag_from_i1_i2_no_i1_i2() {
    run_test(std::path::Path::new("test_cases/input_validation/extract_tag_from_i1_i2_no_i1_i2"));
}

#[test]
fn test_case_input_validation_extract_tag_i1_i2_but_not_output() {
    run_test(std::path::Path::new("test_cases/input_validation/extract_tag_i1_i2_but_not_output"));
}

#[test]
fn test_case_input_validation_index1_file_does_not_exist() {
    run_test(std::path::Path::new("test_cases/input_validation/index1_file_does_not_exist"));
}

#[test]
fn test_case_input_validation_index2_but_not_1() {
    run_test(std::path::Path::new("test_cases/input_validation/index2_but_not_1"));
}

#[test]
fn test_case_input_validation_index2_file_does_not_exist() {
    run_test(std::path::Path::new("test_cases/input_validation/index2_file_does_not_exist"));
}

#[test]
fn test_case_input_validation_input_read2_interleaved_conflict() {
    run_test(std::path::Path::new("test_cases/input_validation/input_read2_interleaved_conflict"));
}

#[test]
fn test_case_input_validation_interleave_no_read2() {
    run_test(std::path::Path::new("test_cases/input_validation/interleave_no_read2"));
}

#[test]
fn test_case_input_validation_invalid_base() {
    run_test(std::path::Path::new("test_cases/input_validation/invalid_base"));
}

#[test]
fn test_case_input_validation_invalid_base_or_dot() {
    run_test(std::path::Path::new("test_cases/input_validation/invalid_base_or_dot"));
}

#[test]
fn test_case_input_validation_invalid_base_or_dot_too_long() {
    run_test(std::path::Path::new("test_cases/input_validation/invalid_base_or_dot_too_long"));
}

#[test]
fn test_case_input_validation_missing_input_file() {
    run_test(std::path::Path::new("test_cases/input_validation/missing_input_file"));
}

#[test]
fn test_case_input_validation_no_newline_and_truncated_qual() {
    run_test(std::path::Path::new("test_cases/input_validation/no_newline_and_truncated_qual"));
}

#[test]
fn test_case_input_validation_no_newline_at_end_ok() {
    run_test(std::path::Path::new("test_cases/input_validation/no_newline_at_end_ok"));
}

#[test]
fn test_case_input_validation_only_one_demultiplex() {
    run_test(std::path::Path::new("test_cases/input_validation/only_one_demultiplex"));
}

#[test]
fn test_case_input_validation_permission_denied_input_file() {
    run_test(std::path::Path::new("test_cases/input_validation/permission_denied_input_file"));
}

#[test]
fn test_case_input_validation_permission_denied_read1() {
    run_test(std::path::Path::new("test_cases/input_validation/permission_denied_read1"));
}

#[test]
fn test_case_input_validation_postfix_len_mismatch() {
    run_test(std::path::Path::new("test_cases/input_validation/postfix_len_mismatch"));
}

#[test]
fn test_case_input_validation_prefix_len_mismatch() {
    run_test(std::path::Path::new("test_cases/input_validation/prefix_len_mismatch"));
}

#[test]
fn test_case_input_validation_read1_empty_list() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_empty_list"));
}

#[test]
fn test_case_input_validation_read1_file_does_not_exist() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_file_does_not_exist"));
}

#[test]
fn test_case_input_validation_read1_len_neq_index1_len() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_len_neq_index1_len"));
}

#[test]
fn test_case_input_validation_read1_len_neq_index2_len() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_len_neq_index2_len"));
}

#[test]
fn test_case_input_validation_read1_len_neq_read2_len() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_len_neq_read2_len"));
}

#[test]
fn test_case_input_validation_read1_not_a_string() {
    run_test(std::path::Path::new("test_cases/input_validation/read1_not_a_string"));
}

#[test]
fn test_case_input_validation_read2_but_not_read1_set() {
    run_test(std::path::Path::new("test_cases/input_validation/read2_but_not_read1_set"));
}

#[test]
fn test_case_input_validation_read2_file_does_not_exist() {
    run_test(std::path::Path::new("test_cases/input_validation/read2_file_does_not_exist"));
}

#[test]
fn test_case_input_validation_read2_not_a_string() {
    run_test(std::path::Path::new("test_cases/input_validation/read2_not_a_string"));
}

#[test]
fn test_case_input_validation_repeated_filenames() {
    run_test(std::path::Path::new("test_cases/input_validation/repeated_filenames"));
}

#[test]
fn test_case_input_validation_repeated_filenames_index1() {
    run_test(std::path::Path::new("test_cases/input_validation/repeated_filenames_index1"));
}

#[test]
fn test_case_input_validation_repeated_filenames_index2() {
    run_test(std::path::Path::new("test_cases/input_validation/repeated_filenames_index2"));
}

#[test]
fn test_case_input_validation_repeated_filenames_one_key() {
    run_test(std::path::Path::new("test_cases/input_validation/repeated_filenames_one_key"));
}

#[test]
fn test_case_input_validation_report_but_no_report_step_html() {
    run_test(std::path::Path::new("test_cases/input_validation/report_but_no_report_step_html"));
}

#[test]
fn test_case_input_validation_report_but_no_report_step_json() {
    run_test(std::path::Path::new("test_cases/input_validation/report_but_no_report_step_json"));
}

#[test]
fn test_case_input_validation_report_names_distinct() {
    run_test(std::path::Path::new("test_cases/input_validation/report_names_distinct"));
}

#[test]
fn test_case_input_validation_stdout_conflict() {
    run_test(std::path::Path::new("test_cases/input_validation/stdout_conflict"));
}

#[test]
fn test_case_input_validation_store_tags_in_table_no_tags_defined() {
    run_test(std::path::Path::new("test_cases/input_validation/store_tags_in_table_no_tags_defined"));
}

#[test]
fn test_case_input_validation_truncated_after_at() {
    run_test(std::path::Path::new("test_cases/input_validation/truncated_after_at"));
}

#[test]
fn test_case_input_validation_two_mistakes_eserde() {
    run_test(std::path::Path::new("test_cases/input_validation/two_mistakes_eserde"));
}

#[test]
fn test_case_input_validation_two_mistakes_post_deserialization() {
    run_test(std::path::Path::new("test_cases/input_validation/two_mistakes_post_deserialization"));
}

#[test]
fn test_case_input_validation_u8_from_char_number_to_large() {
    run_test(std::path::Path::new("test_cases/input_validation/u8_from_char_number_to_large"));
}

#[test]
fn test_case_input_validation_u8_from_char_too_many_chars() {
    run_test(std::path::Path::new("test_cases/input_validation/u8_from_char_too_many_chars"));
}

#[test]
fn test_case_input_validation_validate_regex_fail() {
    run_test(std::path::Path::new("test_cases/input_validation/validate_regex_fail"));
}

#[test]
fn test_case_integration_tests_cat() {
    run_test(std::path::Path::new("test_cases/integration_tests/cat"));
}

#[test]
fn test_case_integration_tests_convert_phred() {
    run_test(std::path::Path::new("test_cases/integration_tests/convert_phred"));
}

#[test]
fn test_case_integration_tests_convert_phred_multi() {
    run_test(std::path::Path::new("test_cases/integration_tests/convert_phred_multi"));
}

#[test]
fn test_case_integration_tests_cut_end() {
    run_test(std::path::Path::new("test_cases/integration_tests/cut_end"));
}

#[test]
fn test_case_integration_tests_cut_start() {
    run_test(std::path::Path::new("test_cases/integration_tests/cut_start"));
}

#[test]
fn test_case_integration_tests_dedup() {
    run_test(std::path::Path::new("test_cases/integration_tests/dedup"));
}

#[test]
fn test_case_integration_tests_dedup_exact() {
    run_test(std::path::Path::new("test_cases/integration_tests/dedup_exact"));
}

#[test]
fn test_case_integration_tests_dedup_read2() {
    run_test(std::path::Path::new("test_cases/integration_tests/dedup_read2"));
}

#[test]
fn test_case_integration_tests_dedup_read_combo() {
    run_test(std::path::Path::new("test_cases/integration_tests/dedup_read_combo"));
}

#[test]
fn test_case_integration_tests_dedup_read_combo_incl_index() {
    run_test(std::path::Path::new("test_cases/integration_tests/dedup_read_combo_incl_index"));
}

#[test]
fn test_case_integration_tests_filter_avg_quality() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_avg_quality"));
}

#[test]
fn test_case_integration_tests_filter_empty() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_empty"));
}

#[test]
fn test_case_integration_tests_filter_empty_all() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_empty_all"));
}

#[test]
fn test_case_integration_tests_filter_empty_segments() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_empty_segments"));
}

#[test]
fn test_case_integration_tests_filter_max_len() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_max_len"));
}

#[test]
fn test_case_integration_tests_filter_min_len() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_min_len"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_name_keep() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_name_keep"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_name_remove() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_name_remove"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_name_remove_bam() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_name_remove_bam"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_name_remove_bam_unaligned() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_name_remove_bam_unaligned"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_name_remove_bam_unaligned_no_ignore() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_name_remove_bam_unaligned_no_ignore"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_seq_keep() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_seq_keep"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_seq_remove() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_seq_remove"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_seq_remove_bam() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_seq_remove_bam"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_seq_remove_bam_unaligned() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_seq_remove_bam_unaligned"));
}

#[test]
fn test_case_integration_tests_filter_other_file_by_seq_remove_bam_unaligned_no_ignore() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_other_file_by_seq_remove_bam_unaligned_no_ignore"));
}

#[test]
fn test_case_integration_tests_filter_qualified_bases() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_qualified_bases"));
}

#[test]
fn test_case_integration_tests_filter_too_many_n() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_too_many_n"));
}

#[test]
fn test_case_integration_tests_filter_too_many_n_all() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_too_many_n_all"));
}

#[test]
fn test_case_integration_tests_filter_too_many_n_segments_vs_all() {
    run_test(std::path::Path::new("test_cases/integration_tests/filter_too_many_n_segments_vs_all"));
}

#[test]
fn test_case_integration_tests_gz_input() {
    run_test(std::path::Path::new("test_cases/integration_tests/gz_input"));
}

#[test]
fn test_case_integration_tests_gzip_blocks_spliting_reads() {
    run_test(std::path::Path::new("test_cases/integration_tests/gzip_blocks_spliting_reads"));
}

#[test]
fn test_case_integration_tests_hash_output_both() {
    run_test(std::path::Path::new("test_cases/integration_tests/hash_output_both"));
}

#[test]
fn test_case_integration_tests_hash_output_compressed() {
    run_test(std::path::Path::new("test_cases/integration_tests/hash_output_compressed"));
}

#[test]
fn test_case_integration_tests_head_with_index() {
    run_test(std::path::Path::new("test_cases/integration_tests/head_with_index"));
}

#[test]
fn test_case_integration_tests_head_with_index_and_demultiplex() {
    run_test(std::path::Path::new("test_cases/integration_tests/head_with_index_and_demultiplex"));
}

#[test]
fn test_case_integration_tests_input_interleaved() {
    run_test(std::path::Path::new("test_cases/integration_tests/input_interleaved"));
}

#[test]
fn test_case_integration_tests_input_interleaved_test_premature_termination() {
    run_test(std::path::Path::new("test_cases/integration_tests/input_interleaved_test_premature_termination"));
}

#[test]
fn test_case_integration_tests_inspect_compression_zstd_level() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_compression_zstd_level"));
}

#[test]
fn test_case_integration_tests_inspect_index1() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_index1"));
}

#[test]
fn test_case_integration_tests_inspect_index2() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_index2"));
}

#[test]
fn test_case_integration_tests_inspect_read1() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_read1"));
}

#[test]
fn test_case_integration_tests_inspect_read1_compressed() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_read1_compressed"));
}

#[test]
fn test_case_integration_tests_inspect_read2() {
    run_test(std::path::Path::new("test_cases/integration_tests/inspect_read2"));
}

#[test]
fn test_case_integration_tests_interleaved_must_have_even_block_size() {
    run_test(std::path::Path::new("test_cases/integration_tests/interleaved_must_have_even_block_size"));
}

#[test]
fn test_case_integration_tests_interleaved_output() {
    run_test(std::path::Path::new("test_cases/integration_tests/interleaved_output"));
}

#[test]
fn test_case_integration_tests_interleaved_output_demultiplex() {
    run_test(std::path::Path::new("test_cases/integration_tests/interleaved_output_demultiplex"));
}

#[test]
fn test_case_integration_tests_low_complexity_filter() {
    run_test(std::path::Path::new("test_cases/integration_tests/low_complexity_filter"));
}

#[test]
fn test_case_integration_tests_max_len() {
    run_test(std::path::Path::new("test_cases/integration_tests/max_len"));
}

#[test]
fn test_case_integration_tests_mega_long_reads() {
    run_test(std::path::Path::new("test_cases/integration_tests/mega_long_reads"));
}

#[test]
fn test_case_integration_tests_noop() {
    run_test(std::path::Path::new("test_cases/integration_tests/noop"));
}

#[test]
fn test_case_integration_tests_noop_minimal() {
    run_test(std::path::Path::new("test_cases/integration_tests/noop_minimal"));
}

#[test]
fn test_case_integration_tests_order_maintained_in_single_core_transforms() {
    run_test(std::path::Path::new("test_cases/integration_tests/order_maintained_in_single_core_transforms"));
}

#[test]
fn test_case_integration_tests_output_compression_gzip_level() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_compression_gzip_level"));
}

#[test]
fn test_case_integration_tests_output_neither_r1_nor_r2() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_neither_r1_nor_r2"));
}

#[test]
fn test_case_integration_tests_output_neither_r1_nor_r2_but_index() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_neither_r1_nor_r2_but_index"));
}

#[test]
fn test_case_integration_tests_output_neither_r1_nor_r2_but_index2() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_neither_r1_nor_r2_but_index2"));
}

#[test]
fn test_case_integration_tests_output_r1_only() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_r1_only"));
}

#[test]
fn test_case_integration_tests_output_r2_only() {
    run_test(std::path::Path::new("test_cases/integration_tests/output_r2_only"));
}

#[test]
fn test_case_integration_tests_prefix_and_postfix() {
    run_test(std::path::Path::new("test_cases/integration_tests/prefix_and_postfix"));
}

#[test]
fn test_case_integration_tests_quality_base_replacement() {
    run_test(std::path::Path::new("test_cases/integration_tests/quality_base_replacement"));
}

#[test]
fn test_case_integration_tests_quantify_regions_multi() {
    run_test(std::path::Path::new("test_cases/integration_tests/quantify_regions_multi"));
}

#[test]
fn test_case_integration_tests_quantify_regions_simple() {
    run_test(std::path::Path::new("test_cases/integration_tests/quantify_regions_simple"));
}

#[test]
fn test_case_integration_tests_rename_regex() {
    run_test(std::path::Path::new("test_cases/integration_tests/rename_regex"));
}

#[test]
fn test_case_integration_tests_rename_regex_gets_longer() {
    run_test(std::path::Path::new("test_cases/integration_tests/rename_regex_gets_longer"));
}

#[test]
fn test_case_integration_tests_rename_regex_shorter() {
    run_test(std::path::Path::new("test_cases/integration_tests/rename_regex_shorter"));
}

#[test]
fn test_case_integration_tests_reverse_complement() {
    run_test(std::path::Path::new("test_cases/integration_tests/reverse_complement"));
}

#[test]
fn test_case_integration_tests_skip() {
    run_test(std::path::Path::new("test_cases/integration_tests/skip"));
}

#[test]
fn test_case_integration_tests_stdout_output() {
    run_test(std::path::Path::new("test_cases/integration_tests/stdout_output"));
}

#[test]
fn test_case_integration_tests_stdout_output_interleaved() {
    run_test(std::path::Path::new("test_cases/integration_tests/stdout_output_interleaved"));
}

#[test]
fn test_case_integration_tests_subsample() {
    run_test(std::path::Path::new("test_cases/integration_tests/subsample"));
}

#[test]
fn test_case_integration_tests_trim_adapter_mismatch_tail() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_adapter_mismatch_tail"));
}

#[test]
fn test_case_integration_tests_trim_poly_tail_detail() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_poly_tail_detail"));
}

#[test]
fn test_case_integration_tests_trim_poly_tail_detail_g() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_poly_tail_detail_g"));
}

#[test]
fn test_case_integration_tests_trim_poly_tail_long() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_poly_tail_long"));
}

#[test]
fn test_case_integration_tests_trim_poly_tail_n() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_poly_tail_n"));
}

#[test]
fn test_case_integration_tests_trim_qual_end() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_qual_end"));
}

#[test]
fn test_case_integration_tests_trim_qual_start() {
    run_test(std::path::Path::new("test_cases/integration_tests/trim_qual_start"));
}

#[test]
fn test_case_integration_tests_very_long_reads() {
    run_test(std::path::Path::new("test_cases/integration_tests/very_long_reads"));
}

#[test]
fn test_case_integration_tests_zstd_input() {
    run_test(std::path::Path::new("test_cases/integration_tests/zstd_input"));
}

#[test]
fn test_case_integration_tests_zstd_input_gzip_output() {
    run_test(std::path::Path::new("test_cases/integration_tests/zstd_input_gzip_output"));
}

#[test]
fn test_case_integration_tests_zstd_input_read_swap() {
    run_test(std::path::Path::new("test_cases/integration_tests/zstd_input_read_swap"));
}

#[test]
fn test_case_integration_tests_zstd_input_read_swap_no_read2() {
    run_test(std::path::Path::new("test_cases/integration_tests/zstd_input_read_swap_no_read2"));
}

#[test]
fn test_case_integration_tests_zstd_input_zst_output() {
    run_test(std::path::Path::new("test_cases/integration_tests/zstd_input_zst_output"));
}

#[test]
fn test_case_reports_duplication_count_is_stable() {
    run_test(std::path::Path::new("test_cases/reports/duplication_count_is_stable"));
}

#[test]
fn test_case_reports_oligo_counts() {
    run_test(std::path::Path::new("test_cases/reports/oligo_counts"));
}

#[test]
fn test_case_reports_oligo_counts_2() {
    run_test(std::path::Path::new("test_cases/reports/oligo_counts_2"));
}

#[test]
fn test_case_reports_read_length_reporting() {
    run_test(std::path::Path::new("test_cases/reports/read_length_reporting"));
}

#[test]
fn test_case_reports_report() {
    run_test(std::path::Path::new("test_cases/reports/report"));
}

#[test]
fn test_case_reports_report_depduplication_per_fragment() {
    run_test(std::path::Path::new("test_cases/reports/report_depduplication_per_fragment"));
}

#[test]
fn test_case_reports_report_no_output() {
    run_test(std::path::Path::new("test_cases/reports/report_no_output"));
}

#[test]
fn test_case_reports_report_pe() {
    run_test(std::path::Path::new("test_cases/reports/report_pe"));
}

#[test]
fn test_case_validation_invalid_compression_levels_inspect_gzip_level_too_high() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/inspect_gzip_level_too_high"));
}

#[test]
fn test_case_validation_invalid_compression_levels_inspect_zstd_level_zero() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/inspect_zstd_level_zero"));
}

#[test]
fn test_case_validation_invalid_compression_levels_output_gzip_level_too_high() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/output_gzip_level_too_high"));
}

#[test]
fn test_case_validation_invalid_compression_levels_output_zstd_level_too_high() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/output_zstd_level_too_high"));
}

#[test]
fn test_case_validation_invalid_compression_levels_output_zstd_level_zero() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/output_zstd_level_zero"));
}

#[test]
fn test_case_validation_invalid_compression_levels_raw_with_compression_level() {
    run_test(std::path::Path::new("test_cases/validation/invalid_compression_levels/raw_with_compression_level"));
}

#[test]
fn test_case_validation_mismatched_seq_qual_len_1st_read_qual_too_long() {
    run_test(std::path::Path::new("test_cases/validation/mismatched_seq_qual_len_1st_read_qual_too_long"));
}

#[test]
fn test_case_validation_mismatched_seq_qual_len_1st_read_qual_too_short() {
    run_test(std::path::Path::new("test_cases/validation/mismatched_seq_qual_len_1st_read_qual_too_short"));
}

#[test]
fn test_case_validation_mismatched_seq_qual_len_2nd_read_qual_too_long() {
    run_test(std::path::Path::new("test_cases/validation/mismatched_seq_qual_len_2nd_read_qual_too_long"));
}

#[test]
fn test_case_validation_mismatched_seq_qual_len_2nd_read_qual_too_short() {
    run_test(std::path::Path::new("test_cases/validation/mismatched_seq_qual_len_2nd_read_qual_too_short"));
}

#[test]
fn test_case_validation_validate_phred() {
    run_test(std::path::Path::new("test_cases/validation/validate_phred"));
}

#[test]
fn test_case_validation_validate_phred_fail() {
    run_test(std::path::Path::new("test_cases/validation/validate_phred_fail"));
}

#[test]
fn test_case_validation_validate_seq() {
    run_test(std::path::Path::new("test_cases/validation/validate_seq"));
}

#[test]
fn test_case_validation_validate_seq_fail() {
    run_test(std::path::Path::new("test_cases/validation/validate_seq_fail"));
}
