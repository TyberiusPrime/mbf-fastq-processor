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
    action = 'TrimPolyN'
[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
anchor = 'Right'
    query = 'AAW'

[[step]]
    action = 'LowercaseTagSequence'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_lowercase.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}
#[test]
fn test_extract_filter() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    query = '^CT.'

[[step]]
    action = FilterTag
    keep_or_remove = 'Keep'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_filter.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}

#[test]
fn test_extract_trim() {
    //
    //
    for (direction, include) in [
        ("start", "true"),
        ("end", "true"),
        ("end", "false"),
        ("start", "false"),
    ] {
        let td = run(&format!(
            "
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'test'
    query = 'TCAA'

[[step]]
    action = TrimTag
    direction = '{direction}'
    include = {include}

[output]
    prefix = 'output'
"
        ));
        assert!(td.path().join("output_1.fq").exists());

        let should = std::fs::read_to_string(format!(
            "sample_data/ten_reads_test_extract_trim_{direction}_{incl}.fq",
            incl = if include == "true" {
                "include"
            } else {
                "exclude"
            }
        ))
        .unwrap();
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
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
#[should_panic(expected = "No Extract* generating label 'test'. Available: ['something', 'other']")]
fn test_filter_no_such_tag() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'something'
    query = '^CT.'

[[step]]
    action = 'ExtractIUPAC'
    label = 'other'
    query = '^CT.'

[[step]]
    action = FilterTag
    keep_or_remove = 'Keep'
    label = 'test'

[output]
    prefix = 'output'
");
}
