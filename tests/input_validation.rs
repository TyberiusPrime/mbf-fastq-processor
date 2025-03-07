/// As in 'input validation' tests
mod common;
use common::*;

#[test]
#[should_panic(expected = "Invalid base found in sequence")]
fn test_validate_seq_fail() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
    action = 'ValidateSeq'
    allowed = 'CGAT' # note the missing n
    target = 'Read1'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Invalid phred quality found")]
fn test_validate_phred_fail() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'ValidatePhred'
    target = 'Read1'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Phred 64-33 conversion yielded values below 33")]
fn test_convert_phred_raises() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[transform]]
    action = 'ConvertPhred64To33'


[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Unexpected symbol where @ was expected")]
fn test_broken_panics() {
    run("
[input]
    read1 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Unexpected symbol where @ was expected in input.")]
fn test_broken_newline() {
    run("
[input]
    read1 = 'sample_data/broken/ten_reads_broken_newline.fq'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Parsing failure, two newlines in sequence")]
fn test_broken_newline2() {
    run("
[input]
    read1 = 'sample_data/broken/ten_reads_broken_newline2.fq'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "If interleaved is set, read2 must not be set")]
fn test_input_read2_interleaved_conflict() {
    //
    run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
    interleaved = true

[output]
    prefix = 'output'
    stdout = true

");
}


#[test]
#[should_panic(expected = "Report labels must be distinct. Duplicated: \"report\"")]
fn test_report_names_distinct() {
    //
    let _td = run("
[input]
    read1 = 'sample_data/ten_reads_of_var_sizes.fq'

[[step]]
    action = 'Report'
    label = 'report'

[[step]]
    action = 'Report'
    label = 'report'

[output]
    prefix = 'output'
    report_json = true
");
}


#[test]
#[should_panic(expected = "Only one level of demultiplexing is supported")]
fn test_only_one_demultiplex() {
    let _td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'gggg'
    TT = 'gggg'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=2, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'gggg'
    TT = 'gggg'

");
}

#[test]
#[should_panic(expected = "Barcode output infixes must be distinct. Duplicated: 'gggg'")]
fn test_barcode_outputs_are_distinct() {
    //
    let _td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'gggg'
    TT = 'gggg'
");
}

#[test]
#[should_panic(
    expected = "Barcode length 2 doesn't match sum of region lengths (4). Offending barcode: (separators ommited): TT"
)]
fn test_barcode_length_mismatch() {
    //
    let _td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
        {source = 'read1', start=3, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT_AA = 'gggg'
    TT = 'gggg'
");
}

#[test]
#[should_panic(expected = "Barcode output infix must not be 'no-barcode'")]
fn test_barcode_outputs_not_named_no_barcode() {
    //
    let _td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'aaaa'
    TT = 'no-barcode'
");
}

#[test]
#[should_panic(expected = "Can't output to stdout and log progress to stdout. ")]
fn test_stdout_conflict() {
    //
    run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[[transform]]
    action = 'Progress'
    n = 10000

[output]
    prefix = 'output'
    stdout = true

");
}

#[test]
#[should_panic(expected = "nterleaving requires read2 files to be specified.")]
fn test_interleave_no_read2() {
    //
    run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'

[output]
    prefix = 'output'
    interleave = true

");
}

#[test]
fn test_empty_name_input() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/broken/empty_name_after_250_reads.fq'
",
    );
    assert!(stderr.contains("Empty name"));
    assert!(exit_code != 0);
}

#[test]
fn test_truncated_after_at() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/broken/truncated_after_at_at_250_reads.fq'
",
    );
    dbg!(stderr.clone());
    assert!(stderr.contains("Empty name"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_file_does_not_exist() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/nosuchfile.fq'
",
    );
    dbg!(stderr.clone());
    assert!(stderr.contains("No such file"));
    assert!(stderr.contains("sample_data/nosuchfile.fq"));
    assert!(stderr.contains("read1"));
    assert!(exit_code != 0);
}

#[test]
fn test_read2_file_does_not_exist() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/nosuchfile.fq'
",
    );
    dbg!(stderr.clone());
    assert!(stderr.contains("No such file"));
    assert!(stderr.contains("sample_data/nosuchfile.fq"));
    assert!(stderr.contains("read2"));
    assert!(exit_code != 0);
}

#[test]
fn test_index1_file_does_not_exist() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    index1 = 'sample_data/nosuchfile.fq'
    index2 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true
",
    );
    dbg!(stderr.clone());
    assert!(stderr.contains("No such file"));
    assert!(stderr.contains("sample_data/nosuchfile.fq"));
    assert!(stderr.contains("index1"));
    assert!(exit_code != 0);
}

#[test]
fn test_index2_file_does_not_exist() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    index2 = 'sample_data/nosuchfile.fq'
    index1 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true
",
    );
    dbg!(stderr.clone());
    assert!(stderr.contains("No such file"));
    assert!(stderr.contains("sample_data/nosuchfile.fq"));
    assert!(stderr.contains("index2"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_not_a_string() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 23
    ",
    );
    assert!(stderr.contains("expected string or list of strings"));
    assert!(stderr.contains("read1"));
    assert!(exit_code != 0);
}

#[test]
fn test_read2_not_a_string() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = true
    ",
    );
    assert!(stderr.contains("expected string or list of strings"));
    assert!(stderr.contains("read2"));
    assert!(exit_code != 0);
}

#[test]
fn test_read2_but_not_read1_set() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read2 = 'sample_data/ten_reads.fq'
    ",
    );
    assert!(stderr.contains("missing field"));
    assert!(exit_code != 0);
}

#[test]
fn test_repeated_filenames() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads.fq'
    ",
    );
    assert!(stderr.contains("Repeated filename"));
    assert!(stderr.contains("options.accept_duplicate_files = true"));
    assert!(exit_code != 0);
}

#[test]
fn test_repeated_filenames_index1() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads2.fq'
    index1 = 'sample_data/ten_reads2.fq'
    ",
    );
    assert!(stderr.contains("Repeated filename"));
    assert!(stderr.contains("ten_reads2.fq"));
    assert!(!stderr.contains("ten_reads.fq"));
    assert!(stderr.contains("options.accept_duplicate_files = true"));
    assert!(exit_code != 0);
}

#[test]
fn test_repeated_filenames_index2() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads2.fq'
    index1 = 'sample_data/ten_reads3.fq'
    index2 = 'sample_data/ten_reads.fq'
    ",
    );
    assert!(stderr.contains("Repeated filename"));
    assert!(stderr.contains("ten_reads.fq"));
    assert!(!stderr.contains("ten_reads2.fq"));
    assert!(!stderr.contains("ten_reads3.fq"));
    assert!(stderr.contains("options.accept_duplicate_files = true"));
    assert!(exit_code != 0);
}

#[test]
fn test_repeated_filenames_one_key() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = ['sample_data/ten_reads.fq', 'sample_data/ten_reads.fq']
    ",
    );
    assert!(stderr.contains("Repeated filename"));
    assert!(stderr.contains("options.accept_duplicate_files = true"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_empty_list() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = []
    ",
    );
    assert!(stderr.contains("No read1 files specified / empty list"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_len_neq_read2_len() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = ['sample_data/ten_reads.fq', 'sample_data/2.fq']
    read2 = ['sample_data/ten_reads_of_var_sizes.fq']
    ",
    );
    assert!(stderr.contains("Number of read2 files must be equal"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_len_neq_index1_len() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = ['sample_data/ten_reads.fq']
    read2 = ['sample_data/ten_reads_of_var_sizes.fq']
    index1 = ['sample_data/1.fq','sample_data/2.fq']
    ",
    );
    assert!(stderr.contains("Number of index1 files must be equal"));
    assert!(exit_code != 0);
}

#[test]
fn test_read1_len_neq_index2_len() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = ['sample_data/ten_reads.fq']
    read2 = ['sample_data/ten_reads_of_var_sizes.fq']
    index1 = ['sample_data/1.fq']
    index2 = ['sample_data/2.fq', 'sample_data/2.fqb']
    ",
    );
    assert!(stderr.contains("Number of index2 files must be equal"));
    assert!(exit_code != 0);
}

#[test]
fn test_index2_but_not_1() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = ['sample_data/ten_reads.fq']
    read2 = ['sample_data/ten_reads_of_var_sizes.fq']
    index2 = ['sample_data/2.fq']
    ",
    );
    assert!(stderr.contains("without index1 file(s)"));
    assert!(exit_code != 0);
}

#[test]
fn test_output_keep_index_but_no_index_files() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.fq'

[output]
    prefix = 'output'
    keep_index = true
    ",
    );
    assert!(stderr.contains("keep_index is set, but no index"));
    assert!(exit_code != 0);
}

#[test]
fn test_output_keep_index_but_no_index_files2() {
    let (_, _, stderr, exit_code) = run_and_capture_failure(
        "
[input]
    read1 = 'sample_data/ten_reads.r1.fq'
    index1 = 'sample_data/ten_reads.i1.fq'

[output]
    prefix = 'output'
    keep_index = true
    ",
    );
    assert!(stderr.contains("keep_index is set, but no index2"));
    assert!(exit_code != 0);
}

#[test]
#[should_panic(expected = "Invalid regex")]
fn test_validate_regex_fail() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'Rename'
    search = '['
    replace = ';'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Invalid DNA base: X")]
fn test_invalid_base() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'Prefix'
    seq = 'X'
    qual = 'B'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Invalid DNA base ('.' for 'any' is also allowed): X")]
fn test_invalid_base_or_dot() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'TrimPolyTail'
    base = 'x'
    min_length = 5
    max_mismatch_rate = 0.3
    max_consecutive_mismatches = 1

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Single DNA base or '.' only): was 'AA'")]
fn test_invalid_base_or_dot_too_long() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'TrimPolyTail'
    base = 'AA'
    min_length = 5
    max_mismatch_rate = 0.3
    max_consecutive_mismatches = 1

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "string should be exactly one")]
fn test_u8_from_char_too_many_chars() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'TrimQualityStart'
    target = 'read1'
    min = 'BB'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(
    expected = "invalid type: integer `300`, expected either a byte character or a number 0..255"
)]
fn test_u8_from_char_number_to_large() {
    //
    run("
[input]
    read1 = 'sample_data/test_phred.fq'
[[transform]]
    action = 'TrimQualityStart'
    target = 'read1'
    min = 300

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Seq and qual must be the same length")]
fn test_prefix_len_mismatch() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
    target = 'read1'
    action = 'Prefix'
    seq ='A'
    qual = 'BB'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Seq and qual must be the same length")]
fn test_postfix_len_mismatch() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
    target = 'read1'
    action = 'Postfix'
    seq ='A'
    qual = 'BB'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Max mismatches must be <= min length")]
fn test_adapter_mismatch_tail_too_many_mismatches() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
    action = 'TrimAdapterMismatchTail'
    query = 'agctc'
    target = 'read1'
    min_length = 5
    max_mismatches = 6

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Min length must be <= query length")]
fn test_adapter_mismatch_tail_min_length_too_high() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[transform]]
    action = 'TrimAdapterMismatchTail'
    query = 'agctc'
    target = 'read1'
    min_length = 6
    max_mismatches = 3

[output]
    prefix = 'output'
");
}
