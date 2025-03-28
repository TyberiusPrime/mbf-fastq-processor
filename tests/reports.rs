#![allow(clippy::identity_op)]
mod common;
use common::*;

#[test]
fn test_report() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'


[[step]]
    action = 'Report'
    label = 'xyz'
    count = true
    base_statistics = true
    duplicate_count_per_read = true
    length_distribution = true

[output]
    prefix = 'output'
    report_json=true

");
    //list all files in td.path()
    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output.json").exists());
    let v = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["xyz"]["molecule_count"], 10);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["total_bases"], 510);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["q20_bases"], 234);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["q30_bases"], 223);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["gc_bases"], 49 + 68);

    let should_a = vec![
        1, 0, 1, 2, 5, 3, 2, 2, 2, 3, 1, 1, 2, 3, 2, 0, 3, 4, 3, 4, 0, 1, 4, 1, 4, 4, 2, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let should_c = vec![
        3, 1, 3, 2, 1, 1, 1, 1, 6, 0, 2, 3, 2, 0, 2, 5, 1, 1, 3, 1, 0, 3, 1, 3, 2, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_g = vec![
        5, 4, 4, 3, 2, 3, 2, 5, 2, 0, 3, 1, 3, 0, 4, 3, 3, 2, 2, 3, 0, 3, 1, 4, 1, 2, 3, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_t = vec![
        1, 5, 2, 3, 2, 3, 5, 2, 0, 7, 4, 5, 3, 7, 2, 2, 3, 3, 2, 2, 0, 3, 4, 2, 3, 3, 5, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_n = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 10, 10,
        10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    ];
    //gc is trivial to calculate...
    /* let should_gc = vec![
        8, 5, 7, 5, 3, 4, 3, 6, 8, 0, 5, 4, 5, 0, 6, 8, 4, 3, 5, 4, 0, 6, 2, 7, 3, 3, 3, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]; */

    assert_eq!(
        v["xyz"]["read1"]["length_distribution"]
            .as_array()
            .unwrap()
            .len(),
        52 //0 is a possible read length.
    );
    assert_eq!(
        v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["a"]
            .as_array()
            .unwrap()
            .len(),
        51
    );
    for (ii, sa) in should_a.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["a"][ii],
            *sa
        );
    }
    for (ii, sa) in should_c.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["c"][ii],
            *sa
        );
    }
    for (ii, sa) in should_g.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["g"][ii],
            *sa
        );
    }
    for (ii, sa) in should_t.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["t"][ii],
            *sa
        );
    }
    for (ii, sa) in should_n.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["n"][ii],
            *sa
        );
    }
    /* for (ii, sa) in should_gc.iter().enumerate() {
        assert_eq!(v["read1"]["per_position_counts"]["gc"][ii], *sa);
    }
    */
    assert_eq!(v["xyz"]["read1"]["length_distribution"][0], 0);
    assert_eq!(v["xyz"]["read1"]["length_distribution"][51], 10);
    assert_eq!(v["xyz"]["read1"]["duplicate_count"], 0);
}

#[test]
fn test_report_no_output() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads.fq'

[output]
    format = 'None'
    prefix = 'output' # still needed to name the report!
    report_json = true

[[step]]
    action = 'Report'
    label = 'xyz'
    count = true
    base_statistics = true
    length_distribution = true
    duplicate_count_per_read = true


");
    //list all files in td.path()
    assert!(!td.path().join("output_1.fq").exists());
    assert!(td.path().join("output.json").exists());
    let v = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    dbg!(&v);
    assert_eq!(v["xyz"]["molecule_count"], 10);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["total_bases"], 510);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["q20_bases"], 234);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["q30_bases"], 223);
    assert_eq!(v["xyz"]["read1"]["base_statistics"]["gc_bases"], 49 + 68);

    let should_a = vec![
        1, 0, 1, 2, 5, 3, 2, 2, 2, 3, 1, 1, 2, 3, 2, 0, 3, 4, 3, 4, 0, 1, 4, 1, 4, 4, 2, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let should_c = vec![
        3, 1, 3, 2, 1, 1, 1, 1, 6, 0, 2, 3, 2, 0, 2, 5, 1, 1, 3, 1, 0, 3, 1, 3, 2, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_g = vec![
        5, 4, 4, 3, 2, 3, 2, 5, 2, 0, 3, 1, 3, 0, 4, 3, 3, 2, 2, 3, 0, 3, 1, 4, 1, 2, 3, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_t = vec![
        1, 5, 2, 3, 2, 3, 5, 2, 0, 7, 4, 5, 3, 7, 2, 2, 3, 3, 2, 2, 0, 3, 4, 2, 3, 3, 5, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let should_n = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 10, 10,
        10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
    ];
    for (ii, sa) in should_a.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["a"][ii],
            *sa
        );
    }
    for (ii, sa) in should_c.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["c"][ii],
            *sa
        );
    }
    for (ii, sa) in should_g.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["g"][ii],
            *sa
        );
    }
    for (ii, sa) in should_t.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["t"][ii],
            *sa
        );
    }
    for (ii, sa) in should_n.iter().enumerate() {
        assert_eq!(
            v["xyz"]["read1"]["base_statistics"]["per_position_counts"]["n"][ii],
            *sa
        );
    }
    /* for (ii, sa) in should_gc.iter().enumerate() {
        assert_eq!(v["xyz"]["read1"]["per_position_counts"]["gc"][ii], *sa);
    }
    */
    assert_eq!(v["xyz"]["read1"]["length_distribution"][0], 0);
    assert_eq!(v["xyz"]["read1"]["length_distribution"][51], 10);
    assert_eq!(v["xyz"]["read1"]["duplicate_count"], 0);
}

#[test]
fn test_duplication_count_is_stable() {
    // we had some issues with the duplicate_counts changing between runs
    // let's fix that.
    let config = "
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'


[[step]]
    action = 'Report'
    label = 'xyz'
    duplicate_count_per_read = true
    debug_reproducibility=true

[output]
    prefix = 'output'
    report_json = true

";
    let mut seen = std::collections::HashSet::new();
    for _ in 0..10 {
        let td = run(config);
        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        let first = v["xyz"]["read1"]["duplicate_count"].as_u64().unwrap();
        seen.insert(first);
    }
    assert_eq!(1, seen.len());
}

#[test]
#[allow(clippy::many_single_char_names)]
fn test_report_pe() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'


[[step]]
    action = 'Report'
    label = 'xyz'
    duplicate_count_per_read = true
    length_distribution = true
    base_statistics = true

[output]
    prefix = 'output'
    report_json = true

");
    assert!(td.path().join("output_1.fq").exists());
    assert!(td.path().join("output.json").exists());
    let vv = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    dbg!(&vv);
    assert_eq!(vv["xyz"]["molecule_count"], 10000);
    assert_eq!(vv["xyz"]["read1"]["length_distribution"][150], 10000);
    assert_eq!(vv["xyz"]["read1"]["duplicate_count"], 787);
    assert_eq!(
        vv["xyz"]["read1"]["base_statistics"]["total_bases"],
        10000 * 150
    );
    for ii in 0..150 {
        let a: u64 = vv["xyz"]["read1"]["base_statistics"]["per_position_counts"]["a"][ii]
            .as_u64()
            .unwrap();
        let c: u64 = vv["xyz"]["read1"]["base_statistics"]["per_position_counts"]["c"][ii]
            .as_u64()
            .unwrap();
        let g: u64 = vv["xyz"]["read1"]["base_statistics"]["per_position_counts"]["g"][ii]
            .as_u64()
            .unwrap();
        let t: u64 = vv["xyz"]["read1"]["base_statistics"]["per_position_counts"]["t"][ii]
            .as_u64()
            .unwrap();
        let n: u64 = vv["xyz"]["read1"]["base_statistics"]["per_position_counts"]["n"][ii]
            .as_u64()
            .unwrap();
        assert_eq!(a + c + g + t + n, 10000);
    }

    assert_eq!(vv["xyz"]["read2"]["duplicate_count"], 769);
    assert_eq!(vv["xyz"]["read2"]["length_distribution"][150], 10000);
    assert_eq!(
        vv["xyz"]["read2"]["base_statistics"]["total_bases"],
        10000 * 150
    );
    for ii in 0..150 {
        let a: u64 = vv["xyz"]["read2"]["base_statistics"]["per_position_counts"]["a"][ii]
            .as_u64()
            .unwrap();
        let c: u64 = vv["xyz"]["read2"]["base_statistics"]["per_position_counts"]["c"][ii]
            .as_u64()
            .unwrap();
        let g: u64 = vv["xyz"]["read2"]["base_statistics"]["per_position_counts"]["g"][ii]
            .as_u64()
            .unwrap();
        let t: u64 = vv["xyz"]["read2"]["base_statistics"]["per_position_counts"]["t"][ii]
            .as_u64()
            .unwrap();
        let n: u64 = vv["xyz"]["read2"]["base_statistics"]["per_position_counts"]["n"][ii]
            .as_u64()
            .unwrap();
        assert_eq!(a + c + g + t + n, 10000);
    }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn test_read_length_reporting() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ten_reads_of_var_sizes.fq'

[[step]]
    action = 'Report'
    label = 'report'
    count = false
    length_distribution = true

[output]
    prefix = 'output'
    report_json = true
");
    assert!(td.path().join("output.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output.json")).unwrap();
    let parsed = serde_json::from_str::<serde_json::Value>(&actual).unwrap();
    dbg!(&parsed);
    assert!(parsed["report"]["molecule_count"].is_null());
    let read1_length_distribution = parsed["report"]["read1"]["length_distribution"]
        .as_array()
        .unwrap();
    let no_length_distri: Vec<usize> = read1_length_distribution
        .iter()
        .map(|x| x.as_number().unwrap().as_u64().unwrap() as usize)
        .collect();
    assert_eq!(no_length_distri, [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
}
