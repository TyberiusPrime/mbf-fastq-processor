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

[[transform]]
    action = 'Report'
    infix = 'start'
    json = true
    html = false


[[transform]]
    action = 'Head'
    n = 100

[[transform]]
    action = 'Report'
    infix = 'pre_multiplex'
    json = true
    html = false

[[transform]]
    action = 'Demultiplex'
    regions = [
        {source = 'read1', start=0, length=2},
    ]
    max_hamming_distance = 0
    output_unmatched = true

[transform.barcodes]
    CT = 'aaaa'
    TT = 'gggg'

[[transform]]
    action = 'Head'
    n = 10


[[transform]]
    action = 'Report' # max 10 output reads
    infix = 'post_multiplex'
    json = true
    html = false


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

    let output_files: Vec<_> = td.path().read_dir().unwrap().collect();
    dbg!(output_files);
    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output_start.json")).unwrap(),
    )
    .unwrap();
    let rc: u64 = v["read_count"].as_number().unwrap().as_u64().unwrap();
    assert!(rc >= 100u64);

    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output_pre_multiplex.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["read_count"], 100);

    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output_post_multiplex_aaaa.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["read_count"], 2);

    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output_post_multiplex_gggg.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["read_count"], 1);

    let v = serde_json::from_str::<serde_json::Value>(
        &ex::fs::read_to_string(td.path().join("output_post_multiplex_no-barcode.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["read_count"], 10 - 2 - 1);
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

[transform.barcodes]
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

[transform.barcodes]
    ATGA = 'aaaa'
    CTCC = 'gggg'
");

    assert!(td.path().join("output_aaaa_1.fq").exists());
    assert!(td.path().join("output_gggg_1.fq").exists());
    //confirm there are no other .fq in td
    let fqs_found = td
        .path()
        .read_dir()
        .unwrap()
        .filter(|x| x.as_ref().unwrap().path().extension().unwrap() == "fq")
        .count();
    assert_eq!(fqs_found, 3);
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

    //let lines_no_barcode = std::fs::read_to_string("output_no_barcode.fq").unwrap().lines().count();
    assert!(lines_barcode1 == 1 * 4);
    assert!(lines_barcode2 == 1 * 4);
    assert!(lines_no_barcode == 8 * 4);
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

[transform.barcodes]
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

[transform.barcodes]
    CT = 'aaaa'

[[transform]] # to trigger iter_tags
    action = 'Report'
    infix = 'report'
    json = true
    html = false


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
}
