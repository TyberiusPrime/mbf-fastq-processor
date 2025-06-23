mod common;
use common::*;

#[test]
fn test_validate_seq() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'ValidateSeq'
    allowed = 'CGATN'
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_validate_phred() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
[[step]]
    action = 'ValidatePhred'
    target = 'Read1'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}