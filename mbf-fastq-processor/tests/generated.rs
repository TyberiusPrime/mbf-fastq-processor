// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;

#[test]
fn test_cases_x_demultiplex_x_bool() {
    println!("Test case is in: test_cases/demultiplex/bool");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/bool"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_bool_with_unmatched() {
    println!("Test case is in: test_cases/demultiplex/bool_with_unmatched");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/bool_with_unmatched"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_chunked_interleaved_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/chunked_interleaved_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/chunked_interleaved_output_demultiplex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_chunked_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/chunked_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/chunked_output_demultiplex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_duplicates() {
    println!("Test case is in: test_cases/demultiplex/duplicates");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/duplicates"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_error_no_upstream() {
    println!("Test case is in: test_cases/demultiplex/error_no_upstream");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/error_no_upstream"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_error_unmatched_not_set() {
    println!("Test case is in: test_cases/demultiplex/error_unmatched_not_set");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/error_unmatched_not_set"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_head_with_index_and_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/head_with_index_and_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/head_with_index_and_demultiplex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_inspect() {
    println!("Test case is in: test_cases/demultiplex/inspect");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/inspect"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_interleaved_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/interleaved_output_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/interleaved_output_demultiplex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_multiple_barcode_and_bool() {
    println!("Test case is in: test_cases/demultiplex/multiple_barcode_and_bool");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/multiple_barcode_and_bool"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_no_matching_barcodes() {
    println!("Test case is in: test_cases/demultiplex/no_matching_barcodes");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/no_matching_barcodes"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_quantify_tag() {
    println!("Test case is in: test_cases/demultiplex/quantify_tag");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/quantify_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_reservoir_sample() {
    println!("Test case is in: test_cases/demultiplex/reservoir_sample");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/reservoir_sample"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_bam_output() {
    println!("Test case is in: test_cases/demultiplex/simple_bam_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_bam_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_basics() {
    println!("Test case is in: test_cases/demultiplex/simple_basics");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_basics"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order1() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order1");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_combined_outputs/order1"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order2_invariant() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order2_invariant");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_combined_outputs/order2_invariant"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_hamming() {
    println!("Test case is in: test_cases/demultiplex/simple_hamming");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_hamming"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_hamming() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_hamming");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac_hamming"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_two_regions() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_two_regions");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_iupac_two_regions"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_multiple_into_one_output() {
    println!("Test case is in: test_cases/demultiplex/simple_multiple_into_one_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_multiple_into_one_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_no_unmatched() {
    println!("Test case is in: test_cases/demultiplex/simple_no_unmatched");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_no_unmatched"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_single_barcode"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode_no_unmatched_output() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode_no_unmatched_output");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/simple_single_barcode_no_unmatched_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_demultiplex_x_two_barcodes() {
    println!("Test case is in: test_cases/demultiplex/two_barcodes");
    run_test(
        std::path::Path::new("../test_cases/demultiplex/two_barcodes"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_error_handling_x_bam_x_disk_full_bam() {
    println!("Test case is in: test_cases/error_handling/bam/disk_full_bam");
    run_test(
        std::path::Path::new("../test_cases/error_handling/bam/disk_full_bam"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_error_handling_x_misc_x_disk_full_fastq() {
    println!("Test case is in: test_cases/error_handling/misc/disk_full_fastq");
    run_test(
        std::path::Path::new("../test_cases/error_handling/misc/disk_full_fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_error_handling_x_misc_x_missing_output_dir() {
    println!("Test case is in: test_cases/error_handling/misc/missing_output_dir");
    run_test(
        std::path::Path::new("../test_cases/error_handling/misc/missing_output_dir"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_error_handling_x_replace_tag_with_letter_x_no_letter() {
    println!("Test case is in: test_cases/error_handling/replace_tag_with_letter/no_letter");
    run_test(
        std::path::Path::new("../test_cases/error_handling/replace_tag_with_letter/no_letter"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_if_tag_x_cut_start_conditional() {
    println!("Test case is in: test_cases/if_tag/cut_start_conditional");
    run_test(
        std::path::Path::new("../test_cases/if_tag/cut_start_conditional"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_if_tag_x_if_tag_inverted() {
    println!("Test case is in: test_cases/if_tag/if_tag_inverted");
    run_test(
        std::path::Path::new("../test_cases/if_tag/if_tag_inverted"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_if_tag_x_if_tag_location_to_bool() {
    println!("Test case is in: test_cases/if_tag/if_tag_location_to_bool");
    run_test(
        std::path::Path::new("../test_cases/if_tag/if_tag_location_to_bool"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_gz_input() {
    println!("Test case is in: test_cases/input/compression/gz_input");
    run_test(
        std::path::Path::new("../test_cases/input/compression/gz_input"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_gzip_blocks_spliting_reads() {
    println!("Test case is in: test_cases/input/compression/gzip_blocks_spliting_reads");
    run_test(
        std::path::Path::new("../test_cases/input/compression/gzip_blocks_spliting_reads"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_basic() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/basic");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_error_no_rapid_gzip() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/error_no_rapid_gzip");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/error_no_rapid_gzip"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_no_index_not_created() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/no_index_not_created");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/no_index_not_created"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_single_thread() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/single_thread");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/single_thread"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_with_index() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/with_index");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/with_index"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_compression_x_rapidgzip_x_with_index_and_head() {
    println!("Test case is in: test_cases/input/compression/rapidgzip/with_index_and_head");
    run_test(
        std::path::Path::new("../test_cases/input/compression/rapidgzip/with_index_and_head"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_empty_input() {
    println!("Test case is in: test_cases/input/error_handling/empty_input");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/empty_input"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_array() {
    println!("Test case is in: test_cases/input/error_handling/input_array");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_array"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_nested_map() {
    println!("Test case is in: test_cases/input/error_handling/input_nested_map");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_nested_map"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_nested_map_output_mistake() {
    println!("Test case is in: test_cases/input/error_handling/input_nested_map_output_mistake");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_nested_map_output_mistake"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_key() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_key");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_key"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_value() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_value");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_value"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_non_str_value_nested() {
    println!("Test case is in: test_cases/input/error_handling/input_non_str_value_nested");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_non_str_value_nested"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_input_str() {
    println!("Test case is in: test_cases/input/error_handling/input_str");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/input_str"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_error_handling_x_non_string_values() {
    println!("Test case is in: test_cases/input/error_handling/non_string_values");
    run_test(
        std::path::Path::new("../test_cases/input/error_handling/non_string_values"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_basic() {
    println!("Test case is in: test_cases/input/interleaved/basic");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_mixing_formats() {
    println!("Test case is in: test_cases/input/interleaved/error_mixing_formats");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_mixing_formats"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_mixing_stdin_and_normal_files() {
    println!("Test case is in: test_cases/input/interleaved/error_mixing_stdin_and_normal_files");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_mixing_stdin_and_normal_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_error_only_one_segment() {
    println!("Test case is in: test_cases/input/interleaved/error_only_one_segment");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/error_only_one_segment"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_gzip() {
    println!("Test case is in: test_cases/input/interleaved/gzip");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/gzip"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_must_have_even_block_size() {
    println!("Test case is in: test_cases/input/interleaved/must_have_even_block_size");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/must_have_even_block_size"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_test_premature_termination() {
    println!("Test case is in: test_cases/input/interleaved/test_premature_termination");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/test_premature_termination"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_interleaved_x_two_files() {
    println!("Test case is in: test_cases/input/interleaved/two_files");
    run_test(
        std::path::Path::new("../test_cases/input/interleaved/two_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_interleaved() {
    println!("Test case is in: test_cases/input/stdin/stdin_interleaved");
    run_test(
        std::path::Path::new("../test_cases/input/stdin/stdin_interleaved"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_regular() {
    println!("Test case is in: test_cases/input/stdin/stdin_regular");
    run_test(
        std::path::Path::new("../test_cases/input/stdin/stdin_regular"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_validation_x_empty_bam_in_middle() {
    println!("Test case is in: test_cases/input_validation/empty_bam_in_middle");
    run_test(
        std::path::Path::new("../test_cases/input_validation/empty_bam_in_middle"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breaking_after_sequence_in_partial() {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breaking_after_sequence_in_partial"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breaking_after_sequence_in_partial",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer()
 {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_input_validation_x_fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows()
 {
    println!(
        "Test case is in: test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/input_validation/fastq_breakng_after_sequence_in_partial_but_stop_at_newline_so_check_in_spacer_windows",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_allow_overwrites() {
    println!("Test case is in: test_cases/integration/basic/allow_overwrites");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/allow_overwrites"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_noop() {
    println!("Test case is in: test_cases/integration/basic/noop");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/noop"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_basic_x_noop_minimal() {
    println!("Test case is in: test_cases/integration/basic/noop_minimal");
    run_test(
        std::path::Path::new("../test_cases/integration/basic/noop_minimal"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_416() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_416");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_416"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_491() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_491");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_491"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_606() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_606");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/fastp_606"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_compatibility_x_old_cli_format() {
    println!("Test case is in: test_cases/integration/compatibility/old_cli_format");
    run_test(
        std::path::Path::new("../test_cases/integration/compatibility/old_cli_format"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_location_loss_on_conditional_trim() {
    println!("Test case is in: test_cases/integration/complex/location_loss_on_conditional_trim");
    run_test(
        std::path::Path::new("../test_cases/integration/complex/location_loss_on_conditional_trim"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_order_maintained_in_single_core_transforms() {
    println!(
        "Test case is in: test_cases/integration/complex/order_maintained_in_single_core_transforms"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/integration/complex/order_maintained_in_single_core_transforms",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_complex_x_ten_segments_creative_transforms() {
    println!("Test case is in: test_cases/integration/complex/ten_segments_creative_transforms");
    run_test(
        std::path::Path::new("../test_cases/integration/complex/ten_segments_creative_transforms"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_max_one_block_in_flight() {
    println!("Test case is in: test_cases/integration/edge_cases/max_one_block_in_flight");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/max_one_block_in_flight"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_mega_long_reads() {
    println!("Test case is in: test_cases/integration/edge_cases/mega_long_reads");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/mega_long_reads"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_very_long_reads() {
    println!("Test case is in: test_cases/integration/edge_cases/very_long_reads");
    run_test(
        std::path::Path::new("../test_cases/integration/edge_cases/very_long_reads"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_head_x_tag_histogram_before_head() {
    println!("Test case is in: test_cases/integration/head/tag_histogram_before_head");
    run_test(
        std::path::Path::new("../test_cases/integration/head/tag_histogram_before_head"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_both() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/both");
    run_test(
        std::path::Path::new("../test_cases/integration/io/cut_end_named_pipes/both"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_input_pipe() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/input_pipe");
    run_test(
        std::path::Path::new("../test_cases/integration/io/cut_end_named_pipes/input_pipe"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_io_x_mixed_input_files() {
    println!("Test case is in: test_cases/integration/io/mixed_input_files");
    run_test(
        std::path::Path::new("../test_cases/integration/io/mixed_input_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_x_misc_x_head_with_index() {
    println!("Test case is in: test_cases/integration/misc/head_with_index");
    run_test(
        std::path::Path::new("../test_cases/integration/misc/head_with_index"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_multi() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_multi");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/calc/quantify_regions_multi"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_simple() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_simple");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/calc/quantify_regions_simple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_tests_x_integration_tests_x_input_is_symlink() {
    println!("Test case is in: test_cases/integration_tests/integration_tests/input_is_symlink");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/integration_tests/input_is_symlink"),
        "input_prelink.toml",
        1,
    );
}

#[test]
fn test_cases_x_integration_tests_x_quality_base_replacement() {
    println!("Test case is in: test_cases/integration_tests/quality_base_replacement");
    run_test(
        std::path::Path::new("../test_cases/integration_tests/quality_base_replacement"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_bam_x_basic() {
    println!("Test case is in: test_cases/output/bam/basic");
    run_test(
        std::path::Path::new("../test_cases/output/bam/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_bam_x_interleaved() {
    println!("Test case is in: test_cases/output/bam/interleaved");
    run_test(
        std::path::Path::new("../test_cases/output/bam/interleaved"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_bam() {
    println!("Test case is in: test_cases/output/chunked/bam");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/bam"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_exceeding_100_chunks() {
    println!("Test case is in: test_cases/output/chunked/fastq_exceeding_100_chunks");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/fastq_exceeding_100_chunks"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_gzip() {
    println!("Test case is in: test_cases/output/chunked/fastq_gzip");
    run_test(
        std::path::Path::new("../test_cases/output/chunked/fastq_gzip"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_custom_ix_separator() {
    println!("Test case is in: test_cases/output/custom_ix_separator");
    run_test(
        std::path::Path::new("../test_cases/output/custom_ix_separator"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_custom_ix_separator_table_no_infix() {
    println!("Test case is in: test_cases/output/custom_ix_separator_table_no_infix");
    run_test(
        std::path::Path::new("../test_cases/output/custom_ix_separator_table_no_infix"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_backslash_in_x_sep() {
    println!("Test case is in: test_cases/output/error_handling/backslash_in_x_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/backslash_in_x_sep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_colon_in_ix_sep() {
    println!("Test case is in: test_cases/output/error_handling/colon_in_ix_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/colon_in_ix_sep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_error_handling_x_slash_in_ix_sep() {
    println!("Test case is in: test_cases/output/error_handling/slash_in_ix_sep");
    run_test(
        std::path::Path::new("../test_cases/output/error_handling/slash_in_ix_sep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_hash_output_both() {
    println!("Test case is in: test_cases/output/hash_output_both");
    run_test(
        std::path::Path::new("../test_cases/output/hash_output_both"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_hash_output_compressed() {
    println!("Test case is in: test_cases/output/hash_output_compressed");
    run_test(
        std::path::Path::new("../test_cases/output/hash_output_compressed"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_interleaved_output() {
    println!("Test case is in: test_cases/output/interleaved_output");
    run_test(
        std::path::Path::new("../test_cases/output/interleaved_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_one_report_is_enough_x_html() {
    println!("Test case is in: test_cases/output/one_report_is_enough/html");
    run_test(
        std::path::Path::new("../test_cases/output/one_report_is_enough/html"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_one_report_is_enough_x_json() {
    println!("Test case is in: test_cases/output/one_report_is_enough/json");
    run_test(
        std::path::Path::new("../test_cases/output/one_report_is_enough/json"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_x_chunked_x_fastq() {
    println!("Test case is in: test_cases/output/output/chunked/fastq");
    run_test(
        std::path::Path::new("../test_cases/output/output/chunked/fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_x_fastq() {
    println!("Test case is in: test_cases/output/output/fastq");
    run_test(
        std::path::Path::new("../test_cases/output/output/fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_compression_gzip_level() {
    println!("Test case is in: test_cases/output/output_compression_gzip_level");
    run_test(
        std::path::Path::new("../test_cases/output/output_compression_gzip_level"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_different_suffix() {
    println!("Test case is in: test_cases/output/output_different_suffix");
    run_test(
        std::path::Path::new("../test_cases/output/output_different_suffix"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2");
    run_test(
        std::path::Path::new("../test_cases/output/output_neither_r1_nor_r2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2_but_index2() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2_but_index2");
    run_test(
        std::path::Path::new("../test_cases/output/output_neither_r1_nor_r2_but_index2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_r1_only() {
    println!("Test case is in: test_cases/output/output_r1_only");
    run_test(
        std::path::Path::new("../test_cases/output/output_r1_only"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_output_r2_only() {
    println!("Test case is in: test_cases/output/output_r2_only");
    run_test(
        std::path::Path::new("../test_cases/output/output_r2_only"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_progress_x_basic() {
    println!("Test case is in: test_cases/output/progress/basic");
    run_test(
        std::path::Path::new("../test_cases/output/progress/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_progress_x_followed_by_head() {
    println!("Test case is in: test_cases/output/progress/followed_by_head");
    run_test(
        std::path::Path::new("../test_cases/output/progress/followed_by_head"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_output_x_stdout_output_interleaved() {
    println!("Test case is in: test_cases/output/stdout_output_interleaved");
    run_test(
        std::path::Path::new("../test_cases/output/stdout_output_interleaved"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_auto_x_detect_x_on_x_stderr() {
    println!("Test case is in: test_cases/single_step/auto-detect-on-stderr");
    run_test(
        std::path::Path::new("../test_cases/single_step/auto-detect-on-stderr"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_basic() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_basic_all_segments() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/basic_all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/basic_all_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_input_error_invalid_quality() {
    println!(
        "Test case is in: test_cases/single_step/calc/expected_error/input_error_invalid_quality"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/calc/expected_error/input_error_invalid_quality",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_max() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/max");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/max"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_max_all_segments() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/max_all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/expected_error/max_all_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_basic() {
    println!("Test case is in: test_cases/single_step/calc/kmer/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_basic_higher_min_count() {
    println!("Test case is in: test_cases/single_step/calc/kmer/basic_higher_min_count");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/basic_higher_min_count"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_files_as_sequence() {
    println!("Test case is in: test_cases/single_step/calc/kmer/files_as_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/files_as_sequence"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_phix() {
    println!("Test case is in: test_cases/single_step/calc/kmer/phix");
    run_test(
        std::path::Path::new("../test_cases/single_step/calc/kmer/phix"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_gzip_output() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_gzip_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_gzip_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_read_swap() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_read_swap");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_read_swap"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_zst_output() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_zst_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/compression/zstd_input_zst_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_and_string_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/location_and_string_concat"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_does_not_provide_location() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_outputs_location() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/location_and_string_concat_outputs_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/location_and_string_concat_outputs_location",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_concat() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/location_concat"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_multiple_hits_per_tag() {
    println!("Test case is in: test_cases/single_step/concat_tags/multiple_hits_per_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/multiple_hits_per_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_multiple_hits_per_tag_anchor_right() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/multiple_hits_per_tag_anchor_right"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/multiple_hits_per_tag_anchor_right",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_string_string_concat() {
    println!("Test case is in: test_cases/single_step/concat_tags/string_string_concat");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/string_string_concat"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_three_tags() {
    println!("Test case is in: test_cases/single_step/concat_tags/three_tags");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/three_tags"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_merge_present() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_merge_present");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/concat_tags/with_missing_tag_merge_present",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_set_missing() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_set_missing");
    run_test(
        std::path::Path::new("../test_cases/single_step/concat_tags/with_missing_tag_set_missing"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_convert_to_rate_x_all_segments() {
    println!("Test case is in: test_cases/single_step/convert/convert_to_rate/all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/convert_to_rate/all_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_basic() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/regions_to_length/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_error_in_equal_out() {
    println!(
        "Test case is in: test_cases/single_step/convert/regions_to_length/error_in_equal_out"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert/regions_to_length/error_in_equal_out",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_multiple_regions() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/multiple_regions");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert/regions_to_length/multiple_regions",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_polyx() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/polyx");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/regions_to_length/polyx"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_basic() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_denominator_tag() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/denominator_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/denominator_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_log_variants() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/log_variants");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert/to_rate/log_variants"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/convert_phred"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred_broken_input() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred_broken_input");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/convert_quality/convert_phred_broken_input",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_convert_phred_multi() {
    println!("Test case is in: test_cases/single_step/convert_quality/convert_phred_multi");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/convert_phred_multi"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_convert_quality_x_error_convert_to_same() {
    println!("Test case is in: test_cases/single_step/convert_quality/error_convert_to_same");
    run_test(
        std::path::Path::new("../test_cases/single_step/convert_quality/error_convert_to_same"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_basic() {
    println!("Test case is in: test_cases/single_step/dedup/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_exact() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_exact");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_exact"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_keep_duplicates() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_keep_duplicates");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_keep_duplicates"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read2() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read2");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_demultiplex() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo_demultiplex");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo_demultiplex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_incl_index() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo_incl_index");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/dedup_read_combo_incl_index"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_error_no_seed() {
    println!("Test case is in: test_cases/single_step/dedup/error_no_seed");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/error_no_seed"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_location_tag() {
    println!("Test case is in: test_cases/single_step/dedup/exact_location_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_location_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_name() {
    println!("Test case is in: test_cases/single_step/dedup/exact_name");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_name"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_tag() {
    println!("Test case is in: test_cases/single_step/dedup/exact_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_tag_missing_values() {
    println!("Test case is in: test_cases/single_step/dedup/exact_tag_missing_values");
    run_test(
        std::path::Path::new("../test_cases/single_step/dedup/exact_tag_missing_values"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_sanger() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_longreads_x_cat() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/longreads/cat"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/longreads/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_cat() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_sanger() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/cat");
    run_test(
        std::path::Path::new("../test_cases/single_step/edge_cases/challenging_formats/solexa/cat"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_wrapping_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/wrapping/cat");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edge_cases/challenging_formats/wrapping/cat",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_name() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_name");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/lowercase_name"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_sequence() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/lowercase_sequence"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_tag() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/lowercase_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_concatenate() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_concatenate");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/no_overlap_concatenate"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_keep() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/no_overlap_keep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_perfect_match() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_perfect_match");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/overlap_perfect_match"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_with_tag() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_with_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/overlap_with_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/vs_fastp"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp_systematic() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp_systematic");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/merge_reads/vs_fastp_systematic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp_systematic_but_limit_is_percentage() {
    println!(
        "Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp_systematic_but_limit_is_percentage"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/edits/merge_reads/vs_fastp_systematic_but_limit_is_percentage",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/reverse_complement"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement_conditional() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement_conditional");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/reverse_complement_conditional"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_auto_detect_two_segments() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_auto_detect_two_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_auto_detect_two_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_extended() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_extended");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_extended"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_majority() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_majority");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_majority"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_minority() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_minority");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_conditional_minority"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_four_segments() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_four_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/swap/swap_four_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_sequence() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_sequence");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/uppercase_sequence"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_tag() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/edits/uppercase_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_spotcheckreadpairing_x_not_paired() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/SpotCheckReadPairing/not_paired"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/SpotCheckReadPairing/not_paired",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_both_false() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_mapped() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_unmapped() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_output_uncompressed_hash() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_segment_barcode() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/segment_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/segment_barcode",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_segment_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/segment_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/segment_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_check_name_collisions_x_tag_barcode() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/check_name_collisions/tag_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/check_name_collisions/tag_barcode",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_compression_detection_wrong_extension()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/compression_detection_wrong_extension"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/compression_detection_wrong_extension",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_gzip_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_zstd_level_zero()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_raw_with_compression_level()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_barcode_lengths()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_files() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/different_files",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_non_iupac() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_same_files() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/same_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/demultiplex/barcodes/same_files",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_agtc()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_empty()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_all_on_non_all_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/all_on_non_all_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/all_on_non_all_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_all_on_non_segment_or_name() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/all_on_non_segment_or_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/all_on_non_segment_or_name",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_barcodes_as_list() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/barcodes_as_list"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/barcodes_as_list",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_show_step_template_on_error() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/show_step_template_on_error"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/show_step_template_on_error",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_tag_not_defined() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/tag_not_defined"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/tag_not_defined",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_eserde() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_eserde"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/two_mistakes_eserde",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_post_deserialization()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_eval_expr_x_len_from_not_a_len_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_absolute_with_ignore()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_empty_count() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_empty_count"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_empty_count",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_invalid_letters() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_gc_panic_on_store_in_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_min_length_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_too_many_mismatches()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_from_name_followed_by_uppercase()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_label_starts_with_name()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_region_from_name_but_storing_location()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_region_from_name_but_storing_location"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_region_from_name_but_storing_location",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_tag_from_i1_i2_no_i1_i2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_by_tag_numeric_rejection() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_no_such_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_no_such_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/filter_no_such_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_numeric_filter_wrong_tag_type() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_insert_char_in_value()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_label()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_value()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_same_infix_twice()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_store_tags_in_table_no_tags_defined()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_tag_name_x_tag_name_not_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_trim_tag_multi_locations() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/trim_tag_multi_locations"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/trim_tag_multi_locations",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_unused_extract_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/unused_extract_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/unused_extract_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_validate_regex_fail() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/validate_regex_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/extraction/validate_regex_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_bynumerictagminormax() {
    println!("Test case is in: test_cases/single_step/error_handling/filter/ByNumericTagMinOrMax");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/ByNumericTagMinOrMax",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_other_file_by_seq_x_negative_fpr() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/filter/other_file_by_seq/negative_fpr"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/other_file_by_seq/negative_fpr",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_filter_x_other_file_by_seq_x_no_seed() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/filter/other_file_by_seq/no_seed"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/filter/other_file_by_seq/no_seed",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_invalid_segment_names_x_all() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/invalid_segment_names/all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/invalid_segment_names/all",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_invalid_segment_names_x_internal() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/invalid_segment_names/internal"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/invalid_segment_names/internal",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_no_segments() {
    println!("Test case is in: test_cases/single_step/error_handling/input_config/no_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/input_config/no_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_segment_name_x_duplicated_after_trim()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/segment_name/duplicated_after_trim"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/segment_name/duplicated_after_trim",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_segment_name_x_empty() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/segment_name/empty"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/segment_name/empty",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_segment_name_x_invalid_path() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/segment_name/invalid_path"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/segment_name/invalid_path",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_segment_name_x_invalid_path2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/segment_name/invalid_path2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/segment_name/invalid_path2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_config_x_segment_name_x_whitespace_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_config/segment_name/whitespace_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_config/segment_name/whitespace_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_fake_fasta_missing() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/fake_fasta_missing"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/fake_fasta_missing",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_filter_missing_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/filter_missing_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/filter_missing_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index1_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index1_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/index1_file_does_not_exist",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index2_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index2_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/index2_file_does_not_exist",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_input_file_is_output_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/input_file_is_output_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/input_file_is_output_file",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_missing_input_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/missing_input_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/missing_input_file",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_output_x_interleave_x_missing_target()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/output/interleave/missing_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/output/interleave/missing_target",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read1_more_than_read2()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read2_more_than_read1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_input_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_input_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/permission_denied_input_file",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_read1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_read1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/permission_denied_read1",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_empty_list() {
    println!("Test case is in: test_cases/single_step/error_handling/input_files/read1_empty_list");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_empty_list",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index1_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index2_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_read2_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_not_a_string() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_not_a_string"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read1_not_a_string",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_file_does_not_exist"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read2_file_does_not_exist",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_not_a_string() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_not_a_string"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/read2_not_a_string",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index1"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_index1",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_index2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_one_key() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_one_key"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/repeated_filenames_one_key",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_a() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_b() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_io_x_stdin_multiple_files() {
    println!("Test case is in: test_cases/single_step/error_handling/io/stdin_multiple_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/io/stdin_multiple_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_no_file() {
    println!("Test case is in: test_cases/single_step/error_handling/kmer/no_file");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/kmer/no_file"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_wrong_type_for_files() {
    println!("Test case is in: test_cases/single_step/error_handling/kmer/wrong_type_for_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/kmer/wrong_type_for_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_kmer_x_wrong_type_for_files_nested() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/kmer/wrong_type_for_files_nested"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/kmer/wrong_type_for_files_nested",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_newline",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_newline2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_panics() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_panics"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/broken_panics",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_diff_ids() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_diff_ids"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_diff_ids",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_double_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_double_seq",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_long_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_long_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_long_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_no_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_no_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_no_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_del() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_del"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_del",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_escape() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_escape"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_escape",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_null() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_null"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_null",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_space() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_space"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_space",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_tab() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_tab"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_tab",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_unit_sep() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_vtab() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_short_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_short_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_short_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_spaces() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_spaces");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_spaces",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_tabs() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_tabs");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/malformed_fastq/error_tabs"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_plus() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_plus() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_seq_tiny_blocksize()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq_tiny_blocksize"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq_tiny_blocksize",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_title() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot_too_long() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_long()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_short()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_long()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_short()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_and_truncated_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_at_all() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_at_all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_at_all",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_at_end_ok() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_quality_starts_with_at() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_truncated_after_at() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/truncated_after_at"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/truncated_after_at",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_windows_newlines() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/windows_newlines"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/malformed_fastq/windows_newlines",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_list_of_files() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_list_of_files");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/empty_list_of_files"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_name_input() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_name_input");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/empty_name_input"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_mixed_input_formats() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/mixed_input_formats");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/mixed_input_formats"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_postfix_len_mismatch() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/postfix_len_mismatch");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/postfix_len_mismatch"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_prefix_len_mismatch() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/prefix_len_mismatch");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/prefix_len_mismatch"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_read_with_comment_in_line_3() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/read_with_comment_in_line_3"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/read_with_comment_in_line_3",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_report_names_distinct() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/report_names_distinct");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/misc/report_names_distinct"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_number_to_large() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_number_to_large"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/u8_from_char_number_to_large",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_too_many_chars() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_barcode_outputs_not_named_no_barcode()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_duplicated_segment() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/duplicated_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/duplicated_segment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_empty_output() {
    println!("Test case is in: test_cases/single_step/error_handling/output_config/empty_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/output_config/empty_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_invalid_segment()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_no_segment_specified()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_tag_i1_i2_but_not_output() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_input_interleaved_multiple_segment_files()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_gzip_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_zero()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_empty_output()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_format_raw() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_fifo() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_fifo"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/chunked_fifo",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_stdout() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_stdout"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/chunked_stdout",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_duplicated_target()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_just_one_target()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/just_one_target"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/output/interleave/just_one_target",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_report_without_output_flags() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/report_without_output_flags"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/report_without_output_flags",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_defaults_multiple_segments_fails()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_duplicated_interleave() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_duplicated_interleave"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/segment_duplicated_interleave",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdin_multiple_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdin_multiple_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/stdin_multiple_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdout_conflict() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdout_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/stdout_conflict",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_undefined_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/undefined_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/undefined_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_unwritable_output_dir() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/unwritable_output_dir"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/unwritable_output_dir",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_validate_name_needs_multiple_segments()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_paired_end_x_paired_end_unqueal_read_count_x_read3_more_than_1_2()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_quality_scores_x_convert_phred_raises() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/quality_scores/convert_phred_raises"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/quality_scores/convert_phred_raises",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_html() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_html"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/reports/report_but_no_report_step_html",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_json() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_json"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/reports/report_but_no_report_step_json",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_showing_docs_x_from_action() {
    println!("Test case is in: test_cases/single_step/error_handling/showing_docs/from_action");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/showing_docs/from_action"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_showing_docs_x_from_braces() {
    println!("Test case is in: test_cases/single_step/error_handling/showing_docs/from_braces");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/showing_docs/from_braces"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_auto_detect_too_few_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap_auto_detect_too_few_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap_auto_detect_too_few_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_auto_detect_too_many_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap_auto_detect_too_many_segments"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap_auto_detect_too_many_segments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_partial_specification_a_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap_partial_specification_a_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap_partial_specification_a_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_partial_specification_b_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap_partial_specification_b_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/error_handling/swap/swap_partial_specification_b_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_same_segment() {
    println!("Test case is in: test_cases/single_step/error_handling/swap/swap_same_segment");
    run_test(
        std::path::Path::new("../test_cases/single_step/error_handling/swap/swap_same_segment"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_basic() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_bool() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_bool");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_bool"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_complex() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_complex");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/eval_expr/eval_expression_complex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_location() {
    println!("Test case is in: test_cases/single_step/eval/location");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/location"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_location_len() {
    println!("Test case is in: test_cases/single_step/eval/location_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/location_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_segment_len() {
    println!("Test case is in: test_cases/single_step/eval/segment_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/segment_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_str() {
    println!("Test case is in: test_cases/single_step/eval/str");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/str"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_str_len() {
    println!("Test case is in: test_cases/single_step/eval/str_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/str_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_eval_x_threshold() {
    println!("Test case is in: test_cases/single_step/eval/threshold");
    run_test(
        std::path::Path::new("../test_cases/single_step/eval/threshold"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_end_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_start_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_false() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_true() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_false()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_true()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_after_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_before_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_prefix() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/prefix"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/prefix",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_rev_complement() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_swap() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/swap"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/swap",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_swap_conditional() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/swap_conditional"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/swap_conditional",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_trim_quality_start() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_hamming() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/hamming");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/hamming"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_leftmost_verification() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_anchor/leftmost_verification"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_anchor/leftmost_verification",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_right_most_verification() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_anchor/right_most_verification"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_anchor/right_most_verification",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_simple() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/simple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/too_far"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far_left() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far_left");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_anchor/too_far_left"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_base_content_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_base_content_simple_test");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_base_content_simple_test",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_x_keep() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_filter/keep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_x_remove() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_filter/remove"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_gc/after_trim"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_segment_all_full_data() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/segment_all_full_data");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_gc/segment_all_full_data",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_segment_all_read1_only() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_gc/segment_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_gc/segment_all_read1_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_x_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc/simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_gc/simple_test"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_growing"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing_from_nothing() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing_from_nothing");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_growing_from_nothing"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_highlight/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight_x_regex() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight/regex");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_highlight/regex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/multiple");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/multiple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_multiple_patterns_different_positions()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/multiple_patterns_different_positions"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/multiple_patterns_different_positions",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_nested_patterns() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/nested_patterns"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/nested_patterns",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_multiple_patterns_x_overlapping_patterns()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/multiple_patterns/overlapping_patterns"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/multiple_patterns/overlapping_patterns",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_suffix() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/suffix");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/suffix"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_iupac/with_indel"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_anchor_left() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_anchor_left"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_anchor_left",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_anchor_right() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_anchor_right"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_anchor_right",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_empty_reads() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_empty_reads"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_empty_reads",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_x_with_indel_empty_search() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_iupac/with_indel_empty_search"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_iupac/with_indel_empty_search",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_label_must_not_be_empty() {
    println!("Test case is in: test_cases/single_step/extraction/extract_label_must_not_be_empty");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_label_must_not_be_empty",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length/after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_length/after_trim"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_segment_all_full_data() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length/segment_all_full_data"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_length/segment_all_full_data",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_segment_all_read1_only() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length/segment_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_length/segment_all_read1_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_x_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length/simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_length/simple_test"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_longest_poly_x/any_base",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base_mismatch() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_longest_poly_x/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n/count_after_trim");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_n/count_after_trim"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n/count_simple_test");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_n/count_simple_test"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_target_all_full_data() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n/count_target_all_full_data"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_n/count_target_all_full_data",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_x_count_target_all_read1_only() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n/count_target_all_read1_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_n/count_target_all_read1_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_regex/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_multi_segment() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_no_replacement()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_no_replacement() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_regex_len_table() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex/regex_len_table");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_regex/regex_len_table"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex_underscores");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_regex_underscores"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores_x_ok_works() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex_underscores/ok_works"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_regex_underscores/ok_works",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_and_replace_multiple() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/and_replace_multiple"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/and_replace_multiple",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_beyond_read_len() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/beyond_read_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/beyond_read_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_beyond_read_len_and_trim() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/beyond_read_len_and_trim"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/beyond_read_len_and_trim",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_from_string_tag() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/from_string_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/from_string_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_read_too_short() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region/read_too_short");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_region/read_too_short"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_x_trim_at_tag_conflict() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region/trim_at_tag_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_region/trim_at_tag_conflict",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_shrinking() {
    println!("Test case is in: test_cases/single_step/extraction/extract_shrinking");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_shrinking"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_duplicate_name_panics() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_tag_duplicate_name_panics"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_tag_duplicate_name_panics",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_i1_i2() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_i1_i2");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag_i1_i2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_r2() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_r2");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_tag_r2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_reserved_name_panics() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_reserved_name_panics");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/extract_tag_reserved_name_panics",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_end_false() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/end_false");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/end_false"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_end_true() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/end_true");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/end_true"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_start_false() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/start_false");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/start_false"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_x_start_true() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim/start_true");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/extract_trim/start_true"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_overlapping_regions_trim_conflict() {
    println!(
        "Test case is in: test_cases/single_step/extraction/overlapping_regions_trim_conflict"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/overlapping_regions_trim_conflict",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_remove_nonexistant_tag() {
    println!("Test case is in: test_cases/single_step/extraction/remove_nonexistant_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/remove_nonexistant_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_basic_many_blocks_expose_order() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/basic_many_blocks_expose_order"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/basic_many_blocks_expose_order",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_demultiplex_output_unmatched_x_false()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_error_bam_output() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/error_bam_output"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/error_bam_output",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_error_no_output_def() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/error_no_output_def"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/error_no_output_def",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_fasta() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/fasta"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_gzipped() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/gzipped");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/gzipped"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_head_after() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/head_after");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/head_after"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_no_location() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/no_location");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tag_in_fastq/no_location"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_with_comments() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/with_comments");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/with_comments",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_with_location() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/with_location");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tag_in_fastq/with_location",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_head_after() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/head_after");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/head_after"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/in_tsv"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_demultiplex() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_demultiplex"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tags_in_table/in_tsv_demultiplex",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_gz() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_gz");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/store_tags_in_table/in_tsv_gz"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_table_x_in_tsv_validate_compression() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tags_in_table/in_tsv_validate_compression"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/store_tags_in_table/in_tsv_validate_compression",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract() {
    println!("Test case is in: test_cases/single_step/extraction/umi_extract");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/umi_extract"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_store_in_all_read_names() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_store_in_all_read_names"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/umi_extract_store_in_all_read_names",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_with_existing_comment() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_with_existing_comment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/umi_extract_with_existing_comment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_forget_all_tags() {
    println!("Test case is in: test_cases/single_step/extraction/use_forget_all_tags");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/use_forget_all_tags"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_removed_tag() {
    println!("Test case is in: test_cases/single_step/extraction/use_removed_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/extraction/use_removed_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_extraction_x_with_demultiplex_x_store_tag_in_fastq_x_demultiplex() {
    println!(
        "Test case is in: test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_but_neither_mapped_nor_unmapped() {
    println!(
        "Test case is in: test_cases/single_step/fileformats/bam_but_neither_mapped_nor_unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_but_neither_mapped_nor_unmapped",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/bam_to_fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_both() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/both");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/bam_with_index_to_fastq/both"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_mapped() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/mapped");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_with_index_to_fastq/mapped",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_with_index_to_fastq_x_unmapped() {
    println!(
        "Test case is in: test_cases/single_step/fileformats/bam_with_index_to_fastq/unmapped"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/bam_with_index_to_fastq/unmapped",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_gz_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_gz_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_gz_to_fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_to_fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_with_desc_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_with_desc_to_fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fasta_with_desc_to_fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_with_empty_desc_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_with_empty_desc_to_fastq");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/fileformats/fasta_with_empty_desc_to_fastq",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_bam() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_bam");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fastq_to_bam"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_fasta() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/fastq_to_fasta"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fasta"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta_compressed() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta_compressed");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fasta_compressed"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fastq");
    run_test(
        std::path::Path::new("../test_cases/single_step/fileformats/stdout/fastq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_all() {
    println!("Test case is in: test_cases/single_step/filter/empty/all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/all"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_basic() {
    println!("Test case is in: test_cases/single_step/filter/empty/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_x_segments() {
    println!("Test case is in: test_cases/single_step/filter/empty/segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/empty/segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_low_complexity_x_all_segments() {
    println!("Test case is in: test_cases/single_step/filter/low_complexity/all_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/low_complexity/all_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_low_complexity_x_basic() {
    println!("Test case is in: test_cases/single_step/filter/low_complexity/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/low_complexity/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_max_len() {
    println!("Test case is in: test_cases/single_step/filter/max_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/max_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_min_len() {
    println!("Test case is in: test_cases/single_step/filter/min_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/min_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_distinct_separators() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/distinct_separators"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/distinct_separators",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_distinct_separators_conflict_with_store_in_comment()
 {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/distinct_separators_conflict_with_store_in_comment"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/distinct_separators_conflict_with_store_in_comment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_error_no_reads_in_other_file() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/error_no_reads_in_other_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/error_no_reads_in_other_file",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_keep() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/keep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/remove"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_approximate() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_approximate"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_approximate",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_approximate_no_bai() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_approximate_no_bai"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_approximate_no_bai",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_both() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_both");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_name/remove_bam_both"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_mapped_only() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_mapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_mapped_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_x_remove_bam_unmapped_only() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name/remove_bam_unmapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_name/remove_bam_unmapped_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_error_no_reads_in_other_file() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/error_no_reads_in_other_file"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/error_no_reads_in_other_file",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_keep() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/keep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/remove");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/remove"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_both() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_both");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/other_file_by_seq/remove_bam_both"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_mapped_not_set() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_not_set"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_not_set",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_mapped_only() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_mapped_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_neither_errors() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_neither_errors"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_neither_errors",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_unmapped_not_set() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_not_set"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_not_set",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_x_remove_bam_unmapped_only() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_only"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/filter/other_file_by_seq/remove_bam_unmapped_only",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/above"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above_or_equal() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above_or_equal");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/above_or_equal"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/below"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below_or_equal() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below_or_equal");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/qualified_bases/below_or_equal"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_all() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/all"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_basic() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_x_segments_vs_all() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n/segments_vs_all");
    run_test(
        std::path::Path::new("../test_cases/single_step/filter/too_many_n/segments_vs_all"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_empty() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_empty");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction_empty"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_keep() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_keep");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/basic_correction_keep"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_require_larger_0() {
    println!("Test case is in: test_cases/single_step/hamming/require_larger_0");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/require_larger_0"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_hamming_x_string_tag_correction() {
    println!("Test case is in: test_cases/single_step/hamming/string_tag_correction");
    run_test(
        std::path::Path::new("../test_cases/single_step/hamming/string_tag_correction"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_quantify() {
    println!("Test case is in: test_cases/single_step/head/head_after_quantify");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_quantify"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report() {
    println!("Test case is in: test_cases/single_step/head/head_after_report");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_base_stats() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_base_stats");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_base_stats"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_duplicate_count() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_duplicate_count");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_duplicate_count"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_duplicate_fragment_count() {
    println!(
        "Test case is in: test_cases/single_step/head/head_after_report_duplicate_fragment_count"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/head_after_report_duplicate_fragment_count",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_length_distribution() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_length_distribution");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/head_after_report_length_distribution",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report_oligos() {
    println!("Test case is in: test_cases/single_step/head/head_after_report_oligos");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_after_report_oligos"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_quantify() {
    println!("Test case is in: test_cases/single_step/head/head_before_quantify");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_before_quantify"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_report() {
    println!("Test case is in: test_cases/single_step/head/head_before_report");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_before_report"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_stops_reading"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading_multiple() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading_multiple");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/head_stops_reading_multiple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_bottom() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_bottom");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_bottom"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_middle"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle_bottom() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle_bottom");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/head/multi_stage_head_report_middle_bottom",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_top() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_top");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_report_top"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_spot_check_pairing() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_spot_check_pairing");
    run_test(
        std::path::Path::new("../test_cases/single_step/head/multi_stage_head_spot_check_pairing"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/all_interleaved"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved_reversed() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved_reversed");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/all_interleaved_reversed"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_compression_zstd_level() {
    println!("Test case is in: test_cases/single_step/inspect/compression_zstd_level");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/compression_zstd_level"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index1");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_index1"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index2() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index2");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_index2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read1");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_read1"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read2() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read2");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect/inspect_read2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_read1_compressed() {
    println!("Test case is in: test_cases/single_step/inspect/inspect_read1_compressed");
    run_test(
        std::path::Path::new("../test_cases/single_step/inspect/inspect_read1_compressed"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_output_x_cut_end_named_pipes_x_output_pipe() {
    println!("Test case is in: test_cases/single_step/output/cut_end_named_pipes/output_pipe");
    run_test(
        std::path::Path::new("../test_cases/single_step/output/cut_end_named_pipes/output_pipe"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_output_x_output_neither_r1_nor_r2_but_index() {
    println!("Test case is in: test_cases/single_step/output/output_neither_r1_nor_r2_but_index");
    run_test(
        std::path::Path::new("../test_cases/single_step/output/output_neither_r1_nor_r2_but_index"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_performance_x_duplicate_input_allocation() {
    println!("Test case is in: test_cases/single_step/performance/duplicate_input_allocation");
    run_test(
        std::path::Path::new("../test_cases/single_step/performance/duplicate_input_allocation"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_read_index_placeholder() {
    println!("Test case is in: test_cases/single_step/rename/rename_read_index_placeholder");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_read_index_placeholder"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_gets_longer() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_gets_longer");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex_gets_longer"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_shorter() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_shorter");
    run_test(
        std::path::Path::new("../test_cases/single_step/rename/rename_regex_shorter"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_duplication_count_is_stable() {
    println!("Test case is in: test_cases/single_step/reports/duplication_count_is_stable");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/duplication_count_is_stable"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/oligo_counts"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts_2() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts_2");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/oligo_counts_2"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_progress_init_messages() {
    println!("Test case is in: test_cases/single_step/reports/progress_init_messages");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/progress_init_messages"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_read_length_reporting() {
    println!("Test case is in: test_cases/single_step/reports/read_length_reporting");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/read_length_reporting"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report() {
    println!("Test case is in: test_cases/single_step/reports/report");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_bam() {
    println!("Test case is in: test_cases/single_step/reports/report_bam");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_bam"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_depduplication_per_fragment() {
    println!("Test case is in: test_cases/single_step/reports/report_depduplication_per_fragment");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/reports/report_depduplication_per_fragment",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_fasta() {
    println!("Test case is in: test_cases/single_step/reports/report_fasta");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_fasta"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_fasta_more_than_one_block() {
    println!("Test case is in: test_cases/single_step/reports/report_fasta_more_than_one_block");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_fasta_more_than_one_block"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_no_output() {
    println!("Test case is in: test_cases/single_step/reports/report_no_output");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_no_output"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_ordering() {
    println!("Test case is in: test_cases/single_step/reports/report_ordering");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_ordering"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_pe() {
    println!("Test case is in: test_cases/single_step/reports/report_pe");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_pe"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_tag_histogram() {
    println!("Test case is in: test_cases/single_step/reports/report_tag_histogram");
    run_test(
        std::path::Path::new("../test_cases/single_step/reports/report_tag_histogram"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_multi_segments() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_multi_segments");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample_multi_segments"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_twice() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_twice");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/reservoir_sample_twice"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_skip() {
    println!("Test case is in: test_cases/single_step/sampling/skip");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/skip"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_sampling_x_subsample() {
    println!("Test case is in: test_cases/single_step/sampling/subsample");
    run_test(
        std::path::Path::new("../test_cases/single_step/sampling/subsample"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_no_insert_char_present() {
    println!("Test case is in: test_cases/single_step/store_tag/in_comment_no_insert_char_present");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/store_tag/in_comment_no_insert_char_present",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_with_insert_char_present() {
    println!(
        "Test case is in: test_cases/single_step/store_tag/in_comment_with_insert_char_present"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/store_tag/in_comment_with_insert_char_present",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_transform_x_max_len() {
    println!("Test case is in: test_cases/single_step/transform/max_len");
    run_test(
        std::path::Path::new("../test_cases/single_step/transform/max_len"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_lowqualityend() {
    println!("Test case is in: test_cases/single_step/trim/LowQualityEnd");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/LowQualityEnd"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_end_x_basic() {
    println!("Test case is in: test_cases/single_step/trim/cut_end/basic");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_end/basic"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_end_x_if_tag() {
    println!("Test case is in: test_cases/single_step/trim/cut_end/if_tag");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_end/if_tag"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_start() {
    println!("Test case is in: test_cases/single_step/trim/cut_start");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/cut_start"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_detail() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/detail");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/detail"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_detail_g() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/detail_g");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/detail_g"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_long() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/long");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/long"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_n() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/n");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/n"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_x_n_keep_end() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail/n_keep_end");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_poly_tail/n_keep_end"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_x_end() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual/end");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_qual/end"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_x_start() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual/start");
    run_test(
        std::path::Path::new("../test_cases/single_step/trim/trim_qual/start"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_disabled() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/disabled");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/spot_check_read_pairing/disabled",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_fail() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/spot_check_read_pairing/fail"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_not_sampled_no_error() {
    println!(
        "Test case is in: test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_simple() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/spot_check_read_pairing/simple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_all() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/all"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/all",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_all_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/all_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/all_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_basic() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/basic"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/basic",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_name() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/name"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/name",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_name_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/name_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/name_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_string_type() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_x_with_tag_string_type_fail()
 {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type_fail"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_all_reads_same_length/with_tag_string_type_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_simple() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/simple");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_name/simple"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_custom_separator() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_name/validate_name_custom_separator"
    );
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_name/validate_name_custom_separator",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/validate_name_fail");
    run_test(
        std::path::Path::new(
            "../test_cases/single_step/validation/validate_name/validate_name_fail",
        ),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_phred"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred_fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_phred_fail"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_seq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq_fail");
    run_test(
        std::path::Path::new("../test_cases/single_step/validation/validate_seq_fail"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_transform_x_prefix_and_postfix() {
    println!("Test case is in: test_cases/transform/prefix_and_postfix");
    run_test(
        std::path::Path::new("../test_cases/transform/prefix_and_postfix"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_01_x_basic_x_quality_x_report() {
    println!("Test case is in: cookbooks/01-basic-quality-report");
    run_test(
        std::path::Path::new("../cookbooks/01-basic-quality-report"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_02_x_umi_x_extraction() {
    println!("Test case is in: cookbooks/02-umi-extraction");
    run_test(
        std::path::Path::new("../cookbooks/02-umi-extraction"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_03_x_lexogen_x_quantseq() {
    println!("Test case is in: cookbooks/03-lexogen-quantseq");
    run_test(
        std::path::Path::new("../cookbooks/03-lexogen-quantseq"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_04_x_phix_x_removal() {
    println!("Test case is in: cookbooks/04-phiX-removal");
    run_test(
        std::path::Path::new("../cookbooks/04-phiX-removal"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_04_x_phix_x_removal_2() {
    println!("Test case is in: cookbooks/04-phiX-removal");
    run_test(
        std::path::Path::new("../cookbooks/04-phiX-removal"),
        "input_demultiplex.toml",
        2,
    );
}

#[test]
fn test_cases_x_05_x_quality_x_filtering() {
    println!("Test case is in: cookbooks/05-quality-filtering");
    run_test(
        std::path::Path::new("../cookbooks/05-quality-filtering"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_06_x_adapter_x_trimming() {
    println!("Test case is in: cookbooks/06-adapter-trimming");
    run_test(
        std::path::Path::new("../cookbooks/06-adapter-trimming"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_07_x_demultiplexing() {
    println!("Test case is in: cookbooks/07-demultiplexing");
    run_test(
        std::path::Path::new("../cookbooks/07-demultiplexing"),
        "input.toml",
        1,
    );
}

#[test]
fn test_cases_x_08_x_length_x_filtering() {
    println!("Test case is in: cookbooks/08-length-filtering");
    run_test(
        std::path::Path::new("../cookbooks/08-length-filtering"),
        "input.toml",
        1,
    );
}
