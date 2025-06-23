mod common;
use ::function_name::named;
use std::path::PathBuf;

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
    search = 'CTN'
    target = 'Read1'

[[step]]
    action = 'StoreTagInComment'
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
#[named]
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
    search = 'AAW'

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
   assert_equal_or_dump(function_name!(), &actual, &should);
}

fn assert_equal_or_dump(func_name: &str, actual: &str, should: &str) {
    if actual != should {
        let common_path = PathBuf::from("tests/failures").join(func_name);
        std::fs::create_dir_all(&common_path).unwrap();
        let actual_path = (&common_path).join("actual.fq");
        let should_path = (&common_path).join("should.fq");
        std::fs::write(&actual_path, actual).unwrap();
        std::fs::write(&should_path, should).unwrap();
        panic!(
            "Output does not match expected. Actual written to 
    {:?}, 
expected written to 
    {:?}",
            actual_path, should_path
        );
    }
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
    search = 'TGTC'
    anchor ='Anywhere'

[[step]]
    action = 'FilterByTag'
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
    search = 'CTN'
    anchor ='Left'

[[step]]
    action = 'FilterByTag'
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
    search = 'TCAA'

[[step]]
    action = 'TrimAtTag'
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
    search = 'CTN'
    anchor = 'Left'

[[step]]
    action = 'ExtractIUPAC'
    target = 'Read1'
    label = 'test'
    anchor = 'Left'
    search = 'GCN'
");
}

#[test]
#[should_panic(expected = "No Extract* generating label 'test' (or removed previously). Available at this point: {\"")]
fn test_filter_no_such_tag() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'something'
    search = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'ExtractIUPAC'
    label = 'other'
    search = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'FilterByTag'
    keep_or_remove = 'Keep'
    label = 'test'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "No Extract* generating label 'nonexistent_tag' (or removed previously). Available at this point: {\"")]
fn test_remove_nonexistent_tag() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'real_tag'
    search = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'RemoveTag'
    label = 'nonexistent_tag'

[output]
    prefix = 'output'
");
}

#[test]
#[should_panic(expected = "No Extract* generating label 'removed_tag' (or removed previously). Available at this point: {}")]
fn test_use_removed_tag() {
    //
    run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractIUPAC'
    label = 'removed_tag'
    search = 'CTN'
    target = 'Read1'
    anchor ='Left'

[[step]]
    action = 'RemoveTag'
    label = 'removed_tag'

[[step]]
    action = 'StoreTagInComment'
    label = 'removed_tag'

[output]
    prefix = 'output'
");
}

#[test]
fn test_extract_regex() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'ExtractRegex'
    label = 'test'
    search = 'CT(..)CT'
    target = 'Read1'
    replacement = '$1'

[[step]]
    action = 'FilterByTag'
    label = 'test'
    keep_or_remove = 'Keep'


[[step]]
    action = 'StoreTagInComment'
    label = 'test'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let should = std::fs::read_to_string("sample_data/ten_reads_test_extract_regexs.fq").unwrap();
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(should, actual);
}


#[test]
fn test_umi_extract() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'Head'
    n = 2

[[step]]
    action = 'ExtractRegion'
    label='umi'
    regions = [{source = 'Read1', start = 1, length = 5}]

[[step]]
    action = 'StoreTagInComment'
    label = 'umi'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1 umi=TCCTG
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2 umi=GCGAT
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
";
    assert_eq!(should, actual);
}

#[test]
fn test_umi_extract_store_in_all_read_names() {
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'
    read2 = 'sample_data/ten_reads_var_n.fq'
    index1 = 'sample_data/ten_reads_var_n.fq'
    index2 = 'sample_data/ten_reads.fq'

[options]
    accept_duplicate_files = true


[[step]]
    action = 'Head'
    n = 2

[[step]]
    action = 'ExtractRegion'
    label = 'UMI'
    regions = [{source = 'Read1', start = 1, length = 5}]

[[step]]
    action = 'StoreTagInComment'
    label = 'UMI'
    target = 'All'

[output]
    prefix = 'output'
    keep_index = true
");
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@Read1 UMI=TCCTG
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2 UMI=GCGAT
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
";
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    let should = "@Read1N UMI=TCCTG
NTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2N UMI=GCGAT
NGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
";
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_i2.fq")).unwrap();
    let should = "@Read1 UMI=TCCTG
CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2 UMI=GCGAT
GGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
";
    assert_eq!(should, actual);

    let actual = std::fs::read_to_string(td.path().join("output_i1.fq")).unwrap();
    let should = "@Read1N UMI=TCCTG
NTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCDCCCCCCCCCC?A???###############################
@Read2N UMI=GCGAT
NGCGATTTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCBCBCCCCCBCCDC?CAC=#@@A@##########################
";
    assert_eq!(should, actual);
}

#[test]
#[named]
fn test_umi_extract_with_existing_comment() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'


[[step]]
    action = 'Head'
    n = 2

[[step]]
    action = 'ExtractRegion'
    label = 'UMI'
    regions = [{source = 'Read1', start = 0, length = 6}]

[[step]]
    action = 'TrimAtTag'
    label = 'UMI'
    direction = 'Start'
    keep_tag = false

[[step]]
    action = 'StoreTagInComment'
    label = 'UMI'
    target = 'All'

[output]
    prefix = 'output'
");
    assert!(td.path().join("output_1.fq").exists());
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    let should = "@ERR664392.1 GAII02_0001:7:1:1116:18963#0/1 UMI=CTCCTG
CACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCCCCCC?A???###############################
@ERR664392.2 GAII02_0001:7:1:1116:17204#0/1 UMI=GGCGAT
TTCAATGTCCAAGGNCAGTTTNNNNNNNNNNNNNNNNNNNNNNNN
+
CCCCBCCDC?CAC=#@@A@##########################
";
    assert_equal_or_dump(function_name!(), &actual, &should);
}

#[test]
#[named]
fn test_store_tags_in_tsv() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'Head'
    n = 4

[[step]]
    action = 'ExtractIUPAC'
    label = 'barcode1'
    search = 'CTC'
    target = 'Read1'
    anchor = 'Left'

[[step]]
    action = 'ExtractIUPAC'
    label = 'barcode2'
    search = 'CAA'
    target = 'Read1'
    anchor = 'Anywhere'

[[step]]
    action = 'StoreTagsInTable'
    table_filename = 'tags.tsv'
    format = 'TSV'

[output]
    prefix = 'output'
");
    assert!(td.path().join("tags.tsv").exists());
    let contents = std::fs::read_to_string(td.path().join("tags.tsv")).unwrap();
    
    // Check that the TSV has the expected format and data
    let lines: Vec<&str> = contents.lines().collect();
    assert!(lines.len() >= 5); // Header + at least 4 reads
    assert_eq!(lines[0], "ReadName\tbarcode1\tbarcode2"); // Header row

    // Check that at least some reads have the expected tags
    assert!(contents.contains("Read1"));
    assert!(contents.contains("CTC"));
    assert!(contents.contains("CAA"));
}

#[test]
#[named]
fn test_store_tags_in_json() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[[step]]
    action = 'Head'
    n = 4

[[step]]
    action = 'ExtractRegex'
    label = 'motif1'
    search = 'C(T.)C'
    target = 'Read1'
    replacement = '$1'

[[step]]
    action = 'ExtractRegion'
    label = 'motif2'
    regions = [{source = 'Read1', start = 5, length = 3}]

[[step]]
    action = 'StoreTagsInTable'
    table_filename = 'tags.json'
    format = 'JSON'

[output]
    prefix = 'output'
");
    assert!(td.path().join("tags.json").exists());
    let contents = std::fs::read_to_string(td.path().join("tags.json")).unwrap();
    
    // Check that JSON is valid and contains the expected data
    let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert!(json.is_object());
    
    // Check that ReadName and both tags exist in the JSON
    assert!(json.get("ReadName").is_some());
    assert!(json.get("motif1").is_some());
    assert!(json.get("motif2").is_some());
    
    // Verify array lengths match the number of reads processed (4)
    assert_eq!(json["ReadName"].as_array().unwrap().len(), 4);
    assert_eq!(json["motif1"].as_array().unwrap().len(), 4);
    assert_eq!(json["motif2"].as_array().unwrap().len(), 4);
}

#[test]
#[should_panic(expected = "ExtractRegion and TrimAtTag only work together on single-entry regions.")]
fn test_extract_region_trim_at_tag_conflict() {
    //
    run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'
    read2 = 'sample_data/ERR664392_1250.fq.gz'

[options]
    accept_duplicate_files = true


[[step]]
    action = 'Head'
    n = 2

[[step]]
    action = 'ExtractRegion'
    label = 'UMI'
    regions = [{source = 'Read1', start = 0, length = 6},{source = 'Read2', start = 0, length = 6}]

[[step]]
    action = 'TrimAtTag'
    label = 'UMI'
    direction = 'Start'
    keep_tag = false

[output]
    prefix = 'output'
");

}
