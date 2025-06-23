/// Head should early terminate, if possible, by telling the upstream
/// threads to stop sending.
/// But: If one of the upstream threads is supposed to see all data nevertheless
/// (for example: reports), we must not terminate prematurely.
///
/// These tests reflect this.
///
mod common;
use common::*;

#[test]
fn test_head_stops_reading() {
    //todo: replace with sensor node
    //this test is maybe slightly timing sensitive?
    //we use a broken fastq for clever checking that head actually terminated here.
    let td = run("
[input]
    read1 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.

[options]
    buffer_size = 100
    block_size = 5

[[step]]
action = 'Head'
n = 128

[output]
    prefix = 'output'
");
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(actual.chars().filter(|x| *x == '\n').count() == 128 * 4);
}

#[test]
fn test_head_stops_reading_multiple() {
    //todo: replace with sensor node
    //this test is maybe slightly timing sensitive?
    //we use a broken fastq for clever checking that head actually terminated here.
    let td = run("
[input]
    read1 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.
    read2 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.
    index1 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.
    index2 = 'sample_data/broken/no_at_after_250_reads.fq' # ! instead of @ after 250 reads.

[options]
    buffer_size = 100
    block_size = 5
    accept_duplicate_files = true

[[step]]
action = 'Head'
n = 128

[output]
    prefix = 'output'
    keep_index = true
");
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert!(actual.chars().filter(|x| *x == '\n').count() == 128 * 4);
    let actual = std::fs::read_to_string(td.path().join("output_2.fq")).unwrap();
    assert!(actual.chars().filter(|x| *x == '\n').count() == 128 * 4);
    let actual = std::fs::read_to_string(td.path().join("output_i1.fq")).unwrap();
    assert!(actual.chars().filter(|x| *x == '\n').count() == 128 * 4);
    let actual = std::fs::read_to_string(td.path().join("output_i2.fq")).unwrap();
    assert!(actual.chars().filter(|x| *x == '\n').count() == 128 * 4);
}

#[test]
/// We used to 'shut down' the input when a head was 'full',
/// but we must not do that if a Report/Quantify/Inspect was before
fn test_head_after_quantify() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
[options]
    block_size = 15

[[step]]
    action = 'ExtractRegion'
    label = 'kmer'

    regions = [
            { source = 'Read1', start = 6, length = 6},
            { source = 'Read2', start = 10, length = 7}
    ]
[[step]]
    action = 'QuantifyTag'
    infix = 'kmer'
    label = 'kmer'

[[step]]
    action ='Head'
    n = 10

[output]
    prefix = 'output'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10);

    //check quantify

    assert!(td.path().join("output_kmer.qr.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output_kmer.qr.json")).unwrap();
    let should = std::fs::read_to_string("sample_data/ERR12828869_10k_1.quantify.json").unwrap();

    let json_actual: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&actual).unwrap();
    let json_should: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&should).unwrap();
    assert_eq!(json_actual, json_should);
}

#[test]
fn test_head_after_report() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
[options]
    block_size = 15

[[step]]
    action = 'Report'
    label = 'report' # Key that the report will be listed under. Must be distinct
    count = true
    base_statistics = false
    length_distribution = false
    duplicate_count_per_read = false
    duplicate_count_per_fragment = false

[[step]]
    action ='Head'
    n = 10

[output]
    prefix = 'output'
    report_json = true

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10);

    //check quantify

    let v = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    dbg!(&v);
    assert_eq!(v["report"]["molecule_count"], 10000);
}

#[test]
/// We used to 'shut down' the input when a head was 'full',
/// but we must not do that if a Report/Quantify/Inspect was before
fn test_head_before_quantify() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
[options]
    block_size = 15

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = 'ExtractRegion'
    label = 'kmer'

    regions = [
            { source = 'Read1', start = 6, length = 6},
    ]
[[step]]
    action = 'QuantifyTag'
    infix = 'kmer'
    label = 'kmer'




[output]
    prefix = 'output'

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10);

    //check quantify

    assert!(td.path().join("output_kmer.qr.json").exists());
    let actual = std::fs::read_to_string(td.path().join("output_kmer.qr.json")).unwrap();
    let should =
        std::fs::read_to_string("sample_data/ERR12828869_10k_head_10.quantify.json").unwrap();

    let json_actual: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&actual).unwrap();
    let json_should: std::collections::HashMap<String, usize> =
        serde_json::from_str::<_>(&should).unwrap();
    assert_eq!(json_actual, json_should);
}

#[test]
fn test_head_before_report() {
    //
    let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
    read2 = 'sample_data/ERR12828869_10k_2.fq.zst'
[options]
    block_size = 15

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = 'Report'
    label = 'report' # Key that the report will be listed under. Must be distinct
    count = true
    base_statistics = false
    length_distribution = false
    duplicate_count_per_read = false
    duplicate_count_per_fragment = false


[output]
    prefix = 'output'
    report_json = true

");

    //check head
    let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
    assert_eq!(actual.lines().count() / 4, 10);

    //check quantify

    let v = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
    )
    .unwrap();
    dbg!(&v);
    assert_eq!(v["report"]["molecule_count"], 10);
}

#[test]
fn test_multi_stage_head() {
    {
        //
        let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
[options]
    block_size = 15

[[step]]
    action = '_InternalReadCount'
    label = 'top'

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = '_InternalReadCount'
    label = 'middle'

[[step]]
    action ='Head'
    n = 1

[[step]]
    action = '_InternalReadCount'
    label = 'bottom'

[output]
    prefix = 'output'
    report_json = true

");

        //check head
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
        assert_eq!(actual.lines().count() / 4, 1);

        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        dbg!(&v);
        assert!(v["top"]["_InternalReadCount"].as_i64().unwrap() <= 90); //we don't terminate it after exactly 10
        //and depending on the timing we might have some more blocks in there,
        //but surley much less than the 10k we could get...
                                                                         //reads, but after the next block or so
        assert_eq!(v["middle"]["_InternalReadCount"], 10);
        assert_eq!(v["bottom"]["_InternalReadCount"], 1);
    }
}
#[test]
fn test_multi_stage_head_report_top() {
    {
        //
        let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
[options]
    block_size = 15

[[step]]
    action = '_InternalReadCount'
    label = 'top'

[[step]]
    action = 'Report'
    label = 'report_top'
    count = true

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = '_InternalReadCount'
    label = 'middle'

[[step]]
    action ='Head'
    n = 1

[[step]]
    action = '_InternalReadCount'
    label = 'bottom'

[output]
    prefix = 'output'
    report_json = true

");

        //check head
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
        assert_eq!(actual.lines().count() / 4, 1);

        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        dbg!(&v);
        assert_eq!(v["top"]["_InternalReadCount"].as_i64().unwrap(), 10000); //that guy sees all 10k
        assert_eq!(v["report_top"]["molecule_count"].as_i64().unwrap(), 10000); //that guy sees all 10k
                                                                                //reads
        assert_eq!(v["middle"]["_InternalReadCount"], 10);
        assert_eq!(v["bottom"]["_InternalReadCount"], 1);
    }
}

#[test]
fn test_multi_stage_head_report_middle() {
    {
        //
        let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
[options]
    block_size = 15

[[step]]
    action = '_InternalReadCount'
    label = 'top'

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = 'Report'
    label = 'report_middle'
    count = true


[[step]]
    action = '_InternalReadCount'
    label = 'middle'

[[step]]
    action ='Head'
    n = 1

[[step]]
    action = '_InternalReadCount'
    label = 'bottom'

[output]
    prefix = 'output'
    report_json = true

");

        //check head
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
        assert_eq!(actual.lines().count() / 4, 1);

        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        dbg!(&v);
        assert!(v["top"]["_InternalReadCount"].as_i64().unwrap() <= 45); //no need to see all of them.
        assert_eq!(v["report_middle"]["molecule_count"].as_i64().unwrap(), 10); //that guy sees
                                                                                //exactly those  10 reads
        assert_eq!(v["middle"]["_InternalReadCount"], 10);
        assert_eq!(v["bottom"]["_InternalReadCount"], 1);
    }
}

#[test]
fn test_multi_stage_head_report_bottom() {
    {
        //
        let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
[options]
    block_size = 15

[[step]]
    action = '_InternalReadCount'
    label = 'top'

[[step]]
    action ='Head'
    n = 10
[[step]]
    action = '_InternalReadCount'
    label = 'middle'

[[step]]
    action ='Head'
    n = 1

[[step]]
    action = '_InternalReadCount'
    label = 'bottom'

[[step]]
    action = 'Report'
    label = 'report_bottom'
    count = true



[output]
    prefix = 'output'
    report_json = true

");

        //check head
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
        assert_eq!(actual.lines().count() / 4, 1);

        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        dbg!(&v);
        assert!(v["top"]["_InternalReadCount"].as_i64().unwrap() <= 60); //no need to see all of them.
        assert_eq!(v["report_bottom"]["molecule_count"].as_i64().unwrap(), 1);
        //reads
        assert_eq!(v["middle"]["_InternalReadCount"], 10);
        assert_eq!(v["bottom"]["_InternalReadCount"], 1);
    }
}

#[test]
fn test_multi_stage_head_report_middle_bottom() {
    {
        //
        let td = run("
[input]
    read1 = 'sample_data/ERR12828869_10k_1.fq.zst'
[options]
    block_size = 15

[[step]]
    action = '_InternalReadCount'
    label = 'top'

[[step]]
    action ='Head'
    n = 10

[[step]]
    action = '_InternalReadCount'
    label = 'middle'

[[step]]
    action = 'Report'
    label = 'report_middle'
    count = true


[[step]]
    action ='Head'
    n = 1

[[step]]
    action = '_InternalReadCount'
    label = 'bottom'

[[step]]
    action = 'Report'
    label = 'report_bottom'
    count = true



[output]
    prefix = 'output'
    report_json = true

");

        //check head
        let actual = std::fs::read_to_string(td.path().join("output_1.fq")).unwrap();
        assert_eq!(actual.lines().count() / 4, 1);

        let v = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(td.path().join("output.json")).unwrap(),
        )
        .unwrap();
        dbg!(&v);
        assert!(v["top"]["_InternalReadCount"].as_i64().unwrap() <= 100); //no need to see all of them. much less than the 10k though.
        assert_eq!(v["report_middle"]["molecule_count"].as_i64().unwrap(), 10);
        assert_eq!(v["report_bottom"]["molecule_count"].as_i64().unwrap(), 1);
        //reads
        assert_eq!(v["middle"]["_InternalReadCount"], 10);
        assert_eq!(v["bottom"]["_InternalReadCount"], 1);
    }
}
