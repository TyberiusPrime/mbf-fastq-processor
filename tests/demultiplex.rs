#![allow(clippy::identity_op)]
mod common;
use common::*;

#[allow(clippy::identity_op)]
#[test]
fn test_simple_demultiplex_basics() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'
    report_json=true

[[transform]]
    action = 'Report'
    label = 'start'


[[transform]]
    action = 'Head'
    n = 100

[[transform]]
    action = 'Report'
    label = 'pre_multiplex'

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = true

[transform.barcode_to_name]
    CT = 'aaaa'
    TT = 'gggg'

[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Report' # max 10 output reads
    label = 'post_multiplex'

");

    assert!(!td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_aaaa_1.fq").exists());
    assert!(td.path().join("output_gggg_1.fq").exists());
    assert!(td.path().join("output_no-barcode_1.fq").exists());
    let lines_barcode1 = ex::fs::read_to_string(td.path().join("output_aaaa_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_barcode2 = ex::fs::read_to_string(td.path().join("output_gggg_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_no_barcode = ex::fs::read_to_string(td.path().join("output_no-barcode_1.fq"))
        .unwrap()
        .lines()
        .count();
    assert!(lines_barcode1 + lines_barcode2 + lines_no_barcode == 10 * 4);
    assert!(lines_barcode1 == 2 * 4);
    assert!(lines_barcode2 == 1 * 4); //double check this, number might be wrong
    assert!(lines_no_barcode == (10 - 2 - 1) * 4);

    // let output_files: Vec<_> = td.path().read_dir().unwrap().collect();
    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    dbg!(&v);
    let rc: u64 = v["start"]["molecule_count"]
        .as_number()
        .unwrap()
        .as_u64()
        .unwrap();
    assert!(rc >= 100u64);

    assert_eq!(v["pre_multiplex"]["molecule_count"], 100);

    assert_eq!(v["post_multiplex"]["aaaa"]["molecule_count"], 2);

    assert_eq!(v["post_multiplex"]["gggg"]["molecule_count"], 1);

    assert_eq!(
        v["post_multiplex"]["no-barcode"]["molecule_count"],
        10 - 2 - 1
    );
}

#[test]
fn test_simple_demultiplex_no_unmatched() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'


[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'aaaa'
    TT = 'gggg'
");

    assert!(!td.path().join("output_1.fq").exists());
    assert!(td.path().join("output_aaaa_1.fq").exists());
    assert!(td.path().join("output_gggg_1.fq").exists());
    assert!(!td.path().join("output_no-barcode_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert!(fqs_found == 2);
    let lines_barcode1 = ex::fs::read_to_string(td.path().join("output_aaaa_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_barcode2 = ex::fs::read_to_string(td.path().join("output_gggg_1.fq"))
        .unwrap()
        .lines()
        .count();
    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    dbg!(&lines_barcode1);
    dbg!(&lines_barcode2);
    assert_eq!(lines_barcode1, 2 * 4);
    assert_eq!(lines_barcode2, 1 * 4);
    assert!(lines_barcode1 + lines_barcode2 == (2 + 1) * 4); //that's wrong.
    //assert!(lines_no_barcode == 4*4);
}

#[test]
fn test_simple_demultiplex_hamming() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'


[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=4},
    ]
    max_hamming_distance = 1
    output_unmatched = true

[transform.barcode_to_name]
    ATGA = 'label1'
    CTCC = 'label2'
");

    assert!(td.path().join("output_label1_1.fq").exists());
    assert!(td.path().join("output_label2_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 3);
    let lines_barcode1 = ex::fs::read_to_string(td.path().join("output_label1_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_barcode2 = ex::fs::read_to_string(td.path().join("output_label2_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_no_barcode = ex::fs::read_to_string(td.path().join("output_no-barcode_1.fq"))
        .unwrap()
        .lines()
        .count();

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    assert!(lines_barcode1 == 1 * 4);
    assert!(lines_barcode2 == 1 * 4);
    assert!(lines_no_barcode == 8 * 4);
}

#[test]
fn test_simple_demultiplex_iupac() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'


[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=4},
    ]
    max_hamming_distance = 0
    output_unmatched = true

[transform.barcode_to_name]
    GNAA = 'label1'
    CTNN = 'label2'
");

    assert!(td.path().join("output_label1_1.fq").exists());
    assert!(td.path().join("output_label2_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 3);
    let lines_barcode1 = ex::fs::read_to_string(td.path().join("output_label1_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_barcode2 = ex::fs::read_to_string(td.path().join("output_label2_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_no_barcode = ex::fs::read_to_string(td.path().join("output_no-barcode_1.fq"))
        .unwrap()
        .lines()
        .count();

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    assert_eq!(lines_barcode1, 1 * 4);
    assert_eq!(lines_barcode2, 2 * 4);
    assert_eq!(lines_no_barcode, 7 * 4);
}
#[test]
fn test_simple_demultiplex_iupac_hamming() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'


[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=4},
    ]
    max_hamming_distance = 1
    output_unmatched = true

[transform.barcode_to_name]
    GNAA = 'label1'
    CTNN = 'label2'
");

    assert!(td.path().join("output_label1_1.fq").exists());
    assert!(td.path().join("output_label2_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 3);
    let lines_barcode1 = ex::fs::read_to_string(td.path().join("output_label1_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_barcode2 = ex::fs::read_to_string(td.path().join("output_label2_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_no_barcode = ex::fs::read_to_string(td.path().join("output_no-barcode_1.fq"))
        .unwrap()
        .lines()
        .count();

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    assert_eq!(lines_barcode1, 1 * 4);
    assert_eq!(lines_barcode2, 6 * 4);
    assert_eq!(lines_no_barcode, 3 * 4);
}

#[test]
fn test_simple_demultiplex_single_barcode() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'

[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 1
    output_unmatched = true

[transform.barcode_to_name]
    CT = 'aaaa'
");
    let files_found: Vec<_> = td.path().read_dir().unwrap().collect();
    dbg!(files_found);

    assert!(td.path().join("output_aaaa_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 2);
    let lines_barcode1 = std::fs::read_to_string(td.path().join("output_aaaa_1.fq"))
        .unwrap()
        .lines()
        .count();
    let lines_no_barcode = ex::fs::read_to_string(td.path().join("output_no-barcode_1.fq"))
        .unwrap()
        .lines()
        .count();

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    dbg!(lines_barcode1);
    dbg!(lines_no_barcode);
    assert!(lines_barcode1 == 6 * 4);
    assert!(lines_no_barcode == 4 * 4);
}

#[test]
fn test_simple_demultiplex_single_barcode_no_unmatched_output() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR664392_1250.fq.gz'

[output]
    prefix = 'output'
    format = 'Raw'
    report_json = true

[[transform]]
    action = 'Head'
    n = 10

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 1
    output_unmatched = false

[transform.barcode_to_name]
    CT = 'aaaa'

[[transform]] # to trigger iter_tags
    action = 'Report'
    label = 'report'


");
    let files_found: Vec<_> = td.path().read_dir().unwrap().collect();
    dbg!(files_found);

    assert!(td.path().join("output_aaaa_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 1);
    let lines_barcode1 = std::fs::read_to_string(td.path().join("output_aaaa_1.fq"))
        .unwrap()
        .lines()
        .count();
    assert!(!td.path().join("output_no-barcode_1.fq").exists());

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    assert!(lines_barcode1 == 6 * 4);

    let report = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    assert!(!report["report"]["aaaa"].is_null());
    assert!(report["report"]["no-barcode"].is_null());
}
