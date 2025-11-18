// this file is written by dev/update_tests.py
// there is a test case that will inform you if tests are missing and you need
// to rerun dev/update_tests.py
mod test_runner;
use test_runner::run_test;

#[test]
fn test_cases_x_demultiplex_x_bool() {
    println!("Test case is in: test_cases/demultiplex/bool");
    run_test(std::path::Path::new("../test_cases/demultiplex/bool"));
}

#[test]
fn test_cases_x_demultiplex_x_bool_with_unmatched() {
    println!("Test case is in: test_cases/demultiplex/bool_with_unmatched");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/bool_with_unmatched",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_chunked_interleaved_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/chunked_interleaved_output_demultiplex");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/chunked_interleaved_output_demultiplex",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_chunked_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/chunked_output_demultiplex");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/chunked_output_demultiplex",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_duplicates() {
    println!("Test case is in: test_cases/demultiplex/duplicates");
    run_test(std::path::Path::new("../test_cases/demultiplex/duplicates"));
}

#[test]
fn test_cases_x_demultiplex_x_head_with_index_and_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/head_with_index_and_demultiplex");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/head_with_index_and_demultiplex",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_inspect() {
    println!("Test case is in: test_cases/demultiplex/inspect");
    run_test(std::path::Path::new("../test_cases/demultiplex/inspect"));
}

#[test]
fn test_cases_x_demultiplex_x_interleaved_output_demultiplex() {
    println!("Test case is in: test_cases/demultiplex/interleaved_output_demultiplex");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/interleaved_output_demultiplex",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_multiple_barcode_and_bool() {
    println!("Test case is in: test_cases/demultiplex/multiple_barcode_and_bool");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/multiple_barcode_and_bool",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_no_matching_barcodes() {
    println!("Test case is in: test_cases/demultiplex/no_matching_barcodes");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/no_matching_barcodes",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_quantify_tag() {
    println!("Test case is in: test_cases/demultiplex/quantify_tag");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/quantify_tag",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_reservoir_sample() {
    println!("Test case is in: test_cases/demultiplex/reservoir_sample");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/reservoir_sample",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_x_hamming() {
    println!("Test case is in: test_cases/demultiplex/simple-hamming");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple-hamming",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_bam_output() {
    println!("Test case is in: test_cases/demultiplex/simple_bam_output");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_bam_output",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_basics() {
    println!("Test case is in: test_cases/demultiplex/simple_basics");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_basics",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order1() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order1");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_combined_outputs/order1",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_combined_outputs_x_order2_invariant() {
    println!("Test case is in: test_cases/demultiplex/simple_combined_outputs/order2_invariant");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_combined_outputs/order2_invariant",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_iupac",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_hamming() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_hamming");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_iupac_hamming",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_iupac_two_regions() {
    println!("Test case is in: test_cases/demultiplex/simple_iupac_two_regions");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_iupac_two_regions",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_multiple_into_one_output() {
    println!("Test case is in: test_cases/demultiplex/simple_multiple_into_one_output");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_multiple_into_one_output",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_no_unmatched() {
    println!("Test case is in: test_cases/demultiplex/simple_no_unmatched");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_no_unmatched",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_single_barcode",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_simple_single_barcode_no_unmatched_output() {
    println!("Test case is in: test_cases/demultiplex/simple_single_barcode_no_unmatched_output");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/simple_single_barcode_no_unmatched_output",
    ));
}

#[test]
fn test_cases_x_demultiplex_x_two_barcodes() {
    println!("Test case is in: test_cases/demultiplex/two_barcodes");
    run_test(std::path::Path::new(
        "../test_cases/demultiplex/two_barcodes",
    ));
}

#[test]
fn test_cases_x_error_handling_x_bam_x_disk_full_bam() {
    println!("Test case is in: test_cases/error_handling/bam/disk_full_bam");
    run_test(std::path::Path::new(
        "../test_cases/error_handling/bam/disk_full_bam",
    ));
}

#[test]
fn test_cases_x_error_handling_x_misc_x_disk_full_fastq() {
    println!("Test case is in: test_cases/error_handling/misc/disk_full_fastq");
    run_test(std::path::Path::new(
        "../test_cases/error_handling/misc/disk_full_fastq",
    ));
}

#[test]
fn test_cases_x_if_tag_x_cut_start_conditional() {
    println!("Test case is in: test_cases/if_tag/cut_start_conditional");
    run_test(std::path::Path::new(
        "../test_cases/if_tag/cut_start_conditional",
    ));
}

#[test]
fn test_cases_x_if_tag_x_if_tag_inverted() {
    println!("Test case is in: test_cases/if_tag/if_tag_inverted");
    run_test(std::path::Path::new("../test_cases/if_tag/if_tag_inverted"));
}

#[test]
fn test_cases_x_if_tag_x_if_tag_location_to_bool() {
    println!("Test case is in: test_cases/if_tag/if_tag_location_to_bool");
    run_test(std::path::Path::new(
        "../test_cases/if_tag/if_tag_location_to_bool",
    ));
}

#[test]
fn test_cases_x_input_x_compression_x_gz_input() {
    println!("Test case is in: test_cases/input/compression/gz_input");
    run_test(std::path::Path::new(
        "../test_cases/input/compression/gz_input",
    ));
}

#[test]
fn test_cases_x_input_x_compression_x_gzip_blocks_spliting_reads() {
    println!("Test case is in: test_cases/input/compression/gzip_blocks_spliting_reads");
    run_test(std::path::Path::new(
        "../test_cases/input/compression/gzip_blocks_spliting_reads",
    ));
}

#[test]
fn test_cases_x_input_x_interleaved() {
    println!("Test case is in: test_cases/input/interleaved");
    run_test(std::path::Path::new("../test_cases/input/interleaved"));
}

#[test]
fn test_cases_x_input_x_interleaved_must_have_even_block_size() {
    println!("Test case is in: test_cases/input/interleaved_must_have_even_block_size");
    run_test(std::path::Path::new(
        "../test_cases/input/interleaved_must_have_even_block_size",
    ));
}

#[test]
fn test_cases_x_input_x_interleaved_test_premature_termination() {
    println!("Test case is in: test_cases/input/interleaved_test_premature_termination");
    run_test(std::path::Path::new(
        "../test_cases/input/interleaved_test_premature_termination",
    ));
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_interleaved() {
    println!("Test case is in: test_cases/input/stdin/stdin_interleaved");
    run_test(std::path::Path::new(
        "../test_cases/input/stdin/stdin_interleaved",
    ));
}

#[test]
fn test_cases_x_input_x_stdin_x_stdin_regular() {
    println!("Test case is in: test_cases/input/stdin/stdin_regular");
    run_test(std::path::Path::new(
        "../test_cases/input/stdin/stdin_regular",
    ));
}

#[test]
fn test_cases_x_integration_x_basic_x_allow_overwrites() {
    println!("Test case is in: test_cases/integration/basic/allow_overwrites");
    run_test(std::path::Path::new(
        "../test_cases/integration/basic/allow_overwrites",
    ));
}

#[test]
fn test_cases_x_integration_x_basic_x_noop() {
    println!("Test case is in: test_cases/integration/basic/noop");
    run_test(std::path::Path::new("../test_cases/integration/basic/noop"));
}

#[test]
fn test_cases_x_integration_x_basic_x_noop_minimal() {
    println!("Test case is in: test_cases/integration/basic/noop_minimal");
    run_test(std::path::Path::new(
        "../test_cases/integration/basic/noop_minimal",
    ));
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_416() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_416");
    run_test(std::path::Path::new(
        "../test_cases/integration/compatibility/fastp_416",
    ));
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_491() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_491");
    run_test(std::path::Path::new(
        "../test_cases/integration/compatibility/fastp_491",
    ));
}

#[test]
fn test_cases_x_integration_x_compatibility_x_fastp_606() {
    println!("Test case is in: test_cases/integration/compatibility/fastp_606");
    run_test(std::path::Path::new(
        "../test_cases/integration/compatibility/fastp_606",
    ));
}

#[test]
fn test_cases_x_integration_x_compatibility_x_old_cli_format() {
    println!("Test case is in: test_cases/integration/compatibility/old_cli_format");
    run_test(std::path::Path::new(
        "../test_cases/integration/compatibility/old_cli_format",
    ));
}

#[test]
fn test_cases_x_integration_x_complex_x_order_maintained_in_single_core_transforms() {
    println!(
        "Test case is in: test_cases/integration/complex/order_maintained_in_single_core_transforms"
    );
    run_test(std::path::Path::new(
        "../test_cases/integration/complex/order_maintained_in_single_core_transforms",
    ));
}

#[test]
fn test_cases_x_integration_x_complex_x_ten_segments_creative_transforms() {
    println!("Test case is in: test_cases/integration/complex/ten_segments_creative_transforms");
    run_test(std::path::Path::new(
        "../test_cases/integration/complex/ten_segments_creative_transforms",
    ));
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_mega_long_reads() {
    println!("Test case is in: test_cases/integration/edge_cases/mega_long_reads");
    run_test(std::path::Path::new(
        "../test_cases/integration/edge_cases/mega_long_reads",
    ));
}

#[test]
fn test_cases_x_integration_x_edge_cases_x_very_long_reads() {
    println!("Test case is in: test_cases/integration/edge_cases/very_long_reads");
    run_test(std::path::Path::new(
        "../test_cases/integration/edge_cases/very_long_reads",
    ));
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_both() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/both");
    run_test(std::path::Path::new(
        "../test_cases/integration/io/cut_end_named_pipes/both",
    ));
}

#[test]
fn test_cases_x_integration_x_io_x_cut_end_named_pipes_x_input_pipe() {
    println!("Test case is in: test_cases/integration/io/cut_end_named_pipes/input_pipe");
    run_test(std::path::Path::new(
        "../test_cases/integration/io/cut_end_named_pipes/input_pipe",
    ));
}

#[test]
fn test_cases_x_integration_x_misc_x_head_with_index() {
    println!("Test case is in: test_cases/integration/misc/head_with_index");
    run_test(std::path::Path::new(
        "../test_cases/integration/misc/head_with_index",
    ));
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_multi() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_multi");
    run_test(std::path::Path::new(
        "../test_cases/integration_tests/calc/quantify_regions_multi",
    ));
}

#[test]
fn test_cases_x_integration_tests_x_calc_x_quantify_regions_simple() {
    println!("Test case is in: test_cases/integration_tests/calc/quantify_regions_simple");
    run_test(std::path::Path::new(
        "../test_cases/integration_tests/calc/quantify_regions_simple",
    ));
}

#[test]
fn test_cases_x_integration_tests_x_quality_base_replacement() {
    println!("Test case is in: test_cases/integration_tests/quality_base_replacement");
    run_test(std::path::Path::new(
        "../test_cases/integration_tests/quality_base_replacement",
    ));
}

#[test]
fn test_cases_x_output_x_bam() {
    println!("Test case is in: test_cases/output/bam");
    run_test(std::path::Path::new("../test_cases/output/bam"));
}

#[test]
fn test_cases_x_output_x_chunked_x_bam() {
    println!("Test case is in: test_cases/output/chunked/bam");
    run_test(std::path::Path::new("../test_cases/output/chunked/bam"));
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_exceeding_10k_chunks() {
    println!("Test case is in: test_cases/output/chunked/fastq_exceeding_10k_chunks");
    run_test(std::path::Path::new(
        "../test_cases/output/chunked/fastq_exceeding_10k_chunks",
    ));
}

#[test]
fn test_cases_x_output_x_chunked_x_fastq_gzip() {
    println!("Test case is in: test_cases/output/chunked/fastq_gzip");
    run_test(std::path::Path::new(
        "../test_cases/output/chunked/fastq_gzip",
    ));
}

#[test]
fn test_cases_x_output_x_custom_ix_separator() {
    println!("Test case is in: test_cases/output/custom_ix_separator");
    run_test(std::path::Path::new(
        "../test_cases/output/custom_ix_separator",
    ));
}

#[test]
fn test_cases_x_output_x_custom_ix_separator_table_no_infix() {
    println!("Test case is in: test_cases/output/custom_ix_separator_table_no_infix");
    run_test(std::path::Path::new(
        "../test_cases/output/custom_ix_separator_table_no_infix",
    ));
}

#[test]
fn test_cases_x_output_x_hash_output_both() {
    println!("Test case is in: test_cases/output/hash_output_both");
    run_test(std::path::Path::new(
        "../test_cases/output/hash_output_both",
    ));
}

#[test]
fn test_cases_x_output_x_hash_output_compressed() {
    println!("Test case is in: test_cases/output/hash_output_compressed");
    run_test(std::path::Path::new(
        "../test_cases/output/hash_output_compressed",
    ));
}

#[test]
fn test_cases_x_output_x_interleaved_output() {
    println!("Test case is in: test_cases/output/interleaved_output");
    run_test(std::path::Path::new(
        "../test_cases/output/interleaved_output",
    ));
}

#[test]
fn test_cases_x_output_x_output_x_chunked_x_fastq() {
    println!("Test case is in: test_cases/output/output/chunked/fastq");
    run_test(std::path::Path::new(
        "../test_cases/output/output/chunked/fastq",
    ));
}

#[test]
fn test_cases_x_output_x_output_x_fastq() {
    println!("Test case is in: test_cases/output/output/fastq");
    run_test(std::path::Path::new("../test_cases/output/output/fastq"));
}

#[test]
fn test_cases_x_output_x_output_compression_gzip_level() {
    println!("Test case is in: test_cases/output/output_compression_gzip_level");
    run_test(std::path::Path::new(
        "../test_cases/output/output_compression_gzip_level",
    ));
}

#[test]
fn test_cases_x_output_x_output_different_suffix() {
    println!("Test case is in: test_cases/output/output_different_suffix");
    run_test(std::path::Path::new(
        "../test_cases/output/output_different_suffix",
    ));
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2");
    run_test(std::path::Path::new(
        "../test_cases/output/output_neither_r1_nor_r2",
    ));
}

#[test]
fn test_cases_x_output_x_output_neither_r1_nor_r2_but_index2() {
    println!("Test case is in: test_cases/output/output_neither_r1_nor_r2_but_index2");
    run_test(std::path::Path::new(
        "../test_cases/output/output_neither_r1_nor_r2_but_index2",
    ));
}

#[test]
fn test_cases_x_output_x_output_r1_only() {
    println!("Test case is in: test_cases/output/output_r1_only");
    run_test(std::path::Path::new("../test_cases/output/output_r1_only"));
}

#[test]
fn test_cases_x_output_x_output_r2_only() {
    println!("Test case is in: test_cases/output/output_r2_only");
    run_test(std::path::Path::new("../test_cases/output/output_r2_only"));
}

#[test]
fn test_cases_x_output_x_stdout_output_interleaved() {
    println!("Test case is in: test_cases/output/stdout_output_interleaved");
    run_test(std::path::Path::new(
        "../test_cases/output/stdout_output_interleaved",
    ));
}

#[test]
fn test_cases_x_output_x_timing_x_chunked_x_fastq() {
    println!("Test case is in: test_cases/output/timing/chunked/fastq");
    run_test(std::path::Path::new(
        "../test_cases/output/timing/chunked/fastq",
    ));
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_basic() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/calc/expected_error/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_input_error_invalid_quality() {
    println!(
        "Test case is in: test_cases/single_step/calc/expected_error/input_error_invalid_quality"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/calc/expected_error/input_error_invalid_quality",
    ));
}

#[test]
fn test_cases_x_single_step_x_calc_x_expected_error_x_max() {
    println!("Test case is in: test_cases/single_step/calc/expected_error/max");
    run_test(std::path::Path::new(
        "../test_cases/single_step/calc/expected_error/max",
    ));
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_basic() {
    println!("Test case is in: test_cases/single_step/calc/kmer/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/calc/kmer/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_calc_x_kmer_x_phix() {
    println!("Test case is in: test_cases/single_step/calc/kmer/phix");
    run_test(std::path::Path::new(
        "../test_cases/single_step/calc/kmer/phix",
    ));
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input");
    run_test(std::path::Path::new(
        "../test_cases/single_step/compression/zstd_input",
    ));
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_gzip_output() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_gzip_output");
    run_test(std::path::Path::new(
        "../test_cases/single_step/compression/zstd_input_gzip_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_read_swap() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_read_swap");
    run_test(std::path::Path::new(
        "../test_cases/single_step/compression/zstd_input_read_swap",
    ));
}

#[test]
fn test_cases_x_single_step_x_compression_x_zstd_input_zst_output() {
    println!("Test case is in: test_cases/single_step/compression/zstd_input_zst_output");
    run_test(std::path::Path::new(
        "../test_cases/single_step/compression/zstd_input_zst_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_and_string_concat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/location_and_string_concat",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_and_string_concat_does_not_provide_location() {
    println!(
        "Test case is in: test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/location_and_string_concat_does_not_provide_location",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_location_concat() {
    println!("Test case is in: test_cases/single_step/concat_tags/location_concat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/location_concat",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_multiple_hits_per_tag() {
    println!("Test case is in: test_cases/single_step/concat_tags/multiple_hits_per_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/multiple_hits_per_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_three_tags() {
    println!("Test case is in: test_cases/single_step/concat_tags/three_tags");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/three_tags",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_merge_present() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_merge_present");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/with_missing_tag_merge_present",
    ));
}

#[test]
fn test_cases_x_single_step_x_concat_tags_x_with_missing_tag_set_missing() {
    println!("Test case is in: test_cases/single_step/concat_tags/with_missing_tag_set_missing");
    run_test(std::path::Path::new(
        "../test_cases/single_step/concat_tags/with_missing_tag_set_missing",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_convert_to_rate_x_all_segments() {
    println!("Test case is in: test_cases/single_step/convert/convert_to_rate/all_segments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/convert_to_rate/all_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_basic() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/regions_to_length/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_multiple_regions() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/multiple_regions");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/regions_to_length/multiple_regions",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_regions_to_length_x_polyx() {
    println!("Test case is in: test_cases/single_step/convert/regions_to_length/polyx");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/regions_to_length/polyx",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_basic() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/to_rate/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_denominator_tag() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/denominator_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/to_rate/denominator_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_convert_x_to_rate_x_log_variants() {
    println!("Test case is in: test_cases/single_step/convert/to_rate/log_variants");
    run_test(std::path::Path::new(
        "../test_cases/single_step/convert/to_rate/log_variants",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_basic() {
    println!("Test case is in: test_cases/single_step/dedup/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_exact() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_exact");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/dedup_exact",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_keep_duplicates() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_keep_duplicates");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/dedup_keep_duplicates",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read2() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/dedup_read2",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/dedup_read_combo",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_dedup_read_combo_incl_index() {
    println!("Test case is in: test_cases/single_step/dedup/dedup_read_combo_incl_index");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/dedup_read_combo_incl_index",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_name() {
    println!("Test case is in: test_cases/single_step/dedup/exact_name");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/exact_name",
    ));
}

#[test]
fn test_cases_x_single_step_x_dedup_x_exact_tag() {
    println!("Test case is in: test_cases/single_step/dedup/exact_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/dedup/exact_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/cat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/illumina/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_sanger() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_sanger",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_illumina_x_to_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/illumina/to_solexa",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_longreads_x_cat() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/longreads/cat"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/longreads/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_illumina",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/as_solexa",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_dna_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_dna/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_illumina",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/as_solexa",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_misc_rna_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/misc_rna/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_illumina",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_as_solexa() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/as_solexa",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_sanger_full_range_x_cat() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/sanger_full_range/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_illumina() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_illumina",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_as_sanger() {
    println!(
        "Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/solexa/as_sanger",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_solexa_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/solexa/cat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/solexa/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edge_cases_x_challenging_formats_x_wrapping_x_cat() {
    println!("Test case is in: test_cases/single_step/edge_cases/challenging_formats/wrapping/cat");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edge_cases/challenging_formats/wrapping/cat",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_sequence() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_sequence");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/lowercase_sequence",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_lowercase_tag() {
    println!("Test case is in: test_cases/single_step/edits/lowercase_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/lowercase_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_concatenate() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_concatenate");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/merge_reads/no_overlap_concatenate",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_no_overlap_keep() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/no_overlap_keep");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/merge_reads/no_overlap_keep",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_perfect_match() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_perfect_match");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/merge_reads/overlap_perfect_match",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_overlap_with_tag() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/overlap_with_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/merge_reads/overlap_with_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_merge_reads_x_vs_fastp() {
    println!("Test case is in: test_cases/single_step/edits/merge_reads/vs_fastp");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/merge_reads/vs_fastp",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/reverse_complement",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_reverse_complement_conditional() {
    println!("Test case is in: test_cases/single_step/edits/reverse_complement_conditional");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/reverse_complement_conditional",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_auto_detect_two_segments() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_auto_detect_two_segments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/swap/swap_auto_detect_two_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/swap/swap_conditional",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_extended() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_extended");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/swap/swap_conditional_extended",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_majority() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_majority");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/swap/swap_conditional_majority",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_swap_x_swap_conditional_minority() {
    println!("Test case is in: test_cases/single_step/edits/swap/swap_conditional_minority");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/swap/swap_conditional_minority",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_sequence() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_sequence");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/uppercase_sequence",
    ));
}

#[test]
fn test_cases_x_single_step_x_edits_x_uppercase_tag() {
    println!("Test case is in: test_cases/single_step/edits/uppercase_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/edits/uppercase_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_both_false() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/both_false",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_mapped() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/mapped",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_missing_input_settings_x_unmapped() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/bam/bam_missing_input_settings/unmapped",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_bam_x_bam_output_uncompressed_hash() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/bam/bam_output_uncompressed_hash",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_cli_x_cli_invalid_command() {
    println!("Test case is in: test_cases/single_step/error_handling/cli/cli_invalid_command");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/cli/cli_invalid_command",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_cli_x_help() {
    println!("Test case is in: test_cases/single_step/error_handling/cli/help");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/cli/help",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_cli_x_old_cli_not_existant_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/cli/old_cli_not_existant_file"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/cli/old_cli_not_existant_file",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_compression_detection_wrong_extension()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/compression_detection_wrong_extension"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/compression/compression_detection_wrong_extension",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_gzip_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_gzip_level_too_high",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_inspect_zstd_level_zero()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/compression/invalid_compression_levels/inspect_zstd_level_zero",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_compression_x_invalid_compression_levels_x_raw_with_compression_level()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/compression/invalid_compression_levels/raw_with_compression_level",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_barcode_lengths()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/demultiplex/barcodes/different_barcode_lengths",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_different_files() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/different_files"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/demultiplex/barcodes/different_files",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_non_iupac() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/demultiplex/barcodes/non_iupac",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_demultiplex_x_barcodes_x_same_files() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/demultiplex/barcodes/same_files"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/demultiplex/barcodes/same_files",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_agtc()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_agtc",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_dna_validation_x_dna_validation_count_oligos_non_empty()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/dna_validation/dna_validation_count_oligos_non_empty",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_show_step_template_on_error() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/show_step_template_on_error"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/error_messages/show_step_template_on_error",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_tag_not_defined() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/tag_not_defined"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/error_messages/tag_not_defined",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_eserde() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_eserde"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/error_messages/two_mistakes_eserde",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_error_messages_x_two_mistakes_post_deserialization()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/error_messages/two_mistakes_post_deserialization",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_eval_expr_x_len_from_not_a_len_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/eval_expr/len_from_not_a_len_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_absolute_with_ignore()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_base_content_absolute_with_ignore",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_empty_count() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_empty_count"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_base_content_empty_count",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_base_content_invalid_letters() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_base_content_invalid_letters",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_gc_panic_on_store_in_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_gc_panic_on_store_in_seq",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_min_length_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_min_length_too_high",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_iupac_suffix_too_many_mismatches()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_iupac_suffix_too_many_mismatches",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_from_name_followed_by_uppercase()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_regex/from_name_followed_by_uppercase",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_regex_x_label_starts_with_name()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_regex/label_starts_with_name",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_extract_tag_from_i1_i2_no_i1_i2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/extract_tag_from_i1_i2_no_i1_i2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_by_tag_numeric_rejection() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/filter_by_tag_numeric_rejection",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_filter_no_such_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/filter_no_such_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/filter_no_such_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_numeric_filter_wrong_tag_type() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/numeric_filter_wrong_tag_type",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_insert_char_in_value()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/insert_char_in_value",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_label()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_label",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tag_in_comment_x_seperator_in_value()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/store_tag_in_comment/seperator_in_value",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_same_infix_twice()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/store_tags_in_table/same_infix_twice",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_store_tags_in_table_x_store_tags_in_table_no_tags_defined()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/store_tags_in_table/store_tags_in_table_no_tags_defined",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_tag_name_x_tag_name_not_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/tag_name/tag_name_not_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_trim_tag_multi_locations() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/trim_tag_multi_locations"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/trim_tag_multi_locations",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_unused_extract_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/unused_extract_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/unused_extract_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_extraction_x_validate_regex_fail() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/extraction/validate_regex_fail"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/extraction/validate_regex_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_fake_fasta_missing() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/fake_fasta_missing"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/fake_fasta_missing",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_filter_missing_tag() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/filter_missing_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/filter_missing_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index1_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index1_file_does_not_exist"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/index1_file_does_not_exist",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_index2_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/index2_file_does_not_exist"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/index2_file_does_not_exist",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_input_file_is_output_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/input_file_is_output_file"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/input_file_is_output_file",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_missing_input_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/missing_input_file"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/missing_input_file",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_output_x_interleave_x_missing_target()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/output/interleave/missing_target"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/output/interleave/missing_target",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read1_more_than_read2()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read1_more_than_read2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_paired_end_unqueal_read_count_x_read2_more_than_read1()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/paired_end_unqueal_read_count/read2_more_than_read1",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_input_file() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_input_file"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/permission_denied_input_file",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_permission_denied_read1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/permission_denied_read1"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/permission_denied_read1",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_empty_list() {
    println!("Test case is in: test_cases/single_step/error_handling/input_files/read1_empty_list");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read1_empty_list",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index1_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read1_len_neq_index1_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_index2_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read1_len_neq_index2_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_len_neq_read2_len() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read1_len_neq_read2_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read1_not_a_string() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read1_not_a_string"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read1_not_a_string",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_file_does_not_exist() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_file_does_not_exist"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read2_file_does_not_exist",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_read2_not_a_string() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/read2_not_a_string"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/read2_not_a_string",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/repeated_filenames",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index1() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index1"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/repeated_filenames_index1",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_index2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_index2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/repeated_filenames_index2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_repeated_filenames_one_key() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/repeated_filenames_one_key"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/repeated_filenames_one_key",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_a() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_a",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_input_files_x_swap_x_swap_missing_segment_b() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/input_files/swap/swap_missing_segment_b",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_io_x_stdin_multiple_files() {
    println!("Test case is in: test_cases/single_step/error_handling/io/stdin_multiple_files");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/io/stdin_multiple_files",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/broken_newline",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_newline2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_newline2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/broken_newline2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_broken_panics() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/broken_panics"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/broken_panics",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_diff_ids() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_diff_ids"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_diff_ids",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_double_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_double_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_double_seq"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_double_seq",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_long_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_long_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_long_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_no_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_no_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_no_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_del() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_del"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_del",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_escape() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_escape"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_escape",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_null() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_null"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_null",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_space() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_space"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_space",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_tab() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_tab"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_tab",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_unit_sep() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_unit_sep",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_qual_vtab() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_qual_vtab",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_short_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_short_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_short_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_spaces() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_spaces");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_spaces",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_tabs() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_tabs");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_tabs",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_plus() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_plus",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_at_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_at_seq",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_plus() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_plus",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_seq() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_seq",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_error_trunc_in_title() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/error_trunc_in_title",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base() {
    println!("Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/invalid_base",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_invalid_base_or_dot_too_long() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/invalid_base_or_dot_too_long",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_long()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_long",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_1st_read_qual_too_short()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_1st_read_qual_too_short",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_long()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_long",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_mismatched_seq_qual_len_2nd_read_qual_too_short()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/mismatched_seq_qual_len_2nd_read_qual_too_short",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_and_truncated_qual() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/no_newline_and_truncated_qual",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_no_newline_at_end_ok() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/no_newline_at_end_ok",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_quality_starts_with_at() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/quality_starts_with_at",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_truncated_after_at() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/truncated_after_at"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/truncated_after_at",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_malformed_fastq_x_windows_newlines() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/malformed_fastq/windows_newlines"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/malformed_fastq/windows_newlines",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_list_of_files() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_list_of_files");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/empty_list_of_files",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_empty_name_input() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/empty_name_input");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/empty_name_input",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_mixed_input_formats() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/mixed_input_formats");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/mixed_input_formats",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_postfix_len_mismatch() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/postfix_len_mismatch");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/postfix_len_mismatch",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_prefix_len_mismatch() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/prefix_len_mismatch");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/prefix_len_mismatch",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_read_with_comment_in_line_3() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/read_with_comment_in_line_3"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/read_with_comment_in_line_3",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_report_names_distinct() {
    println!("Test case is in: test_cases/single_step/error_handling/misc/report_names_distinct");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/report_names_distinct",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_number_to_large() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_number_to_large"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/u8_from_char_number_to_large",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_misc_x_u8_from_char_too_many_chars() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/misc/u8_from_char_too_many_chars",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_barcode_outputs_not_named_no_barcode()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/barcode_outputs_not_named_no_barcode",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_empty_output() {
    println!("Test case is in: test_cases/single_step/error_handling/output_config/empty_output");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/empty_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_invalid_segment()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/extract_regex/name_invalid_segment",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_regex_x_name_no_segment_specified()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/extract_regex/name_no_segment_specified",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_extract_tag_i1_i2_but_not_output() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/extract_tag_i1_i2_but_not_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_input_interleaved_multiple_segment_files()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/input_interleaved_multiple_segment_files",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_gzip_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_gzip_level_too_high",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_too_high()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_too_high",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_compression_levels_x_output_zstd_level_zero()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/invalid_compression_levels/output_zstd_level_zero",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_segment_names_x_all() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_segment_names/all"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/invalid_segment_names/all",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_invalid_segment_names_x_internal() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/invalid_segment_names/internal"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/invalid_segment_names/internal",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_empty_output()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/no_output_no_reports/empty_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_output_no_reports_x_format_raw() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/no_output_no_reports/format_raw",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_no_segments() {
    println!("Test case is in: test_cases/single_step/error_handling/output_config/no_segments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/no_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_fifo() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_fifo"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/output/chunked_fifo",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_chunked_stdout() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/chunked_stdout"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/output/chunked_stdout",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_duplicated_target()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/output/interleave/duplicated_target",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_output_x_interleave_x_just_one_target()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/output/interleave/just_one_target"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/output/interleave/just_one_target",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_report_without_output_flags() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/report_without_output_flags"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/report_without_output_flags",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_defaults_multiple_segments_fails()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_defaults_multiple_segments_fails",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_duplicated_interleave() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_duplicated_interleave"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_duplicated_interleave",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_duplicated_after_trim()
{
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_duplicated_after_trim"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_name_duplicated_after_trim",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_empty() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_empty"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_name_empty",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_invalid_path() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_invalid_path"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_name_invalid_path",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_invalid_path2() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_invalid_path2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_name_invalid_path2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_segment_name_whitespace_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/segment_name_whitespace_only"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/segment_name_whitespace_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdin_multiple_segments() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdin_multiple_segments"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/stdin_multiple_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_stdout_conflict() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/stdout_conflict"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/stdout_conflict",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_auto_detect_too_few_segments()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_few_segments"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_few_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_auto_detect_too_many_segments()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_many_segments"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/swap/swap_auto_detect_too_many_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_swap_x_swap_same_segment() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/swap/swap_same_segment"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/swap/swap_same_segment",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_unwritable_output_dir() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/unwritable_output_dir"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/unwritable_output_dir",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_output_config_x_validate_name_needs_multiple_segments()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/output_config/validate_name_needs_multiple_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_paired_end_x_paired_end_unqueal_read_count_x_read3_more_than_1_2()
 {
    println!(
        "Test case is in: test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/paired_end/paired_end_unqueal_read_count/read3_more_than_1_2",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_quality_scores_x_convert_phred_raises() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/quality_scores/convert_phred_raises"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/quality_scores/convert_phred_raises",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_html() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_html"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/reports/report_but_no_report_step_html",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_reports_x_report_but_no_report_step_json() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/reports/report_but_no_report_step_json"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/reports/report_but_no_report_step_json",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_x_swap_partial_specification_a_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap/swap_partial_specification_a_only"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/swap/swap/swap_partial_specification_a_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_error_handling_x_swap_x_swap_x_swap_partial_specification_b_only() {
    println!(
        "Test case is in: test_cases/single_step/error_handling/swap/swap/swap_partial_specification_b_only"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/error_handling/swap/swap/swap_partial_specification_b_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_basic() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/eval_expr/eval_expression_basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_bool() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_bool");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/eval_expr/eval_expression_bool",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_eval_expr_x_eval_expression_complex() {
    println!("Test case is in: test_cases/single_step/eval/eval_expr/eval_expression_complex");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/eval_expr/eval_expression_complex",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_location() {
    println!("Test case is in: test_cases/single_step/eval/location");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/location",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_location_len() {
    println!("Test case is in: test_cases/single_step/eval/location_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/location_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_segment_len() {
    println!("Test case is in: test_cases/single_step/eval/segment_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/segment_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_str() {
    println!("Test case is in: test_cases/single_step/eval/str");
    run_test(std::path::Path::new("../test_cases/single_step/eval/str"));
}

#[test]
fn test_cases_x_single_step_x_eval_x_str_len() {
    println!("Test case is in: test_cases/single_step/eval/str_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/str_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_eval_x_threshold() {
    println!("Test case is in: test_cases/single_step/eval/threshold");
    run_test(std::path::Path::new(
        "../test_cases/single_step/eval/threshold",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_end_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_end_inside_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_cut_start_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/cut_start_inside_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_false() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_false",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_end_true() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_end_true",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_false()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_false",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_extract_trim_start_true()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/extract_trim_start_true",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_after_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_after_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_before_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_before_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_max_len_inside_tag() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/max_len_inside_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_prefix() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/prefix"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/prefix",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_rev_complement() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/rev_complement",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_swap() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/swap"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/swap",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_edits_altering_tag_locations_x_trim_quality_start() {
    println!(
        "Test case is in: test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/edits_altering_tag_locations/trim_quality_start",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_hamming() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/hamming");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_anchor/hamming",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_leftmost_verification() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_anchor/leftmost_verification"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_anchor/leftmost_verification",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_simple() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/simple");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_anchor/simple",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_anchor/too_far",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_anchor_x_too_far_left() {
    println!("Test case is in: test_cases/single_step/extraction/extract_anchor/too_far_left");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_anchor/too_far_left",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_base_content_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_base_content_simple_test");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_base_content_simple_test",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_keep() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter_keep");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_filter_keep",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_filter_remove() {
    println!("Test case is in: test_cases/single_step/extraction/extract_filter_remove");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_filter_remove",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc_after_trim");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_gc_after_trim",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc_simple_test");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_gc_simple_test",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_target_all_full_data() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc_target_all_full_data");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_gc_target_all_full_data",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_gc_target_all_read1_only() {
    println!("Test case is in: test_cases/single_step/extraction/extract_gc_target_all_read1_only");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_gc_target_all_read1_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_growing",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_growing_from_nothing() {
    println!("Test case is in: test_cases/single_step/extraction/extract_growing_from_nothing");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_growing_from_nothing",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_highlight",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_highlight_regex() {
    println!("Test case is in: test_cases/single_step/extraction/extract_highlight_regex");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_highlight_regex",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_multiple() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac_multiple");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_iupac_multiple",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_suffix() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac_suffix");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_iupac_suffix",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_iupac_with_indel() {
    println!("Test case is in: test_cases/single_step/extraction/extract_iupac_with_indel");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_iupac_with_indel",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_label_must_not_be_empty() {
    println!("Test case is in: test_cases/single_step/extraction/extract_label_must_not_be_empty");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_label_must_not_be_empty",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length_after_trim");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_length_after_trim",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_length_simple_test");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_length_simple_test",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_target_all_full_data() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length_target_all_full_data"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_length_target_all_full_data",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_length_target_all_read1_only() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_length_target_all_read1_only"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_length_target_all_read1_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_longest_poly_x/any_base",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_any_base_mismatch() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_longest_poly_x/any_base_mismatch",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_longest_poly_x_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/extract_longest_poly_x/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_longest_poly_x/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_count_after_trim() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n_count_after_trim");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_n_count_after_trim",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_count_simple_test() {
    println!("Test case is in: test_cases/single_step/extraction/extract_n_count_simple_test");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_n_count_simple_test",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_count_target_all_full_data() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n_count_target_all_full_data"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_n_count_target_all_full_data",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_n_count_target_all_read1_only() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_n_count_target_all_read1_only"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_n_count_target_all_read1_only",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_multi_segment() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_multi_segment",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_from_name_no_replacement()
{
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex/extract_regex_from_name_no_replacement",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_x_extract_regex_no_replacement() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex/extract_regex_no_replacement",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores() {
    println!("Test case is in: test_cases/single_step/extraction/extract_regex_underscores");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex_underscores",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_regex_underscores_x_ok_works() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_regex_underscores/ok_works"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_regex_underscores/ok_works",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_and_replace_multiple() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region_and_replace_multiple"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_region_and_replace_multiple",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_beyond_read_len() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region_beyond_read_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_region_beyond_read_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_beyond_read_len_and_trim() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region_beyond_read_len_and_trim"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_region_beyond_read_len_and_trim",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_read_too_short() {
    println!("Test case is in: test_cases/single_step/extraction/extract_region_read_too_short");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_region_read_too_short",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_region_trim_at_tag_conflict() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_region_trim_at_tag_conflict"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_region_trim_at_tag_conflict",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_shrinking() {
    println!("Test case is in: test_cases/single_step/extraction/extract_shrinking");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_shrinking",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_duplicate_name_panics() {
    println!(
        "Test case is in: test_cases/single_step/extraction/extract_tag_duplicate_name_panics"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_tag_duplicate_name_panics",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_i1_i2() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_i1_i2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_tag_i1_i2",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_r2() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_r2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_tag_r2",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_tag_reserved_name_panics() {
    println!("Test case is in: test_cases/single_step/extraction/extract_tag_reserved_name_panics");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_tag_reserved_name_panics",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_end_false() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim_end_false");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_trim_end_false",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_end_true() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim_end_true");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_trim_end_true",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_start_false() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim_start_false");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_trim_start_false",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_extract_trim_start_true() {
    println!("Test case is in: test_cases/single_step/extraction/extract_trim_start_true");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/extract_trim_start_true",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_overlapping_regions_trim_conflict() {
    println!(
        "Test case is in: test_cases/single_step/extraction/overlapping_regions_trim_conflict"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/overlapping_regions_trim_conflict",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_remove_nonexistant_tag() {
    println!("Test case is in: test_cases/single_step/extraction/remove_nonexistant_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/remove_nonexistant_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_basic() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/basic");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tag_in_fastq/basic",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_demultiplex_output_unmatched_x_false()
 {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tag_in_fastq/demultiplex_output_unmatched=false",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_gzipped() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/gzipped");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tag_in_fastq/gzipped",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_no_location() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/no_location");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tag_in_fastq/no_location",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tag_in_fastq_x_with_comments() {
    println!("Test case is in: test_cases/single_step/extraction/store_tag_in_fastq/with_comments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tag_in_fastq/with_comments",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_tsv() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_tsv");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tags_in_tsv",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_tsv_demultiplex() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_tsv_demultiplex");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tags_in_tsv_demultiplex",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_tsv_gz() {
    println!("Test case is in: test_cases/single_step/extraction/store_tags_in_tsv_gz");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tags_in_tsv_gz",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_store_tags_in_tsv_validate_compression() {
    println!(
        "Test case is in: test_cases/single_step/extraction/store_tags_in_tsv_validate_compression"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/store_tags_in_tsv_validate_compression",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract() {
    println!("Test case is in: test_cases/single_step/extraction/umi_extract");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/umi_extract",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_store_in_all_read_names() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_store_in_all_read_names"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/umi_extract_store_in_all_read_names",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_umi_extract_with_existing_comment() {
    println!(
        "Test case is in: test_cases/single_step/extraction/umi_extract_with_existing_comment"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/umi_extract_with_existing_comment",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_forget_all_tags() {
    println!("Test case is in: test_cases/single_step/extraction/use_forget_all_tags");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/use_forget_all_tags",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_use_removed_tag() {
    println!("Test case is in: test_cases/single_step/extraction/use_removed_tag");
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/use_removed_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_extraction_x_with_demultiplex_x_store_tag_in_fastq_x_demultiplex() {
    println!(
        "Test case is in: test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/extraction/with_demultiplex/store_tag_in_fastq/demultiplex",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_bam_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/bam_to_fastq");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/bam_to_fastq",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fasta_to_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/fasta_to_fastq");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/fasta_to_fastq",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_bam() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_bam");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/fastq_to_bam",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_fastq_to_fasta() {
    println!("Test case is in: test_cases/single_step/fileformats/fastq_to_fasta");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/fastq_to_fasta",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/stdout/fasta",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fasta_compressed() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fasta_compressed");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/stdout/fasta_compressed",
    ));
}

#[test]
fn test_cases_x_single_step_x_fileformats_x_stdout_x_fastq() {
    println!("Test case is in: test_cases/single_step/fileformats/stdout/fastq");
    run_test(std::path::Path::new(
        "../test_cases/single_step/fileformats/stdout/fastq",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty() {
    println!("Test case is in: test_cases/single_step/filter/empty");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/empty",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_all() {
    println!("Test case is in: test_cases/single_step/filter/empty_all");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/empty_all",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_empty_segments() {
    println!("Test case is in: test_cases/single_step/filter/empty_segments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/empty_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_low_complexity() {
    println!("Test case is in: test_cases/single_step/filter/low_complexity");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/low_complexity",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_max_len() {
    println!("Test case is in: test_cases/single_step/filter/max_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/max_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_min_len() {
    println!("Test case is in: test_cases/single_step/filter/min_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/min_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_distinct_separators() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name_distinct_separators"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_distinct_separators",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_keep() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name_keep");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_keep",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name_remove");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove_bam() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_name_remove_bam");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove_bam",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove_bam_approximate() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name_remove_bam_approximate"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove_bam_approximate",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove_bam_approximate_no_bai() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name_remove_bam_approximate_no_bai"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove_bam_approximate_no_bai",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove_bam_unaligned() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name_remove_bam_unaligned"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove_bam_unaligned",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_name_remove_bam_unaligned_no_ignore() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_name_remove_bam_unaligned_no_ignore"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_name_remove_bam_unaligned_no_ignore",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_keep() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq_keep");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_seq_keep",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_remove() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq_remove");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_seq_remove",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_remove_bam() {
    println!("Test case is in: test_cases/single_step/filter/other_file_by_seq_remove_bam");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_seq_remove_bam",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_remove_bam_unaligned() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq_remove_bam_unaligned"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_seq_remove_bam_unaligned",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_other_file_by_seq_remove_bam_unaligned_no_ignore() {
    println!(
        "Test case is in: test_cases/single_step/filter/other_file_by_seq_remove_bam_unaligned_no_ignore"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/other_file_by_seq_remove_bam_unaligned_no_ignore",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/qualified_bases/above",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_above_or_equal() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/above_or_equal");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/qualified_bases/above_or_equal",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/qualified_bases/below",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_qualified_bases_x_below_or_equal() {
    println!("Test case is in: test_cases/single_step/filter/qualified_bases/below_or_equal");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/qualified_bases/below_or_equal",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/too_many_n",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_all() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n_all");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/too_many_n_all",
    ));
}

#[test]
fn test_cases_x_single_step_x_filter_x_too_many_n_segments_vs_all() {
    println!("Test case is in: test_cases/single_step/filter/too_many_n_segments_vs_all");
    run_test(std::path::Path::new(
        "../test_cases/single_step/filter/too_many_n_segments_vs_all",
    ));
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction");
    run_test(std::path::Path::new(
        "../test_cases/single_step/hamming/basic_correction",
    ));
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_empty() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_empty");
    run_test(std::path::Path::new(
        "../test_cases/single_step/hamming/basic_correction_empty",
    ));
}

#[test]
fn test_cases_x_single_step_x_hamming_x_basic_correction_keep() {
    println!("Test case is in: test_cases/single_step/hamming/basic_correction_keep");
    run_test(std::path::Path::new(
        "../test_cases/single_step/hamming/basic_correction_keep",
    ));
}

#[test]
fn test_cases_x_single_step_x_hamming_x_string_tag_correction() {
    println!("Test case is in: test_cases/single_step/hamming/string_tag_correction");
    run_test(std::path::Path::new(
        "../test_cases/single_step/hamming/string_tag_correction",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_quantify() {
    println!("Test case is in: test_cases/single_step/head/head_after_quantify");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_after_quantify",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_after_report() {
    println!("Test case is in: test_cases/single_step/head/head_after_report");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_after_report",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_quantify() {
    println!("Test case is in: test_cases/single_step/head/head_before_quantify");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_before_quantify",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_before_report() {
    println!("Test case is in: test_cases/single_step/head/head_before_report");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_before_report",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_stops_reading",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_head_stops_reading_multiple() {
    println!("Test case is in: test_cases/single_step/head/head_stops_reading_multiple");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/head_stops_reading_multiple",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/multi_stage_head",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_bottom() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_bottom");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/multi_stage_head_report_bottom",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/multi_stage_head_report_middle",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_middle_bottom() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_middle_bottom");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/multi_stage_head_report_middle_bottom",
    ));
}

#[test]
fn test_cases_x_single_step_x_head_x_multi_stage_head_report_top() {
    println!("Test case is in: test_cases/single_step/head/multi_stage_head_report_top");
    run_test(std::path::Path::new(
        "../test_cases/single_step/head/multi_stage_head_report_top",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/all_interleaved",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_all_interleaved_reversed() {
    println!("Test case is in: test_cases/single_step/inspect/all_interleaved_reversed");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/all_interleaved_reversed",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_compression_zstd_level() {
    println!("Test case is in: test_cases/single_step/inspect/compression_zstd_level");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/compression_zstd_level",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index1");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/inspect/inspect_index1",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_index2() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_index2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/inspect/inspect_index2",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read1() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read1");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/inspect/inspect_read1",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_x_inspect_read2() {
    println!("Test case is in: test_cases/single_step/inspect/inspect/inspect_read2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/inspect/inspect_read2",
    ));
}

#[test]
fn test_cases_x_single_step_x_inspect_x_inspect_read1_compressed() {
    println!("Test case is in: test_cases/single_step/inspect/inspect_read1_compressed");
    run_test(std::path::Path::new(
        "../test_cases/single_step/inspect/inspect_read1_compressed",
    ));
}

#[test]
fn test_cases_x_single_step_x_output_x_cut_end_named_pipes_x_output_pipe() {
    println!("Test case is in: test_cases/single_step/output/cut_end_named_pipes/output_pipe");
    run_test(std::path::Path::new(
        "../test_cases/single_step/output/cut_end_named_pipes/output_pipe",
    ));
}

#[test]
fn test_cases_x_single_step_x_output_x_output_neither_r1_nor_r2_but_index() {
    println!("Test case is in: test_cases/single_step/output/output_neither_r1_nor_r2_but_index");
    run_test(std::path::Path::new(
        "../test_cases/single_step/output/output_neither_r1_nor_r2_but_index",
    ));
}

#[test]
fn test_cases_x_single_step_x_performance_x_duplicate_input_allocation() {
    println!("Test case is in: test_cases/single_step/performance/duplicate_input_allocation");
    run_test(std::path::Path::new(
        "../test_cases/single_step/performance/duplicate_input_allocation",
    ));
}

#[test]
fn test_cases_x_single_step_x_quality_x_convert_phred() {
    println!("Test case is in: test_cases/single_step/quality/convert_phred");
    run_test(std::path::Path::new(
        "../test_cases/single_step/quality/convert_phred",
    ));
}

#[test]
fn test_cases_x_single_step_x_quality_x_convert_phred_broken_input() {
    println!("Test case is in: test_cases/single_step/quality/convert_phred_broken_input");
    run_test(std::path::Path::new(
        "../test_cases/single_step/quality/convert_phred_broken_input",
    ));
}

#[test]
fn test_cases_x_single_step_x_quality_x_convert_phred_multi() {
    println!("Test case is in: test_cases/single_step/quality/convert_phred_multi");
    run_test(std::path::Path::new(
        "../test_cases/single_step/quality/convert_phred_multi",
    ));
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_read_index_placeholder() {
    println!("Test case is in: test_cases/single_step/rename/rename_read_index_placeholder");
    run_test(std::path::Path::new(
        "../test_cases/single_step/rename/rename_read_index_placeholder",
    ));
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex");
    run_test(std::path::Path::new(
        "../test_cases/single_step/rename/rename_regex",
    ));
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_gets_longer() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_gets_longer");
    run_test(std::path::Path::new(
        "../test_cases/single_step/rename/rename_regex_gets_longer",
    ));
}

#[test]
fn test_cases_x_single_step_x_rename_x_rename_regex_shorter() {
    println!("Test case is in: test_cases/single_step/rename/rename_regex_shorter");
    run_test(std::path::Path::new(
        "../test_cases/single_step/rename/rename_regex_shorter",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_duplication_count_is_stable() {
    println!("Test case is in: test_cases/single_step/reports/duplication_count_is_stable");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/duplication_count_is_stable",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/oligo_counts",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_oligo_counts_2() {
    println!("Test case is in: test_cases/single_step/reports/oligo_counts_2");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/oligo_counts_2",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_progress_init_messages() {
    println!("Test case is in: test_cases/single_step/reports/progress_init_messages");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/progress_init_messages",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_read_length_reporting() {
    println!("Test case is in: test_cases/single_step/reports/read_length_reporting");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/read_length_reporting",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_report() {
    println!("Test case is in: test_cases/single_step/reports/report");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/report",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_depduplication_per_fragment() {
    println!("Test case is in: test_cases/single_step/reports/report_depduplication_per_fragment");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/report_depduplication_per_fragment",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_no_output() {
    println!("Test case is in: test_cases/single_step/reports/report_no_output");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/report_no_output",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_ordering() {
    println!("Test case is in: test_cases/single_step/reports/report_ordering");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/report_ordering",
    ));
}

#[test]
fn test_cases_x_single_step_x_reports_x_report_pe() {
    println!("Test case is in: test_cases/single_step/reports/report_pe");
    run_test(std::path::Path::new(
        "../test_cases/single_step/reports/report_pe",
    ));
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample");
    run_test(std::path::Path::new(
        "../test_cases/single_step/sampling/reservoir_sample",
    ));
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_multi_segments() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_multi_segments");
    run_test(std::path::Path::new(
        "../test_cases/single_step/sampling/reservoir_sample_multi_segments",
    ));
}

#[test]
fn test_cases_x_single_step_x_sampling_x_reservoir_sample_twice() {
    println!("Test case is in: test_cases/single_step/sampling/reservoir_sample_twice");
    run_test(std::path::Path::new(
        "../test_cases/single_step/sampling/reservoir_sample_twice",
    ));
}

#[test]
fn test_cases_x_single_step_x_sampling_x_skip() {
    println!("Test case is in: test_cases/single_step/sampling/skip");
    run_test(std::path::Path::new(
        "../test_cases/single_step/sampling/skip",
    ));
}

#[test]
fn test_cases_x_single_step_x_sampling_x_subsample() {
    println!("Test case is in: test_cases/single_step/sampling/subsample");
    run_test(std::path::Path::new(
        "../test_cases/single_step/sampling/subsample",
    ));
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_no_insert_char_present() {
    println!("Test case is in: test_cases/single_step/store_tag/in_comment_no_insert_char_present");
    run_test(std::path::Path::new(
        "../test_cases/single_step/store_tag/in_comment_no_insert_char_present",
    ));
}

#[test]
fn test_cases_x_single_step_x_store_tag_x_in_comment_with_insert_char_present() {
    println!(
        "Test case is in: test_cases/single_step/store_tag/in_comment_with_insert_char_present"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/store_tag/in_comment_with_insert_char_present",
    ));
}

#[test]
fn test_cases_x_single_step_x_transform_x_max_len() {
    println!("Test case is in: test_cases/single_step/transform/max_len");
    run_test(std::path::Path::new(
        "../test_cases/single_step/transform/max_len",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_end() {
    println!("Test case is in: test_cases/single_step/trim/cut_end");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/cut_end",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_cut_start() {
    println!("Test case is in: test_cases/single_step/trim/cut_start");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/cut_start",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_detail() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail_detail");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_poly_tail_detail",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_detail_g() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail_detail_g");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_poly_tail_detail_g",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_long() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail_long");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_poly_tail_long",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_poly_tail_n() {
    println!("Test case is in: test_cases/single_step/trim/trim_poly_tail_n");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_poly_tail_n",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_end() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual_end");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_qual_end",
    ));
}

#[test]
fn test_cases_x_single_step_x_trim_x_trim_qual_start() {
    println!("Test case is in: test_cases/single_step/trim/trim_qual_start");
    run_test(std::path::Path::new(
        "../test_cases/single_step/trim/trim_qual_start",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_disabled() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/disabled");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/spot_check_read_pairing/disabled",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_fail() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/fail");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/spot_check_read_pairing/fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_not_sampled_no_error() {
    println!(
        "Test case is in: test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/spot_check_read_pairing/not_sampled_no_error",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_spot_check_read_pairing_x_simple() {
    println!("Test case is in: test_cases/single_step/validation/spot_check_read_pairing/simple");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/spot_check_read_pairing/simple",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length() {
    println!("Test case is in: test_cases/single_step/validation/validate_all_reads_same_length");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length_fail"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_name() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length_name"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length_name",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_name_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length_name_fail"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length_name_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_with_tag() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length_with_tag"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length_with_tag",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_all_reads_same_length_with_tag_fail() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_all_reads_same_length_with_tag_fail"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_all_reads_same_length_with_tag_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_simple() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/simple");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_name/simple",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_custom_separator() {
    println!(
        "Test case is in: test_cases/single_step/validation/validate_name/validate_name_custom_separator"
    );
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_name/validate_name_custom_separator",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_name_x_validate_name_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_name/validate_name_fail");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_name/validate_name_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_phred",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_phred_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_phred_fail");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_phred_fail",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_seq",
    ));
}

#[test]
fn test_cases_x_single_step_x_validation_x_validate_seq_fail() {
    println!("Test case is in: test_cases/single_step/validation/validate_seq_fail");
    run_test(std::path::Path::new(
        "../test_cases/single_step/validation/validate_seq_fail",
    ));
}

#[test]
fn test_cases_x_transform_x_prefix_and_postfix() {
    println!("Test case is in: test_cases/transform/prefix_and_postfix");
    run_test(std::path::Path::new(
        "../test_cases/transform/prefix_and_postfix",
    ));
}
