#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]
#![allow(clippy::default_trait_access)]

pub mod cli;
pub mod config;
pub mod cookbooks;
pub mod demultiplex;
mod dna;
pub mod documentation;
pub mod interactive;
pub mod io;
pub mod list_steps;
mod output;
mod pipeline;
mod pipeline_workpool;
mod transformations;

pub use io::FastQRead;
pub use cli::process::run;
pub use cli::validate::validate_config;
pub use cli::verify::verify_outputs;
pub use cli::decompress_file;
pub use cli::join_nonempty;
pub use cli::calculate_size_difference_percent;
pub use transformations::Transformation;

use regex::Regex;
use std::path::Path;

pub fn normalize_report_content(content: &str, input_dir: Option<&Path>) -> String {
    let normalize_re = Regex::new(
        r#""(?P<key>version|program_version|cwd|working_directory|repository)"\s*:\s*"[^"]*""#,
    )
    .expect("invalid normalize regex");

    let content = normalize_re
        .replace_all(content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    let normalize_re = Regex::new(r#""(?P<key>threads_per_segment|thread_count)"\s*:\s*[^"]*"#)
        .expect("invalid normalize regex");

    let content = normalize_re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    let input_toml_re =
        Regex::new(r#""input_toml"\s*:\s*"(?:[^"\\]|\\.)*""#).expect("invalid input_toml regex");

    let content = input_toml_re
        .replace_all(&content, r#""input_toml": "_IGNORED_""#)
        .into_owned();

    if let Some(input_dir) = input_dir {
        content.replace(&format!("{}/", input_dir.to_string_lossy()), "")
    } else {
        content
    }
}

pub fn normalize_progress_content(content: &str) -> String {
    let float_re = Regex::new(r"\d+[._0-9]*").expect("invalid float regex");
    let normalized = float_re.replace_all(content, "_IGNORED_").into_owned();

    let int_re = Regex::new(r"\b\d+\b").expect("invalid int regex");
    let normalized = int_re.replace_all(&normalized, "_IGNORED_").into_owned();

    let file_re =
        Regex::new("(?:^|[^A-Za-z0-9._-])(/(?:[^/\\s]+/)*([^/\\s]+))").expect("invalid file regex");
    file_re.replace_all(&normalized, "$2").into_owned()
}

#[must_use]
#[mutants::skip]
pub fn get_number_of_cores() -> usize {
    std::env::var("MBF_FASTQ_PROCESSOR_NUM_CPUS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_calculate_size_difference_percent() {
        use crate::cli::calculate_size_difference_percent;

        let test_cases = vec![
            (100, 105, 5.0),
            (100, 95, 5.0),
            (100, 97, 3.0),
            (0, 100, 100.0),
            (100, 0, 100.0),
            (0, 0, 0.0),
            (200, 210, 5.0),
            (200, 190, 5.0),
        ];

        for (len_a, len_b, expected) in test_cases {
            let result = calculate_size_difference_percent(len_a, len_b);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "Failed for len_a: {}, len_b: {}: expected {}, got {}",
                len_a,
                len_b,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_decompress_file() {
        use super::decompress_file;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut temp_file, flate2::Compression::default());
            encoder
                .write_all(b"Hello, world!")
                .expect("Failed to write to encoder");
            encoder.finish().expect("Failed to finish encoding");
        }

        let decompressed_data =
            decompress_file(temp_file.path()).expect("Failed to decompress file");

        assert_eq!(decompressed_data, b"Hello, world!");
    }
}
