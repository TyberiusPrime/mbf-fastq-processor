// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;

#[test]
fn test_cases_x_demultiplex_x_bool_1() {
    println!("Test case is in: test_cases/demultiplex/bool");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/bool"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_bool_with_unmatched_1() {
    println!("Test case is in: test_cases/demultiplex/bool_with_unmatched");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/bool_with_unmatched"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_chunked_interleaved_output_demultiplex_1() {
    println!("Test case is in: test_cases/demultiplex/chunked_interleaved_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/chunked_interleaved_output_demultiplex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_chunked_output_demultiplex_1() {
    println!("Test case is in: test_cases/demultiplex/chunked_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/chunked_output_demultiplex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_duplicates_1() {
    println!("Test case is in: test_cases/demultiplex/duplicates");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/duplicates"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_error_no_upstream_1() {
    println!("Test case is in: test_cases/demultiplex/error_no_upstream");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/error_no_upstream"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_error_unmatched_not_set_1() {
    println!("Test case is in: test_cases/demultiplex/error_unmatched_not_set");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/error_unmatched_not_set"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_head_with_index_and_demultiplex_1() {
    println!("Test case is in: test_cases/demultiplex/head_with_index_and_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/head_with_index_and_demultiplex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_inspect_1() {
    println!("Test case is in: test_cases/demultiplex/inspect");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/inspect"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_interleaved_output_demultiplex_1() {
    println!("Test case is in: test_cases/demultiplex/interleaved_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/interleaved_output_demultiplex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_multiple_barcode_and_bool_1() {
    println!("Test case is in: test_cases/demultiplex/multiple_barcode_and_bool");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/multiple_barcode_and_bool"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_no_matching_barcodes_1() {
    println!("Test case is in: test_cases/demultiplex/no_matching_barcodes");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/no_matching_barcodes"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_quantify_tag_1() {
    println!("Test case is in: test_cases/demultiplex/quantify_tag");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/quantify_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_reservoir_sample_1() {
    println!("Test case is in: test_cases/demultiplex/reservoir_sample");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/reservoir_sample"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_bam_output_1() {
    println!("Test case is in: test_cases/demultiplex/simple_bam_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_bam_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_basics_1() {
    println!("Test case is in: test_cases/demultiplex/simple_basics");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_basics"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order1_1() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order1");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_combined_outputs/order1"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order2_invariant_1() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order2_invariant");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_combined_outputs/order2_invariant"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_hamming_1() {
    println!("Test case is in: test_cases/demultiplex/simple_hamming");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_hamming"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_1() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_hamming_1() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_hamming");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac_hamming"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_two_regions_1() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_two_regions");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac_two_regions"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_multiple_into_one_output_1() {
    println!("Test case is in: test_cases/demultiplex/simple_multiple_into_one_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_multiple_into_one_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_no_unmatched_1() {
    println!("Test case is in: test_cases/demultiplex/simple_no_unmatched");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_no_unmatched"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode_1() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_single_barcode"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode_no_unmatched_output_1() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode_no_unmatched_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_single_barcode_no_unmatched_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_demultiplex_x_two_barcodes_1() {
    println!("Test case is in: test_cases/demultiplex/two_barcodes");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/two_barcodes"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_error_handling_x_bam_x_disk_full_bam_1() {
    println!("Test case is in: test_cases/error_handling/bam/disk_full_bam");
    run_test(
        std::path::Path::new("../test_cases/error_handling/bam/disk_full_bam"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_error_handling_x_misc_x_disk_full_fastq_1() {
    println!("Test case is in: test_cases/error_handling/misc/disk_full_fastq");
    run_test(
        std::path::Path::new("../test_cases/error_handling/misc/disk_full_fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_error_handling_x_misc_x_missing_output_dir_1() {
    println!("Test case is in: test_cases/error_handling/misc/missing_output_dir");
    run_test(
        std::path::Path::new("../test_cases/error_handling/misc/missing_output_dir"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_error_handling_x_replace_tag_with_letter_x_no_letter_1() {
    println!("Test case is in: test_cases/error_handling/replace_tag_with_letter/no_letter");
    run_test(
        std::path::Path::new("../test_cases/error_handling/replace_tag_with_letter/no_letter"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_if_tag_x_cut_start_conditional_1() {
    println!("Test case is in: test_cases/if_tag/cut_start_conditional");
    run_test(
        std::path::Path::new("../test_cases/if_tag/cut_start_conditional"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_if_tag_x_if_tag_inverted_1() {
    println!("Test case is in: test_cases/if_tag/if_tag_inverted");
    run_test(
        std::path::Path::new("../test_cases/if_tag/if_tag_inverted"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_if_tag_x_if_tag_location_to_bool_1() {
    println!("Test case is in: test_cases/if_tag/if_tag_location_to_bool");
    run_test(
        std::path::Path::new("../test_cases/if_tag/if_tag_location_to_bool"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_gz_input_1() {
    println!("Test case is in: test_cases/input/compression/gz_input");
    run_test(
        std::path::Path::new("../test_cases/input/compression/gz_input"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_gzip_blocks_spliting_reads_1() {
    println!("Test case is in: test_cases/input/compression/gzip_blocks_spliting_reads");
    run_test(
        std::path::Path::new("../test_cases/input/compression/gzip_blocks_spliting_reads"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_basic_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/basic");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_error_no_rapid_gzip_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/error_no_rapid_gzip");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/error_no_rapid_gzip"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_no_index_not_created_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/no_index_not_created");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/no_index_not_created"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_single_thread_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/single_thread");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/single_thread"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_with_index_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/with_index");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/with_index"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_with_index_and_head_1() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/with_index_and_head");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/with_index_and_head"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_empty_input_1() {
    println!("Test case is in: test_cases/input/error_handling/empty_input");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/empty_input"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_array_1() {
    println!("Test case is in: test_cases/input/error_handling/input_array");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_array"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_nested_map_1() {
    println!("Test case is in: test_cases/input/error_handling/input_nested_map");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_nested_map"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_nested_map_output_mistake_1() {
    println!("Test case is in: test_cases/input/error_handling/input_nested_map_output_mistake");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_nested_map_output_mistake"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_key_1() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_key");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_key"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_value_1() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_value");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_value"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_value_nested_1() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_value_nested");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_value_nested"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_str_1() {
    println!("Test case is in: test_cases/input/error_handling/input_str");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_str"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_non_string_values_1() {
    println!("Test case is in: test_cases/input/error_handling/non_string_values");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/non_string_values"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_basic_1() {
    println!("Test case is in: test_cases/input/interleaved/basic");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_mixing_formats_1() {
    println!("Test case is in: test_cases/input/interleaved/error_mixing_formats");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_mixing_formats"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_mixing_stdin_and_normal_files_1() {
    println!("Test case is in: test_cases/input/interleaved/error_mixing_stdin_and_normal_files");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_mixing_stdin_and_normal_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_only_one_segment_1() {
    println!("Test case is in: test_cases/input/interleaved/error_only_one_segment");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_only_one_segment"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_gzip_1() {
    println!("Test case is in: test_cases/input/interleaved/gzip");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/gzip"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_must_have_even_block_size_1() {
    println!("Test case is in: test_cases/input/interleaved/must_have_even_block_size");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/must_have_even_block_size"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_test_premature_termination_1() {
    println!("Test case is in: test_cases/input/interleaved/test_premature_termination");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/test_premature_termination"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_two_files_1() {
    println!("Test case is in: test_cases/input/interleaved/two_files");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/two_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_interleaved_1() {
    println!("Test case is in: test_cases/input/stdin/stdin_interleaved");
    run_test(
        std::path::Path::new("../test_cases/input/stdin/stdin_interleaved"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_regular_1() {
    println!("Test case is in: test_cases/input/stdin/stdin_regular");
    run_test(
        std::path::Path::new("../test_cases/input/stdin/stdin_regular"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_validation_x_empty_bam_in_middle_1() {
    println!("Test case is in: test_cases/input_validation/empty_bam_in_middle");
    run_test(
        std::path::Path::new("../test_cases/input_validation/empty_bam_in_middle"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breaking_after_sequence_in_partial_1() {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breaking_after_sequence_in_partial"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breaking_after_sequence_in_partial",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_1()
 {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows_1()
 {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_allow_overwrites_1() {
    println!("Test case is in: test_cases/integration/basic/allow_overwrites");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/allow_overwrites"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_noop_1() {
    println!("Test case is in: test_cases/integration/basic/noop");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/noop"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_noop_minimal_1() {
    println!("Test case is in: test_cases/integration/basic/noop_minimal");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/noop_minimal"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_416_1() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_416");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_416"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_491_1() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_491");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_491"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_606_1() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_606");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_606"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_old_cli_format_1() {
    println!("Test case is in: test_cases/integration/compatibility/old_cli_format");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/old_cli_format"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_location_loss_on_conditional_trim_1() {
    println!("Test case is in: test_cases/integration/complex/location_loss_on_conditional_trim");
    run_test(
        std::path::Path::new("../test_cases/integration/complex/location_loss_on_conditional_trim"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_order_maintained_in_single_core_transforms_1() {
    println!(
        "Test case is in: test_cases/integration/complex/order_maintained_in_single_core_transforms"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/integration/complex/order_maintained_in_single_core_transforms",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_ten_segments_creative_transforms_1() {
    println!("Test case is in: test_cases/integration/complex/ten_segments_creative_transforms");
    run_test(
        std::path::Path::new("../test_cases/integration/complex/ten_segments_creative_transforms"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_max_one_block_in_flight_1() {
    println!("Test case is in: test_cases/integration/edge_cases/max_one_block_in_flight");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/max_one_block_in_flight"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_mega_long_reads_1() {
    println!("Test case is in: test_cases/integration/edge_cases/mega_long_reads");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/mega_long_reads"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_very_long_reads_1() {
    println!("Test case is in: test_cases/integration/edge_cases/very_long_reads");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/very_long_reads"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_head_x_tag_histogram_before_head_1() {
    println!("Test case is in: test_cases/integration/head/tag_histogram_before_head");
    run_test(
        std::path::Path::new("../test_cases/integration/head/tag_histogram_before_head"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_both_1() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/both");
    run_test(
        std::path::Path::new("../test_cases/integration/io/cut_end_named_pipes/both"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_input_pipe_1() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/input_pipe");
    run_test(
        std::path::Path::new("../test_cases/integration/io/cut_end_named_pipes/input_pipe"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_io_x_mixed_input_files_1() {
    println!("Test case is in: test_cases/integration/io/mixed_input_files");
    run_test(
        std::path::Path::new("../test_cases/integration/io/mixed_input_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_x_misc_x_head_with_index_1() {
    println!("Test case is in: test_cases/integration/misc/head_with_index");
    run_test(
        std::path::Path::new("../test_cases/integration/misc/head_with_index"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_multi_1() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_multi");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/calc/quantify_regions_multi"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_simple_1() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_simple");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/calc/quantify_regions_simple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_integration_tests_x_integration_tests_x_input_is_symlink_1() {
    println!("Test case is in: test_cases/integration_tests/integration_tests/input_is_symlink");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/integration_tests/input_is_symlink"),
        "input_prelink.toml",
    );
}

#[test]
fn test_cases_x_integration_tests_x_quality_base_replacement_1() {
    println!("Test case is in: test_cases/integration_tests/quality_base_replacement");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/quality_base_replacement"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_bam_x_basic_1() {
    println!("Test case is in: test_cases/output/bam/basic");
    run_test(
        std::path::Path::new("../test_cases/output/bam/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_bam_x_interleaved_1() {
    println!("Test case is in: test_cases/output/bam/interleaved");
    run_test(
        std::path::Path::new("../test_cases/output/bam/interleaved"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_bam_1() {
    println!("Test case is in: test_cases/output/chunked/bam");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/bam"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_exceeding_100_chunks_1() {
    println!("Test case is in: test_cases/output/chunked/fastq_exceeding_100_chunks");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/fastq_exceeding_100_chunks"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_gzip_1() {
    println!("Test case is in: test_cases/output/chunked/fastq_gzip");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/fastq_gzip"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_custom_ix_separator_1() {
    println!("Test case is in: test_cases/output/custom_ix_separator");
    run_test(
        std::path::Path::new("../test_cases/output/custom_ix_separator"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_custom_ix_separator_table_no_infix_1() {
    println!("Test case is in: test_cases/output/custom_ix_separator_table_no_infix");
    run_test(
        std::path::Path::new("../test_cases/output/custom_ix_separator_table_no_infix"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_backslash_in_x_sep_1() {
    println!("Test case is in: test_cases/output/error_handling/backslash_in_x_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/backslash_in_x_sep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_colon_in_ix_sep_1() {
    println!("Test case is in: test_cases/output/error_handling/colon_in_ix_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/colon_in_ix_sep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_slash_in_ix_sep_1() {
    println!("Test case is in: test_cases/output/error_handling/slash_in_ix_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/slash_in_ix_sep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_hash_output_both_1() {
    println!("Test case is in: test_cases/output/hash_output_both");
    run_test(
        std::path::Path::new("../test_cases/output/hash_output_both"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_hash_output_compressed_1() {
    println!("Test case is in: test_cases/output/hash_output_compressed");
    run_test(
        std::path::Path::new("../test_cases/output/hash_output_compressed"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_interleaved_output_1() {
    println!("Test case is in: test_cases/output/interleaved_output");
    run_test(
        std::path::Path::new("../test_cases/output/interleaved_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_one_report_is_enough_x_html_1() {
    println!("Test case is in: test_cases/output/one_report_is_enough/html");
    run_test(
        std::path::Path::new("../test_cases/output/one_report_is_enough/html"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_one_report_is_enough_x_json_1() {
    println!("Test case is in: test_cases/output/one_report_is_enough/json");
    run_test(
        std::path::Path::new("../test_cases/output/one_report_is_enough/json"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_x_chunked_x_fastq_1() {
    println!("Test case is in: test_cases/output/output/chunked/fastq");
    run_test(
        std::path::Path::new("../test_cases/output/output/chunked/fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_x_fastq_1() {
    println!("Test case is in: test_cases/output/output/fastq");
    run_test(
        std::path::Path::new("../test_cases/output/output/fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_compression_gzip_level_1() {
    println!("Test case is in: test_cases/output/output_compression_gzip_level");
    run_test(
        std::path::Path::new("../test_cases/output/output_compression_gzip_level"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_different_suffix_1() {
    println!("Test case is in: test_cases/output/output_different_suffix");
    run_test(
        std::path::Path::new("../test_cases/output/output_different_suffix"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2_1() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2");
    run_test(
        std::path::Path::new("../test_cases/output/output_neither_r1_nor_r2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2_but_index2_1() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2_but_index2");
    run_test(
        std::path::Path::new("../test_cases/output/output_neither_r1_nor_r2_but_index2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_r1_only_1() {
    println!("Test case is in: test_cases/output/output_r1_only");
    run_test(
        std::path::Path::new("../test_cases/output/output_r1_only"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_output_r2_only_1() {
    println!("Test case is in: test_cases/output/output_r2_only");
    run_test(
        std::path::Path::new("../test_cases/output/output_r2_only"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_progress_x_basic_1() {
    println!("Test case is in: test_cases/output/progress/basic");
    run_test(
        std::path::Path::new("../test_cases/output/progress/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_progress_x_followed_by_head_1() {
    println!("Test case is in: test_cases/output/progress/followed_by_head");
    run_test(
        std::path::Path::new("../test_cases/output/progress/followed_by_head"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_output_x_stdout_output_interleaved_1() {
    println!("Test case is in: test_cases/output/stdout_output_interleaved");
    run_test(
        std::path::Path::new("../test_cases/output/stdout_output_interleaved"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_auto_x_detect_x_on_x_stderr_1() {
    println!("Test case is in: test_cases/single_step/auto-detect-on-stderr");
    run_test(
        std::path::Path::new("../test_cases/single_step/auto-detect-on-stderr"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_basic_1() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_basic_all_segments_1() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/basic_all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/basic_all_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_input_error_invalid_quality_1() {
    println!(
        "Test case is in: test_cases/single_step/calc/expected_error/input_error_invalid_quality"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/calc/expected_error/input_error_invalid_quality",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_max_1() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/max");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/max"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_max_all_segments_1() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/max_all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/max_all_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_basic_1() {
    println!("Test case is in: test_cases/single_step/calc/kmer/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_basic_higher_min_count_1() {
    println!("Test case is in: test_cases/single_step/calc/kmer/basic_higher_min_count");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/basic_higher_min_count"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_files_as_sequence_1() {
    println!("Test case is in: test_cases/single_step/calc/kmer/files_as_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/files_as_sequence"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_phix_1() {
    println!("Test case is in: test_cases/single_step/calc/kmer/phix");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/phix"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_1() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_gzip_output_1() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_gzip_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_gzip_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_read_swap_1() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_read_swap");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_read_swap"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_zst_output_1() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_zst_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_zst_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_and_string_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/location_and_string_concat"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_does_not_provide_location_1()
{
    println!(
        "Test case is in: test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_outputs_location_1() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/location_and_string_concat_outputs_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/location_and_string_concat_outputs_location",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_concat_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/location_concat"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_multiple_hits_per_tag_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/multiple_hits_per_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/multiple_hits_per_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_multiple_hits_per_tag_anchor_right_1() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/multiple_hits_per_tag_anchor_right"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/multiple_hits_per_tag_anchor_right",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_string_string_concat_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/string_string_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/string_string_concat"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_three_tags_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/three_tags");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/three_tags"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_merge_present_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_merge_present");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/with_missing_tag_merge_present",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_set_missing_1() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_set_missing");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/with_missing_tag_set_missing"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_convert_to_rate_x_all_segments_1() {
    println!("Test case is in: test_cases/single_step/convert/convert_to_rate/all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/convert_to_rate/all_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_basic_1() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/regions_to_length/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_error_in_equal_out_1() {
    println!(
        "Test case is in: test_cases/single_step/convert/regions_to_length/error_in_equal_out"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert/regions_to_length/error_in_equal_out",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_multiple_regions_1() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/multiple_regions");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert/regions_to_length/multiple_regions",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_polyx_1() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/polyx");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/regions_to_length/polyx"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_basic_1() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_denominator_tag_1() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/denominator_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/denominator_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_log_variants_1() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/log_variants");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/log_variants"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred_1() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/convert_phred"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred_broken_input_1() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred_broken_input");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert_quality/convert_phred_broken_input",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred_multi_1() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred_multi");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/convert_phred_multi"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_error_convert_to_same_1() {
    println!("Test case is in: test_cases/single_step/convert_quality/error_convert_to_same");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/error_convert_to_same"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_basic_1() {
    println!("Test case is in: test_cases/single_step/dedup/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_exact_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_exact");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_exact"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_keep_duplicates_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_keep_duplicates");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_keep_duplicates"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read2_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read2");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_demultiplex_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo_demultiplex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_incl_index_1() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo_incl_index");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo_incl_index"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_error_no_seed_1() {
    println!("Test case is in: test_cases/single_step/dedup/error_no_seed");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/error_no_seed"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_location_tag_1() {
    println!("Test case is in: test_cases/single_step/dedup/exact_location_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_location_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_name_1() {
    println!("Test case is in: test_cases/single_step/dedup/exact_name");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_name"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_tag_1() {
    println!("Test case is in: test_cases/single_step/dedup/exact_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_tag_missing_values_1() {
    println!("Test case is in: test_cases/single_step/dedup/exact_tag_missing_values");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_tag_missing_values"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_cat_1() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_sanger_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_solexa_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_longreads_x_cat_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/longreads/cat"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/longreads/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_illumina_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_solexa_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_cat_1() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_illumina_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_solexa_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_cat_1() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_illumina_1()
{
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_solexa_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_cat_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_illumina_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_sanger_1() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_cat_1() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/cat");
    run_test(
        std::path::Path::new("../test_cases/single_step/edge_cases/challenging_formats/solexa/cat"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_wrapping_x_cat_1() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/wrapping/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/wrapping/cat",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_sequence_1() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/lowercase_sequence"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_tag_1() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/lowercase_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_concatenate_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_concatenate");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/no_overlap_concatenate"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_keep_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/no_overlap_keep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_perfect_match_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_perfect_match");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/overlap_perfect_match"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_with_tag_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_with_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/overlap_with_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/vs_fastp"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp_systematic_1() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp_systematic");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/vs_fastp_systematic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp_systematic_but_limit_is_percentage_1()
{
    println!(
        "Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp_systematic_but_limit_is_percentage"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edits/merge_reads/vs_fastp_systematic_but_limit_is_percentage",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement_1() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/reverse_complement"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement_conditional_1() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement_conditional");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/reverse_complement_conditional"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_auto_detect_two_segments_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_auto_detect_two_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_auto_detect_two_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_extended_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_extended");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_extended"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_majority_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_majority");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_majority"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_minority_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_minority");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_minority"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_four_segments_1() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_four_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_four_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_sequence_1() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/uppercase_sequence"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_tag_1() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/uppercase_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_spotcheckreadpairing_x_not_paired_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/SpotCheckReadPairing/not_paired"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/SpotCheckReadPairing/not_paired",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_both_false_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_mapped_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_unmapped_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_output_uncompressed_hash_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_segment_barcode_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/segment_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/segment_barcode",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_segment_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/segment_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/segment_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_tag_barcode_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/tag_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/tag_barcode",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_compression_detection_wrong_extension_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/compression_detection_wrong_extension"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/compression_detection_wrong_extension",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_gzip_level_too_high_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_zstd_level_zero_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_raw_with_compression_level_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_barcode_lengths_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_files_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/different_files",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_non_iupac_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_same_files_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/same_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/same_files",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_agtc_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_empty_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_all_on_non_all_segments_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/all_on_non_all_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/all_on_non_all_segments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_all_on_non_segment_or_name_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/all_on_non_segment_or_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/all_on_non_segment_or_name",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_barcodes_as_list_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/barcodes_as_list"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/barcodes_as_list",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_show_step_template_on_error_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/show_step_template_on_error"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/show_step_template_on_error",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_tag_not_defined_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/tag_not_defined"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/tag_not_defined",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_eserde_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_eserde"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/two_mistakes_eserde",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_post_deserialization_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_eval_expr_x_len_from_not_a_len_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_absolute_with_ignore_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_empty_count_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_empty_count"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_empty_count",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_invalid_letters_1()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_gc_panic_on_store_in_seq_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_min_length_too_high_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_too_many_mismatches_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_from_name_followed_by_uppercase_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_label_starts_with_name_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_region_from_name_but_storing_location_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_region_from_name_but_storing_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_region_from_name_but_storing_location",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_tag_from_i1_i2_no_i1_i2_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_by_tag_numeric_rejection_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_no_such_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_no_such_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/filter_no_such_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_numeric_filter_wrong_tag_type_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_insert_char_in_value_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_label_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_value_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_same_infix_twice_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_store_tags_in_table_no_tags_defined_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_tag_name_x_tag_name_not_len_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_trim_tag_multi_locations_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/trim_tag_multi_locations"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/trim_tag_multi_locations",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_unused_extract_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/unused_extract_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/unused_extract_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_validate_regex_fail_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/validate_regex_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/validate_regex_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_bynumerictagminormax_1() {
    println!("Test case is in: test_cases/single_step/error_handling/filter/ByNumericTagMinOrMax");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/ByNumericTagMinOrMax",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_other_file_by_seq_x_negative_fpr_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/filter/other_file_by_seq/negative_fpr"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/other_file_by_seq/negative_fpr",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_other_file_by_seq_x_no_seed_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/filter/other_file_by_seq/no_seed"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/other_file_by_seq/no_seed",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_fake_fasta_missing_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/fake_fasta_missing"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/fake_fasta_missing",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_filter_missing_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/filter_missing_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/filter_missing_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index1_file_does_not_exist_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index1_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/index1_file_does_not_exist",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index2_file_does_not_exist_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index2_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/index2_file_does_not_exist",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_input_file_is_output_file_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/input_file_is_output_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/input_file_is_output_file",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_missing_input_file_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/missing_input_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/missing_input_file",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_output_x_interleave_x_missing_target_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/output/interleave/missing_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/output/interleave/missing_target",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read1_more_than_read2_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read2_more_than_read1_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_input_file_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_input_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/permission_denied_input_file",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_read1_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_read1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/permission_denied_read1",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_empty_list_1() {
    println!("Test case is in: test_cases/single_step/error_handling/input_files/read1_empty_list");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_empty_list",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index1_len_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index2_len_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_read2_len_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_not_a_string_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_not_a_string"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_not_a_string",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_file_does_not_exist_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read2_file_does_not_exist",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_not_a_string_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_not_a_string"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read2_not_a_string",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index1_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_index1",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index2_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_index2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_one_key_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_one_key"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_one_key",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_a_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_b_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_io_x_stdin_multiple_files_1() {
    println!("Test case is in: test_cases/single_step/error_handling/io/stdin_multiple_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/io/stdin_multiple_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_no_file_1() {
    println!("Test case is in: test_cases/single_step/error_handling/kmer/no_file");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/kmer/no_file"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_wrong_type_for_files_1() {
    println!("Test case is in: test_cases/single_step/error_handling/kmer/wrong_type_for_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/kmer/wrong_type_for_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_wrong_type_for_files_nested_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/kmer/wrong_type_for_files_nested"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/kmer/wrong_type_for_files_nested",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_newline",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline2_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_newline2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_panics_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_panics"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_panics",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_diff_ids_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_diff_ids"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_diff_ids",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_double_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_seq_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_double_seq",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_long_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_long_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_long_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_no_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_no_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_no_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_del_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_del"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_del",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_escape_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_escape"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_escape",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_null_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_null"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_null",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_space_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_space"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_space",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_tab_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_tab"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_tab",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_unit_sep_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_vtab_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_short_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_short_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_short_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_spaces_1() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_spaces");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_spaces",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_tabs_1() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_tabs");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/malformed_fastq/error_tabs"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_plus_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_seq_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_plus_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_seq_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_seq_tiny_blocksize_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq_tiny_blocksize"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq_tiny_blocksize",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_title_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_1() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot_too_long_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_long_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_short_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_long_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_short_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_and_truncated_qual_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_at_all_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_at_all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_at_all",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_at_end_ok_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_quality_starts_with_at_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_truncated_after_at_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/truncated_after_at"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/truncated_after_at",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_windows_newlines_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/windows_newlines"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/windows_newlines",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_list_of_files_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_list_of_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/empty_list_of_files"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_name_input_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_name_input");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/empty_name_input"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_mixed_input_formats_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/mixed_input_formats");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/mixed_input_formats"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_postfix_len_mismatch_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/postfix_len_mismatch");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/postfix_len_mismatch"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_prefix_len_mismatch_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/prefix_len_mismatch");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/prefix_len_mismatch"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_read_with_comment_in_line_3_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/read_with_comment_in_line_3"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/read_with_comment_in_line_3",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_report_names_distinct_1() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/report_names_distinct");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/report_names_distinct"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_number_to_large_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_number_to_large"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/u8_from_char_number_to_large",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_too_many_chars_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_barcode_outputs_not_named_no_barcode_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_empty_output_1() {
    println!("Test case is in: test_cases/single_step/error_handling/output_config/empty_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/output_config/empty_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_invalid_segment_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_no_segment_specified_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_tag_i1_i2_but_not_output_1()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_input_interleaved_multiple_segment_files_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_gzip_level_too_high_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_too_high_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_zero_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_segment_names_x_all_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_segment_names/all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_segment_names/all",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_segment_names_x_internal_1()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_segment_names/internal"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_segment_names/internal",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_empty_output_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_format_raw_1()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_segments_1() {
    println!("Test case is in: test_cases/single_step/error_handling/output_config/no_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/output_config/no_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_fifo_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_fifo"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/chunked_fifo",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_stdout_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_stdout"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/chunked_stdout",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_duplicated_target_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_just_one_target_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/just_one_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/interleave/just_one_target",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_report_without_output_flags_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/report_without_output_flags"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/report_without_output_flags",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_defaults_multiple_segments_fails_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_duplicated_interleave_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_duplicated_interleave"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_duplicated_interleave",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_duplicated_after_trim_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_duplicated_after_trim"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_name_duplicated_after_trim",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_empty_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_empty"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_name_empty",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_invalid_path_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_invalid_path"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_name_invalid_path",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_invalid_path2_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_invalid_path2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_name_invalid_path2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_whitespace_only_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_whitespace_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_name_whitespace_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdin_multiple_segments_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdin_multiple_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/stdin_multiple_segments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdout_conflict_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdout_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/stdout_conflict",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_auto_detect_too_few_segments_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_few_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_few_segments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_auto_detect_too_many_segments_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_many_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_many_segments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_same_segment_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_same_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/swap/swap_same_segment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_unwritable_output_dir_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/unwritable_output_dir"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/unwritable_output_dir",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_validate_name_needs_multiple_segments_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_paired_end_x_paired_end_unqueal_read_count_x_read3_more_than_1_2_1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_quality_scores_x_convert_phred_raises_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/quality_scores/convert_phred_raises"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/quality_scores/convert_phred_raises",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_html_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_html"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/reports/report_but_no_report_step_html",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_json_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_json"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/reports/report_but_no_report_step_json",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_showing_docs_x_from_action_1() {
    println!("Test case is in: test_cases/single_step/error_handling/showing_docs/from_action");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/showing_docs/from_action"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_showing_docs_x_from_braces_1() {
    println!("Test case is in: test_cases/single_step/error_handling/showing_docs/from_braces");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/showing_docs/from_braces"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_x_swap_partial_specification_a_only_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap/swap_partial_specification_a_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap/swap_partial_specification_a_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_x_swap_partial_specification_b_only_1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap/swap_partial_specification_b_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap/swap_partial_specification_b_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_basic_1() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_bool_1() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_bool");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_bool"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_complex_1() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_complex");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_complex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_location_1() {
    println!("Test case is in: test_cases/single_step/eval/location");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/location"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_location_len_1() {
    println!("Test case is in: test_cases/single_step/eval/location_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/location_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_segment_len_1() {
    println!("Test case is in: test_cases/single_step/eval/segment_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/segment_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_str_1() {
    println!("Test case is in: test_cases/single_step/eval/str");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/str"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_str_len_1() {
    println!("Test case is in: test_cases/single_step/eval/str_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/str_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_threshold_1() {
    println!("Test case is in: test_cases/single_step/eval/threshold");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/threshold"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_end_inside_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_start_inside_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_false_1()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_true_1()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_false_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_true_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_after_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_before_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_inside_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_prefix_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/prefix"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/prefix",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_rev_complement_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_swap_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/swap"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/swap",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_swap_conditional_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/swap_conditional"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/swap_conditional",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_trim_quality_start_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_hamming_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/hamming");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/hamming"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_leftmost_verification_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_anchor/leftmost_verification"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_anchor/leftmost_verification",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_right_most_verification_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_anchor/right_most_verification"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_anchor/right_most_verification",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_simple_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/simple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/too_far"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far_left_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far_left");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/too_far_left"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_base_content_simple_test_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_base_content_simple_test");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_base_content_simple_test",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_x_keep_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_filter/keep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_x_remove_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_filter/remove"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_after_trim_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_gc/after_trim"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_segment_all_full_data_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/segment_all_full_data");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_gc/segment_all_full_data",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_segment_all_read1_only_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_gc/segment_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_gc/segment_all_read1_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_simple_test_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_gc/simple_test"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_growing"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing_from_nothing_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing_from_nothing");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_growing_from_nothing"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight_x_basic_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_highlight/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight_x_regex_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight/regex");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_highlight/regex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/multiple");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/multiple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_multiple_patterns_different_positions_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/multiple_patterns_different_positions"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/multiple_patterns_different_positions",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_nested_patterns_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/nested_patterns"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/nested_patterns",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_overlapping_patterns_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/overlapping_patterns"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/overlapping_patterns",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_suffix_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/suffix");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/suffix"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/with_indel"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_anchor_left_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_anchor_left"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_anchor_left",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_anchor_right_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_anchor_right"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_anchor_right",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_empty_reads_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_empty_reads"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_empty_reads",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_empty_search_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_empty_search"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_empty_search",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_label_must_not_be_empty_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_label_must_not_be_empty");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_label_must_not_be_empty",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_after_trim_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length/after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_length/after_trim"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_segment_all_full_data_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length/segment_all_full_data"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_length/segment_all_full_data",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_segment_all_read1_only_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length/segment_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_length/segment_all_read1_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_simple_test_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length/simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_length/simple_test"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_longest_poly_x/any_base",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base_mismatch_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_basic_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_longest_poly_x/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_after_trim_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n/count_after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_n/count_after_trim"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_simple_test_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n/count_simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_n/count_simple_test"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_target_all_full_data_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n/count_target_all_full_data"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_n/count_target_all_full_data",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_target_all_read1_only_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n/count_target_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_n/count_target_all_read1_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_multi_segment_1()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_no_replacement_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_no_replacement_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_regex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex_underscores");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_regex_underscores"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores_x_ok_works_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex_underscores/ok_works"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex_underscores/ok_works",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_and_replace_multiple_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/and_replace_multiple"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/and_replace_multiple",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_beyond_read_len_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/beyond_read_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/beyond_read_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_beyond_read_len_and_trim_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/beyond_read_len_and_trim"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/beyond_read_len_and_trim",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_from_string_tag_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/from_string_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/from_string_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_read_too_short_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/read_too_short");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/read_too_short"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_trim_at_tag_conflict_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/trim_at_tag_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/trim_at_tag_conflict",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_shrinking_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_shrinking");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_shrinking"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_duplicate_name_panics_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_tag_duplicate_name_panics"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_tag_duplicate_name_panics",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_i1_i2_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_i1_i2");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag_i1_i2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_r2_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_r2");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag_r2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_reserved_name_panics_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_reserved_name_panics");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_tag_reserved_name_panics",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_end_false_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/end_false");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/end_false"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_end_true_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/end_true");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/end_true"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_start_false_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/start_false");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/start_false"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_start_true_1() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/start_true");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/start_true"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_overlapping_regions_trim_conflict_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/overlapping_regions_trim_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/overlapping_regions_trim_conflict",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_remove_nonexistant_tag_1() {
    println!("Test case is in: test_cases/single_step/extraction/remove_nonexistant_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/remove_nonexistant_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_basic_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_basic_many_blocks_expose_order_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/basic_many_blocks_expose_order"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/basic_many_blocks_expose_order",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_demultiplex_output_unmatched_x_false_1()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_error_bam_output_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/error_bam_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/error_bam_output",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_error_no_output_def_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/error_no_output_def"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/error_no_output_def",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_fasta_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/fasta"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_gzipped_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/gzipped");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/gzipped"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_head_after_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/head_after");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/head_after"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_no_location_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/no_location");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/no_location"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_with_comments_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/with_comments");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/with_comments",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_with_location_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/with_location");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/with_location",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_head_after_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/head_after");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/head_after"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/in_tsv"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_demultiplex_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_demultiplex"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tags_in_table/in_tsv_demultiplex",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_gz_1() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_gz");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/in_tsv_gz"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_validate_compression_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_validate_compression"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tags_in_table/in_tsv_validate_compression",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_1() {
    println!("Test case is in: test_cases/single_step/extraction/umi_extract");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/umi_extract"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_store_in_all_read_names_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_store_in_all_read_names"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/umi_extract_store_in_all_read_names",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_with_existing_comment_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_with_existing_comment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/umi_extract_with_existing_comment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_forget_all_tags_1() {
    println!("Test case is in: test_cases/single_step/extraction/use_forget_all_tags");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/use_forget_all_tags"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_removed_tag_1() {
    println!("Test case is in: test_cases/single_step/extraction/use_removed_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/use_removed_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_with_demultiplex_x_store_tag_in_fastq_x_demultiplex_1() {
    println!(
        "Test case is in: test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_but_neither_mapped_nor_unmapped_1() {
    println!(
        "Test case is in: test_cases/single_step/fileformats/bam_but_neither_mapped_nor_unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_but_neither_mapped_nor_unmapped",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_to_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/bam_to_fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_both_1() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/both");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/bam_with_index_to_fastq/both"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_mapped_1() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/mapped");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_with_index_to_fastq/mapped",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_unmapped_1() {
    println!(
        "Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_with_index_to_fastq/unmapped",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_gz_to_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_gz_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_gz_to_fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_to_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_to_fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_with_desc_to_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_with_desc_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_with_desc_to_fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_with_empty_desc_to_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_with_empty_desc_to_fastq");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/fasta_with_empty_desc_to_fastq",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_bam_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_bam");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fastq_to_bam"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_fasta_1() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fastq_to_fasta"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta_1() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fasta"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta_compressed_1() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta_compressed");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fasta_compressed"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fastq_1() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fastq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_all_1() {
    println!("Test case is in: test_cases/single_step/filter/empty/all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/all"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_basic_1() {
    println!("Test case is in: test_cases/single_step/filter/empty/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_segments_1() {
    println!("Test case is in: test_cases/single_step/filter/empty/segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_low_complexity_x_all_segments_1() {
    println!("Test case is in: test_cases/single_step/filter/low_complexity/all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/low_complexity/all_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_low_complexity_x_basic_1() {
    println!("Test case is in: test_cases/single_step/filter/low_complexity/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/low_complexity/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_max_len_1() {
    println!("Test case is in: test_cases/single_step/filter/max_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/max_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_min_len_1() {
    println!("Test case is in: test_cases/single_step/filter/min_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/min_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_distinct_separators_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/distinct_separators"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/distinct_separators",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_distinct_separators_conflict_with_store_in_comment_1()
 {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/distinct_separators_conflict_with_store_in_comment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/distinct_separators_conflict_with_store_in_comment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_error_no_reads_in_other_file_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/error_no_reads_in_other_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/error_no_reads_in_other_file",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_keep_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/keep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/remove"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_approximate_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_approximate"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_approximate",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_approximate_no_bai_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_approximate_no_bai"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_approximate_no_bai",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_both_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_both");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/remove_bam_both"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_mapped_only_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_mapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_mapped_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_unmapped_only_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_unmapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_unmapped_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_error_no_reads_in_other_file_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/error_no_reads_in_other_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/error_no_reads_in_other_file",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_keep_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/keep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/remove"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_both_1() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_both");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/remove_bam_both"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_mapped_not_set_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_not_set"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_not_set",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_mapped_only_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_neither_errors_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_neither_errors"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_neither_errors",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_unmapped_not_set_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_not_set"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_not_set",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_unmapped_only_1() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_only",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above_1() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/above"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above_or_equal_1() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above_or_equal");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/above_or_equal"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below_1() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/below"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below_or_equal_1() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below_or_equal");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/below_or_equal"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_all_1() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/all"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_basic_1() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_segments_vs_all_1() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/segments_vs_all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/segments_vs_all"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_1() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_empty_1() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_empty");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction_empty"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_keep_1() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction_keep"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_require_larger_0_1() {
    println!("Test case is in: test_cases/single_step/hamming/require_larger_0");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/require_larger_0"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_string_tag_correction_1() {
    println!("Test case is in: test_cases/single_step/hamming/string_tag_correction");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/string_tag_correction"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_quantify_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_quantify");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_quantify"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_report");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_base_stats_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_base_stats");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_base_stats"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_duplicate_count_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_duplicate_count");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_duplicate_count"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_duplicate_fragment_count_1() {
    println!(
        "Test case is in: test_cases/single_step/head/head_after_report_duplicate_fragment_count"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/head_after_report_duplicate_fragment_count",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_length_distribution_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_length_distribution");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/head_after_report_length_distribution",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_oligos_1() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_oligos");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_oligos"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_quantify_1() {
    println!("Test case is in: test_cases/single_step/head/head_before_quantify");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_before_quantify"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_report_1() {
    println!("Test case is in: test_cases/single_step/head/head_before_report");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_before_report"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading_1() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_stops_reading"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading_multiple_1() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading_multiple");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_stops_reading_multiple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_bottom_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_bottom");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_bottom"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_middle"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle_bottom_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle_bottom");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/multi_stage_head_report_middle_bottom",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_top_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_top");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_top"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_spot_check_pairing_1() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_spot_check_pairing");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_spot_check_pairing"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved_1() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/all_interleaved"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved_reversed_1() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved_reversed");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/all_interleaved_reversed"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_compression_zstd_level_1() {
    println!("Test case is in: test_cases/single_step/inspect/compression_zstd_level");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/compression_zstd_level"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index1_1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index1");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_index1"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index2_1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index2");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_index2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read1_1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read1");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_read1"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read2_1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read2");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_read2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_read1_compressed_1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect_read1_compressed");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect_read1_compressed"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_output_x_cut_end_named_pipes_x_output_pipe_1() {
    println!("Test case is in: test_cases/single_step/output/cut_end_named_pipes/output_pipe");
    run_test(
        std::path::Path::new("../test_cases/single_step/output/cut_end_named_pipes/output_pipe"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_output_x_output_neither_r1_nor_r2_but_index_1() {
    println!("Test case is in: test_cases/single_step/output/output_neither_r1_nor_r2_but_index");
    run_test(
        std::path::Path::new("../test_cases/single_step/output/output_neither_r1_nor_r2_but_index"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_performance_x_duplicate_input_allocation_1() {
    println!("Test case is in: test_cases/single_step/performance/duplicate_input_allocation");
    run_test(
        std::path::Path::new("../test_cases/single_step/performance/duplicate_input_allocation"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_performance_x_duplicate_input_allocation_2() {
    println!("Test case is in: test_cases/single_step/performance/duplicate_input_allocation");
    run_test(
        std::path::Path::new("../test_cases/single_step/performance/duplicate_input_allocation"),
        "input_duplicate.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_read_index_placeholder_1() {
    println!("Test case is in: test_cases/single_step/rename/rename_read_index_placeholder");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_read_index_placeholder"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_1() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_gets_longer_1() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_gets_longer");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex_gets_longer"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_shorter_1() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_shorter");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex_shorter"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_duplication_count_is_stable_1() {
    println!("Test case is in: test_cases/single_step/reports/duplication_count_is_stable");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/duplication_count_is_stable"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts_1() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/oligo_counts"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts_2_1() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts_2");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/oligo_counts_2"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_progress_init_messages_1() {
    println!("Test case is in: test_cases/single_step/reports/progress_init_messages");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/progress_init_messages"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_read_length_reporting_1() {
    println!("Test case is in: test_cases/single_step/reports/read_length_reporting");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/read_length_reporting"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_1() {
    println!("Test case is in: test_cases/single_step/reports/report");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_bam_1() {
    println!("Test case is in: test_cases/single_step/reports/report_bam");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_bam"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_depduplication_per_fragment_1() {
    println!("Test case is in: test_cases/single_step/reports/report_depduplication_per_fragment");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/reports/report_depduplication_per_fragment",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_fasta_1() {
    println!("Test case is in: test_cases/single_step/reports/report_fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_fasta"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_fasta_more_than_one_block_1() {
    println!("Test case is in: test_cases/single_step/reports/report_fasta_more_than_one_block");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_fasta_more_than_one_block"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_no_output_1() {
    println!("Test case is in: test_cases/single_step/reports/report_no_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_no_output"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_ordering_1() {
    println!("Test case is in: test_cases/single_step/reports/report_ordering");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_ordering"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_pe_1() {
    println!("Test case is in: test_cases/single_step/reports/report_pe");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_pe"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_tag_histogram_1() {
    println!("Test case is in: test_cases/single_step/reports/report_tag_histogram");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_tag_histogram"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_1() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_multi_segments_1() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_multi_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample_multi_segments"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_twice_1() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_twice");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample_twice"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_skip_1() {
    println!("Test case is in: test_cases/single_step/sampling/skip");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/skip"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_subsample_1() {
    println!("Test case is in: test_cases/single_step/sampling/subsample");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/subsample"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_no_insert_char_present_1() {
    println!("Test case is in: test_cases/single_step/store_tag/in_comment_no_insert_char_present");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/store_tag/in_comment_no_insert_char_present",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_with_insert_char_present_1() {
    println!(
        "Test case is in: test_cases/single_step/store_tag/in_comment_with_insert_char_present"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/store_tag/in_comment_with_insert_char_present",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_transform_x_max_len_1() {
    println!("Test case is in: test_cases/single_step/transform/max_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/transform/max_len"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_lowqualityend_1() {
    println!("Test case is in: test_cases/single_step/trim/LowQualityEnd");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/LowQualityEnd"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_end_x_basic_1() {
    println!("Test case is in: test_cases/single_step/trim/cut_end/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_end/basic"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_end_x_if_tag_1() {
    println!("Test case is in: test_cases/single_step/trim/cut_end/if_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_end/if_tag"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_start_1() {
    println!("Test case is in: test_cases/single_step/trim/cut_start");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_start"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_detail_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/detail");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/detail"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_detail_g_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/detail_g");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/detail_g"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_long_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/long");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/long"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_n_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/n");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/n"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_n_keep_end_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/n_keep_end");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/n_keep_end"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_x_end_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual/end");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_qual/end"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_x_start_1() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual/start");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_qual/start"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_disabled_1() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/disabled");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/spot_check_read_pairing/disabled",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_fail_1() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/spot_check_read_pairing/fail"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_not_sampled_no_error_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_simple_1() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/spot_check_read_pairing/simple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_all_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/all",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_all_fail_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/all_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/all_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_basic_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/basic"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/basic",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_fail_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_name_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/name",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_name_fail_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/name_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/name_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_fail_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_string_type_1()
{
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_string_type_fail_1()
 {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_simple_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_name/simple"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_custom_separator_1() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_name/validate_name_custom_separator"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_name/validate_name_custom_separator",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_fail_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/validate_name_fail");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_name/validate_name_fail",
        ),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_phred"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred_fail_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred_fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_phred_fail"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_seq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq_fail_1() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq_fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_seq_fail"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_transform_x_prefix_and_postfix_1() {
    println!("Test case is in: test_cases/transform/prefix_and_postfix");
    run_test(
        std::path::Path::new("../test_cases/transform/prefix_and_postfix"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_01_x_basic_x_quality_x_report_1() {
    println!("Test case is in: cookbooks/01-basic-quality-report");
    run_test(
        std::path::Path::new("../cookbooks/01-basic-quality-report"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_02_x_umi_x_extraction_1() {
    println!("Test case is in: cookbooks/02-umi-extraction");
    run_test(
        std::path::Path::new("../cookbooks/02-umi-extraction"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_03_x_lexogen_x_quantseq_1() {
    println!("Test case is in: cookbooks/03-lexogen-quantseq");
    run_test(
        std::path::Path::new("../cookbooks/03-lexogen-quantseq"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_04_x_phix_x_removal_1() {
    println!("Test case is in: cookbooks/04-phiX-removal");
    run_test(
        std::path::Path::new("../cookbooks/04-phiX-removal"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_04_x_phix_x_removal_2() {
    println!("Test case is in: cookbooks/04-phiX-removal");
    run_test(
        std::path::Path::new("../cookbooks/04-phiX-removal"),
        "input_demultiplex.toml",
    );
}

#[test]
fn test_cases_x_05_x_quality_x_filtering_1() {
    println!("Test case is in: cookbooks/05-quality-filtering");
    run_test(
        std::path::Path::new("../cookbooks/05-quality-filtering"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_06_x_adapter_x_trimming_1() {
    println!("Test case is in: cookbooks/06-adapter-trimming");
    run_test(
        std::path::Path::new("../cookbooks/06-adapter-trimming"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_07_x_demultiplexing_1() {
    println!("Test case is in: cookbooks/07-demultiplexing");
    run_test(
        std::path::Path::new("../cookbooks/07-demultiplexing"),
        "input.toml",
    );
}

#[test]
fn test_cases_x_08_x_length_x_filtering_1() {
    println!("Test case is in: cookbooks/08-length-filtering");
    run_test(
        std::path::Path::new("../cookbooks/08-length-filtering"),
        "input.toml",
    );
}
