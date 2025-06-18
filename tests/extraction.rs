mod common;
use common::*;

#[test]
fn test_extract_tag() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    anchor = 'Left'
    query = 'CTN'
    target = 'Read1'

[[step]]
    action = 'TagSequenceToName'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_tag.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_extract_highlight() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'TrimPolyTail'
    target = 'Read1'
    base = 'N'
    min_length=1
    max_mismatch_rate=0
    max_consecutive_mismatches=0

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    anchor = 'Right'
    target = 'Read1'
    query = 'AAW'

[[step]]
    action = 'LowercaseTag'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should =
        std::fs::read_to_string("sample_data/ten_reads_test_extract_lowercase.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}
#[test]
fn test_extract_filter_keep() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    target = 'Read1'
    query = 'TGTC'
    anchor ='Anywhere'

[[step]]
    action = 'FilterTag'
    keep_or_remove = 'Keep'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_filter_keep.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_extract_filter_remove() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    target = 'Read1'
    query = 'CTN'
    anchor ='Left'

[[step]]
    action = 'FilterTag'
    keep_or_remove = 'Remove'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_filter_remove.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_extract_trim() {
    //
    //
    for (direction, include) in [
        ("Start", "false"),
        ("Start", "true"),
        ("End", "true"),
        ("End", "false"),
    ] {
        let td = run(&format!(
            "
[input]
    read1 = 'sample_data/ten_reads_of_var_sizes.fq'
    read2 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    anchor = 'Anywhere'
    label = 'test'
    target = 'Read2'
    query = 'TCAA'

[[step]]
    action = 'TrimTag'
    label = 'test'
    direction = '{direction}'
    keep_tag = {include}

[output]
    prefix = 'output'
"
        ));
        assert!(td.path().join("output_2.fq").exists());

        let should = std::fs::read_to_string(format!(
            "sample_data/ten_reads_test_extract_trim_{direction}_{incl}.fq",
            incl = if include == "true" {
                "include"
            } else {
                "exclude"
            }
        ))
        .unwrap();
        let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
        assert_eq!(should, actual);
    }
}

#[test]
#[should_panic(expected = "Duplicate extract label")]
fn test_extract_tag_duplicate_name_panics() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    target = 'Read1'
    query = 'CTN'
    anchor = 'Left'

[[step]]
    action = 'ExtractIUPAC'
    target = 'Read1'
    label = 'test'
    anchor = 'Left'
    query = 'GCN'
");
}

#[test]
#[should_panic(expected = "No Extract* generating label 'test'. Available at this point: {\"")]
fn test_filter_no_such_tag() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'something'
    query = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'ExtractIUPAC'
    label = 'other'
    query = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'FilterTag'
    keep_or_remove = 'Keep'
    label = 'test'

[output]
    prefix = 'output'
");
}
