use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

struct BenchmarkConfig {
    name: String,
    steps: String,
    molecule_count: u32,
    thread_count: u8,
    paired: bool,
}

impl BenchmarkConfig {
    fn new(name: &str, steps: &str, molecule_count: u32, thread_count: u8) -> Self {
        Self {
            name: name.to_string(),
            steps: steps.to_string(),
            molecule_count,
            thread_count,
            paired: false,
        }
    }
    fn set_parallel(self, value: bool) -> Self {
        Self {
            paired: value,
            ..self
        }
    }
}

fn run_benchmark_pipeline(config: &BenchmarkConfig) -> std::time::Duration {
    let sample_file = std::env::current_dir()
        .unwrap()
        .parent() // Go up from mbf-fastq-processor to project root
        .unwrap()
        .join("test_cases/sample_data/fastp_606.fq.gz");

    let toml_content = format!(
        r#"[input]
    read1 = "{}"
    {}

[options]
    thread_count = {}
    accept_duplicate_files = true

[benchmark]
    enable = true
    molecule_count = {}

{}
"#,
        sample_file.to_string_lossy(),
        if config.paired {
            r#"read2 = "test_cases/sample_data/fastp_606.fq.gz""#
        } else {
            ""
        },
        config.thread_count,
        config.molecule_count,
        config.steps
    );

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let toml_path = temp_dir.path().join("benchmark.toml");
    fs::write(&toml_path, &toml_content).expect("Failed to write TOML file");

    let start = std::time::Instant::now();
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release") // Use release build for realistic performance
        .arg("--")
        .arg("process")
        .arg(&toml_path)
        .current_dir(std::env::current_dir().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run mbf-fastq-processor");

    if !output.status.success() {
        panic!(
            "Benchmark failed for {}: {}",
            config.name,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    start.elapsed()
}

fn benchmark_key_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("step_benchmarks");
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(2));
    group.measurement_time(std::time::Duration::from_secs(10));

    let molecule_count = 1_000_000;
    let thread_count = 4; // Fixed thread count for consistency

    let benchmarks = vec![
        BenchmarkConfig::new(
            "Progress",
            r#"[[step]]
    action = "Progress""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Head",
            r#"[[step]]
    action = "Head"
    n = 500000"#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcLength",
            r#"[[step]]
    action = "CalcLength"
    out_label = "length"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcGCContent",
            r#"[[step]]
    action = "CalcGCContent"
    out_label = "gc_content"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CutStart",
            r#"[[step]]
    action = "CutStart"
    n = 10
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ReverseComplement",
            r#"[[step]]
    action = "ReverseComplement"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "FilterEmpty",
            r#"[[step]]
    action = "FilterEmpty"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Report_count",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = true
    "#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Report_base_statistics",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = false
    base_statistics = true
    "#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Report_length_distribution",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = false
    length_distribution = true
    "#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Report_duplicate_count_per_read",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = false
    duplicate_count_per_read = true
    "#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Report_duplicate_count_per_fragment",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = false
    duplicate_count_per_fragment = true
    "#,
            molecule_count,
            thread_count,
        )
        .set_parallel(true),
        BenchmarkConfig::new(
            "Report_count_oligios",
            r#"[[step]]
    action = "Report"
    name = "benchmark_report"
    count = false
    count_oligos = ["AGTCTA", "CGATCG"]
    "#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcBaseContent",
            r#"[[step]]
    action = "CalcBaseContent"
    bases_to_count = "AC"
    out_label = "base_content"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcComplexity",
            r#"[[step]]
    action = "CalcComplexity"
    out_label = "complexity"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcExpectedError",
            r#"[[step]]
    action = "CalcExpectedError"
    out_label = "expected_error"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcKmers",
            r#"[[step]]
    action = "CalcKmers"
    k = 3
    out_label = "kmers"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcNCount",
            r#"[[step]]
    action = "CalcNCount"
    out_label = "n_count"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "CalcQualifiedBases",
            r#"[[step]]
    action = "CalcQualifiedBases"
    out_label = "qualified_bases"
    segment = "read1"
    min_quality = 20

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ConcatTags",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag1"
    anchor = "Start"

[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 3
    length = 3
    out_label = "tag2"
    anchor = "Start"

[[step]]
    action = "ConcatTags"
    in_label = ["tag1", "tag2"]
    out_label = "concatenated"
    separator = "_"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ConvertQuality",
            r#"[[step]]
    action = "ConvertQuality"
    input_encoding = "Phred33"
    output_encoding = "Phred64"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ConvertRegionsToLength",
            r#"[[step]]
    action = "ExtractRegions"
    segment = "read1"
    out_label = "regions"
    start_pattern = "ATG"
    end_pattern = "TAA"

[[step]]
    action = "ConvertRegionsToLength"
    in_label = "regions"
    out_label = "lengths"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Demultiplex",
            r#"[barcodes.sample_barcodes]
    'AAAAAAAA' = 'sample_1'
    'CCCCCCCC' = 'sample_2'

[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 8
    out_label = "barcode"
    anchor = "Start"

[[step]]
    action = "Demultiplex"
    barcodes = "sample_barcodes"
    in_label = "barcode"
    max_mismatches = 1

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "EvalExpression",
            r#"[[step]]
    action = "CalcLength"
    segment = "read1"
    out_label = "length"

[[step]]
    action = "CalcGCContent"
    segment = "read1"
    out_label = "gc_content"

[[step]]
    action = "EvalExpression"
    expression = "length * gc_content"
    out_label = "score"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractIUPAC",
            r#"[[step]]
    action = "ExtractIUPAC"
    segment = "read1"
    pattern = "WSWYW"
    out_label = "iupac_match"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractIUPACSuffix",
            r#"[[step]]
    action = "ExtractIUPACSuffix"
    segment = "read1"
    pattern = "ATWGCR"
    out_label = "suffix"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractIUPACWithIndel",
            r#"[[step]]
    action = "ExtractIUPACWithIndel"
    segment = "read1"
    pattern = "ATNGC"
    out_label = "indel_match"
    max_mismatches = 1

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractLongestPolyX",
            r#"[[step]]
    action = "ExtractLongestPolyX"
    segment = "read1"
    base = "A"
    out_label = "poly_a"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractLowQualityEnd",
            r#"[[step]]
    action = "ExtractLowQualityEnd"
    segment = "read1"
    quality_threshold = 20
    out_label = "low_qual_end"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractLowQualityStart",
            r#"[[step]]
    action = "ExtractLowQualityStart"
    segment = "read1"
    quality_threshold = 20
    out_label = "low_qual_start"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractPolyTail",
            r#"[[step]]
    action = "ExtractPolyTail"
    segment = "read1"
    base = "A"
    min_length = 5
    out_label = "poly_tail"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractRegex",
            r#"[[step]]
    action = "ExtractRegex"
    segment = "read1"
    pattern = "(ATG...)"
    out_label = "regex_match"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractRegion",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 10
    length = 20
    out_label = "region"
    anchor = "Start"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractRegions",
            r#"[[step]]
    action = "ExtractRegions"
    segment = "read1"
    start_pattern = "ATG"
    end_pattern = "TAA"
    out_label = "regions"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ExtractRegionsOfLowQuality",
            r#"[[step]]
    action = "ExtractRegionsOfLowQuality"
    segment = "read1"
    quality_threshold = 20
    min_length = 5
    out_label = "low_qual_regions"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "FilterByTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "FilterByTag"
    in_label = "tag"
    value = "ATG"
    keep_or_remove = "Keep"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "FilterReservoirSample",
            r#"[[step]]
    action = "FilterReservoirSample"
    n = 500000
    seed = 42""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "FilterSample",
            r#"[[step]]
    action = "FilterSample"
    rate = 0.5
    seed = 42""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ForgetTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "ForgetTag"
    in_label = "tag""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "HammingCorrect",
            r#"[barcodes.sample_barcodes]
    'AAAAAAAA' = 'sample_1'
    'CCCCCCCC' = 'sample_2'

[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 8
    out_label = "barcode"
    anchor = "Start"

[[step]]
    action = "HammingCorrect"
    barcodes = "sample_barcodes"
    in_label = "barcode"
    out_label = "corrected"
    max_distance = 1

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Inspect",
            r#"[[step]]
    action = "Inspect"
    segment = "read1"
    n = 10""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "LowercaseSequence",
            r#"[[step]]
    action = "LowercaseSequence"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "LowercaseTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "LowercaseTag"
    in_label = "tag"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "MergeReads",
            r#"[[step]]
    action = "MergeReads"
    min_overlap = 10""#,
            molecule_count,
            thread_count,
        )
        .set_parallel(true),
        BenchmarkConfig::new(
            "Postfix",
            r#"[[step]]
    action = "Postfix"
    text = "_test"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Prefix",
            r#"[[step]]
    action = "Prefix"
    text = "test_"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "QuantifyTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "QuantifyTag"
    in_label = "tag"
    out_label = "quantified"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Rename",
            r#"[[step]]
    action = "Rename"
    segment = "read1"
    new_name = "renamed_read""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ReplaceTagWithLetter",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "ReplaceTagWithLetter"
    in_label = "tag"
    letter = "N"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Skip",
            r#"[[step]]
    action = "Skip"
    n = 100""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "SpotCheckReadPairing",
            r#"[[step]]
    action = "SpotCheckReadPairing"
    n = 1000""#,
            molecule_count,
            thread_count,
        )
        .set_parallel(true),
        BenchmarkConfig::new(
            "StoreTagInComment",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "StoreTagInComment"
    in_label = "tag"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "StoreTagInFastQ",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "StoreTagInFastQ"
    in_label = "tag"
    segment = "read1"
    location = "Name"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "StoreTagInSequence",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "StoreTagInSequence"
    in_label = "tag"
    segment = "read1"
    location = "Start"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "StoreTagLocationInComment",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "StoreTagLocationInComment"
    in_label = "tag"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "StoreTagsInTable",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "StoreTagsInTable"
    infix = "tags"
    compression = "Raw""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Swap",
            r#"[[step]]
    action = "Swap""#,
            molecule_count,
            thread_count,
        )
        .set_parallel(true),
        BenchmarkConfig::new(
            "TagDuplicates",
            r#"[[step]]
    action = "TagDuplicates"
    source = "read1"
    out_label = "is_duplicate"
    false_positive_rate = 0.01
    seed = 42

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "TagOtherFileByName",
            r#"[[step]]
    action = "TagOtherFileByName"
    other_file = "test_cases/sample_data/fastp_606.fq.gz"
    out_label = "in_other"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "TagOtherFileBySequence",
            r#"[[step]]
    action = "TagOtherFileBySequence"
    other_file = "test_cases/sample_data/fastp_606.fq.gz"
    out_label = "seq_in_other"
    segment = "read1"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "TrimAtTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 10
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "TrimAtTag"
    in_label = "tag"
    segment = "read1"
    direction = "Start"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "Truncate",
            r#"[[step]]
    action = "Truncate"
    segment = "read1"
    max_length = 50""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "UppercaseSequence",
            r#"[[step]]
    action = "UppercaseSequence"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "UppercaseTag",
            r#"[[step]]
    action = "ExtractRegion"
    segment = "read1"
    start = 0
    length = 3
    out_label = "tag"
    anchor = "Start"

[[step]]
    action = "UppercaseTag"
    in_label = "tag"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ValidateAllReadsSameLength",
            r#"[[step]]
    action = "ValidateAllReadsSameLength""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ValidateName",
            r#"[[step]]
    action = "ValidateName"
    regex = "@.*""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ValidateQuality",
            r#"[[step]]
    action = "ValidateQuality"
    min_quality = 0
    max_quality = 93
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ValidateSeq",
            r#"[[step]]
    action = "ValidateSeq"
    segment = "read1"
    allowed_letters = "ATCGN""#,
            molecule_count,
            thread_count,
        ),
        //         // Combined pipeline examples
        //         BenchmarkConfig::new(
        //             "QualityPipeline",
        //             r#"[[step]]
        //     action = "CalcLength"
        //     out_label = "length"
        //     segment = "read1"
        //
        // [[step]]
        //     action = "CalcGCContent"
        //     out_label = "gc_content"
        //     segment = "read1"
        //
        // [[step]]
        //     action = "FilterByNumericTag"
        //     in_label = "length"
        //     min_value = 50
        //     keep_or_remove = "Keep"
        //
        // [[step]]
        //     action = "ForgetAllTags""#,
        //             molecule_count,
        //             thread_count,
        //         ),
        //         BenchmarkConfig::new(
        //             "ProcessingPipeline",
        //             r#"[[step]]
        //     action = "CutStart"
        //     n = 5
        //     segment = "read1"
        //
        // [[step]]
        //     action = "CutEnd"
        //     n = 5
        //     segment = "read1"
        //
        // [[step]]
        //     action = "FilterEmpty"
        //     segment = "read1"
        //
        // [[step]]
        //     action = "ReverseComplement"
        //     segment = "read1""#,
        //             molecule_count,
        //             thread_count,
        //         ),
    ];

    for config in benchmarks {
        group
            .bench_with_input(
                BenchmarkId::new("pipeline", &config.name),
                &config,
                |b, config| b.iter(|| run_benchmark_pipeline(config)),
            )
            .measurement_time(std::time::Duration::from_secs(35));
    }
    group.finish();
}

criterion_group!(benches, benchmark_key_steps);
criterion_main!(benches);
