use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::config::{Config, InputOptions};
use crate::io::FastQBlocksCombined;
use crate::transformations::{InputInfo, Step, Transformation};

/// Configuration for benchmarking a single step
pub struct BenchmarkConfig {
    /// The step to benchmark
    pub step: Transformation,
    /// Total number of reads to process
    pub total_read_count: usize,
    /// Input file paths
    pub input_files: Vec<String>,
    /// Block size for reading
    pub block_size: usize,
    /// Buffer size for reading
    pub buffer_size: usize,
}

/// Result of a benchmark run
#[derive(Debug)]
pub struct BenchmarkResult {
    pub step_name: String,
    pub total_reads: usize,
    pub duration: Duration,
    pub reads_per_second: f64,
}

impl BenchmarkResult {
    pub fn format(&self) -> String {
        format!(
            "{:<30} | {:>12} reads | {:>10.3} s | {:>15.0} reads/s",
            self.step_name,
            self.total_reads,
            self.duration.as_secs_f64(),
            self.reads_per_second
        )
    }
}

/// Benchmark a single transformation step
pub fn benchmark_step(config: BenchmarkConfig) -> Result<BenchmarkResult> {
    // Parse config to get input structure
    let input_files = config.input_files;

    // Read one block of reads
    let input_options = InputOptions {
        fasta_fake_quality: Some(33),
        bam_include_mapped: Some(true),
        bam_include_unmapped: Some(true),
        read_comment_character: b' ',
    };

    // Open input files directly
    let mut opened_files = Vec::new();
    for file_path in &input_files {
        opened_files.push(crate::io::open_input_file(file_path)?);
    }

    let mut parser = crate::io::parsers::ChainedParser::new(
        opened_files,
        config.block_size,
        config.buffer_size,
        input_options,
    );

    let parse_result = parser.parse()?;
    let base_block = parse_result.fastq_block;

    if base_block.entries.is_empty() {
        bail!("No reads found in input files");
    }

    // Create a combined block structure (assuming single segment for now)
    let base_combined = FastQBlocksCombined {
        segments: vec![base_block],
        output_tags: None,
        tags: HashMap::default(),
        is_final: false,
    };

    // Calculate how many times we need to clone the block
    let reads_per_block = base_combined.segments[0].entries.len();
    let repetitions = (config.total_read_count + reads_per_block - 1) / reads_per_block;

    // Clone the block to create enough reads
    let mut blocks_to_process = Vec::new();
    for _ in 0..repetitions {
        blocks_to_process.push(base_combined.clone());
    }

    // Initialize the step (we need dummy values for some parameters)
    let mut step = config.step.clone();
    let input_info = InputInfo {
        segment_order: vec!["read1".to_string()],
        barcodes_data: std::collections::BTreeMap::new(),
        comment_insert_char: b' ',
        initial_filter_capacity: Some(10000),
    };

    let temp_dir = tempfile::tempdir()?;
    step.init(
        &input_info,
        "benchmark",
        temp_dir.path(),
        "_",
        &crate::demultiplex::OptDemultiplex::No,
        false,
    )?;

    // Benchmark the processing
    let start = Instant::now();
    let mut total_processed = 0;

    for (block_no, block) in blocks_to_process.into_iter().enumerate() {
        let (processed_block, _) = step.apply(
            block,
            &input_info,
            block_no,
            &crate::demultiplex::OptDemultiplex::No,
        )?;
        total_processed += processed_block.segments[0].entries.len();

        // Stop if we've processed enough reads
        if total_processed >= config.total_read_count {
            break;
        }
    }

    let duration = start.elapsed();
    let reads_per_second = total_processed as f64 / duration.as_secs_f64();

    // Get step name
    let step_name = get_step_name(&config.step);

    Ok(BenchmarkResult {
        step_name,
        total_reads: total_processed,
        duration,
        reads_per_second,
    })
}

/// Get a human-readable name for a transformation step
fn get_step_name(step: &Transformation) -> String {
    match step {
        Transformation::CutStart(_) => "CutStart".to_string(),
        Transformation::CutEnd(_) => "CutEnd".to_string(),
        Transformation::Truncate(_) => "Truncate".to_string(),
        Transformation::Prefix(_) => "Prefix".to_string(),
        Transformation::Postfix(_) => "Postfix".to_string(),
        Transformation::ConvertQuality(_) => "ConvertQuality".to_string(),
        Transformation::ReverseComplement(_) => "ReverseComplement".to_string(),
        Transformation::Rename(_) => "Rename".to_string(),
        Transformation::Swap(_) => "Swap".to_string(),
        Transformation::LowercaseTag(_) => "LowercaseTag".to_string(),
        Transformation::UppercaseTag(_) => "UppercaseTag".to_string(),
        Transformation::LowercaseSequence(_) => "LowercaseSequence".to_string(),
        Transformation::UppercaseSequence(_) => "UppercaseSequence".to_string(),
        Transformation::TrimAtTag(_) => "TrimAtTag".to_string(),
        Transformation::MergeReads(_) => "MergeReads".to_string(),
        Transformation::FilterByTag(_) => "FilterByTag".to_string(),
        Transformation::FilterByNumericTag(_) => "FilterByNumericTag".to_string(),
        Transformation::Head(_) => "Head".to_string(),
        Transformation::Skip(_) => "Skip".to_string(),
        Transformation::FilterEmpty(_) => "FilterEmpty".to_string(),
        Transformation::FilterSample(_) => "FilterSample".to_string(),
        Transformation::FilterReservoirSample(_) => "FilterReservoirSample".to_string(),
        Transformation::SpotCheckReadPairing(_) => "SpotCheckReadPairing".to_string(),
        Transformation::ValidateSeq(_) => "ValidateSeq".to_string(),
        Transformation::ValidateQuality(_) => "ValidateQuality".to_string(),
        Transformation::ValidateName(_) => "ValidateName".to_string(),
        Transformation::ValidateAllReadsSameLength(_) => "ValidateAllReadsSameLength".to_string(),
        Transformation::ExtractIUPAC(_) => "ExtractIUPAC".to_string(),
        Transformation::ExtractIUPACWithIndel(_) => "ExtractIUPACWithIndel".to_string(),
        Transformation::ExtractRegex(_) => "ExtractRegex".to_string(),
        Transformation::ExtractRegion(_) => "ExtractRegion".to_string(),
        Transformation::ExtractRegions(_) => "ExtractRegions".to_string(),
        Transformation::CalcLength(_) => "CalcLength".to_string(),
        Transformation::CalcBaseContent(_) => "CalcBaseContent".to_string(),
        Transformation::CalcGCContent(_) => "CalcGCContent".to_string(),
        Transformation::CalcNCount(_) => "CalcNCount".to_string(),
        Transformation::CalcComplexity(_) => "CalcComplexity".to_string(),
        Transformation::CalcQualifiedBases(_) => "CalcQualifiedBases".to_string(),
        Transformation::CalcExpectedError(_) => "CalcExpectedError".to_string(),
        Transformation::CalcKmers(_) => "CalcKmers".to_string(),
        _ => format!("{:?}", step).split('(').next().unwrap_or("Unknown").to_string(),
    }
}

/// Run benchmarks from a TOML config file
/// The config should have an [input] section and one or more [[step]] sections
pub fn benchmark_from_config(
    toml_file: &Path,
    total_read_count: usize,
) -> Result<Vec<BenchmarkResult>> {
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;

    // Initialize the input to populate structured input
    parsed.input.init()?;

    // Get input files
    let input_files = match &parsed.input.structured {
        Some(crate::config::StructuredInput::Interleaved { files, .. }) => {
            files.clone()
        }
        Some(crate::config::StructuredInput::Segmented { segment_files, .. }) => {
            // For segmented input, just take the first segment for benchmarking
            segment_files.values().next()
                .context("No segment files found")?
                .clone()
        }
        None => bail!("No input files specified in config"),
    };

    let mut results = Vec::new();

    // Benchmark each step
    for step in &parsed.transform {
        let config = BenchmarkConfig {
            step: step.clone(),
            total_read_count,
            input_files: input_files.clone(),
            block_size: 10000,
            buffer_size: 1024 * 1024,
        };

        match benchmark_step(config) {
            Ok(result) => {
                println!("{}", result.format());
                results.push(result);
            }
            Err(e) => {
                eprintln!("Error benchmarking step: {}", e);
            }
        }
    }

    Ok(results)
}
