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
        // Combined pipeline examples
        BenchmarkConfig::new(
            "QualityPipeline",
            r#"[[step]]
    action = "CalcLength"
    out_label = "length"
    segment = "read1"

[[step]]
    action = "CalcGCContent"
    out_label = "gc_content"
    segment = "read1"

[[step]]
    action = "FilterByNumericTag"
    in_label = "length"
    min_value = 50
    keep_or_remove = "Keep"

[[step]]
    action = "ForgetAllTags""#,
            molecule_count,
            thread_count,
        ),
        BenchmarkConfig::new(
            "ProcessingPipeline",
            r#"[[step]]
    action = "CutStart"
    n = 5
    segment = "read1"

[[step]]
    action = "CutEnd"
    n = 5
    segment = "read1"

[[step]]
    action = "FilterEmpty"
    segment = "read1"

[[step]]
    action = "ReverseComplement"
    segment = "read1""#,
            molecule_count,
            thread_count,
        ),
    ];

    for config in benchmarks {
        group.bench_with_input(
            BenchmarkId::new("pipeline", &config.name),
            &config,
            |b, config| b.iter(|| run_benchmark_pipeline(config)),
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark_key_steps);
criterion_main!(benches);
