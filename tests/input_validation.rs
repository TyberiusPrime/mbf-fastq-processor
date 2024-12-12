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
    read1 = 'sample_data/broken.fq' # ! instead of @ after 250 reads.

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Unexpected symbol where @ was expected in input.")]
fn test_broken_newline() {
    run("
[input]
    read1 = 'sample_data/ten_reads_broken_newline.fq'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "Parsing failure, two newlines in sequence")]
fn test_broken_newline2() {
    run("
[input]
    read1 = 'sample_data/ten_reads_broken_newline2.fq'

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
#[should_panic(expected = "Report output infixes must be distinct. Duplicated: 'xyz'")]
fn test_report_infixes_are_distinct() {
    let _td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'


[[transform]]
    action = 'Report'
    infix = 'xyz'
    json = true
    html = false

[[transform]]
    action = 'Report'
    infix = 'xyz'
    json = true
    html = false


[output]
    prefix = 'output'
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

[transform.barcodes]
    CT = 'gggg'
    TT = 'gggg'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=2, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcodes]
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

[transform.barcodes]
    CT = 'gggg'
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

[transform.barcodes]
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
