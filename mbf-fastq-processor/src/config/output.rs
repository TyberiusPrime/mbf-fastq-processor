use std::collections::HashSet;

use schemars::JsonSchema;
use toml_pretty_deser::{prelude::*, suggest_alternatives};

use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Output {
    pub prefix: String,
    #[tpd(default)]
    pub suffix: Option<String>,
    #[tpd(default)]
    pub format: FileFormat,
    pub compression: CompressionFormat,
    #[tpd(default)]
    pub compression_level: Option<u8>,
    #[tpd(default)]
    pub compression_threads: Option<usize>,

    #[tpd(default)]
    pub report_html: bool,
    #[tpd(default)]
    pub report_json: bool,
    #[tpd(default)]
    pub report_timing: bool,

    #[tpd(default)]
    pub stdout: bool,
    #[tpd(default)]
    pub interleave: Option<Vec<String>>,

    #[tpd(default)]
    pub output: Option<Vec<String>>,

    #[tpd(default)]
    pub output_hash_uncompressed: bool,
    #[tpd(default)]
    pub output_hash_compressed: bool,
    pub ix_separator: String,

    #[tpd(default)]
    pub chunksize: Option<usize>,
}

impl VerifyIn<super::PartialConfig> for PartialOutput {
    fn verify(
        &mut self,
        parent: &super::PartialConfig,
        _options: &VerifyOptions,
    ) -> Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.ix_separator.verify(|ix_separator| {
            if ix_separator.contains('/')
                || ix_separator.contains('\\')
                || ix_separator.contains(':')
            {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not contain '/', '\\' or ':'"),
                ))
            } else if ix_separator.is_empty() {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not be empty"),
                ))
            } else {
                Ok(())
            }
        });

        self.chunksize.verify(|chunk_size| {
            if let Some(chunk_size) = chunk_size.as_ref() {
                if *chunk_size == 0 {
                    return Err(ValidationFailure::new(
                        "Must not be 0.",
                        Some("'Chunksize' must be greater than zero when specified. Increase or remove setting"),
                    ));
                } else if let Some(true) = self.stdout.as_ref() {
                    return Err(ValidationFailure::new(
                        "Invalid when stdout = true",
                        Some("Either remove 'chunksize' or set 'stdout' to false"),
                    ));
                }
        }
            Ok(())
        });
        self.ix_separator.or_with(default_ix_separator);

        if let Some(Some(_level)) = self.compression_level.value {
            if self
                .compression
                .as_ref()
                .is_some_and(CompressionFormat::is_compressed)
            {
                validate_compression_level_u8(&self.compression, &mut self.compression_level);
            } else {
                self.compression_level.state = TomlValueState::ValidationFailed {
                    message: "Invalid when compression='uncompressed'".to_string(),
                };
                self.compression_level.help = Some(
                "Either remove the compression_lever parameter, or set the compression to 'Gzip'/'Zstd'".to_string());
            }
        }
        self.verify_compression_and_stdout();

        self.verify_stdout(parent);
        self.compression.or_default();
        self.verify_output_segments(parent);
        Ok(())
    }
}

impl PartialOutput {
    fn verify_compression_and_stdout(&mut self) {
        if let Some(FileFormat::Bam) = self.format.as_ref() {
            if *self.output_hash_uncompressed.unwrap_ref() {
                self.output_hash_uncompressed.state = TomlValueState::new_validation_failed(
                    "Uncompressed hashing is not supported when format = 'bam'.",
                );
                self.output_hash_uncompressed.help = Some(
                    "Either disable output_hash_uncompressed, or switch output formats".to_string(),
                );
            }
            if *self.stdout.unwrap_ref() {
                self.stdout.state = TomlValueState::new_validation_failed(
                    "Output to stdout is not supported when format = 'bam'.",
                );
                self.stdout.help =
                    Some("Either disable stdout output, or switch output formats".to_string());
            }
            if let Some(CompressionFormat::Uncompressed) = self.compression.as_ref() {
                self.compression.state = TomlValueState::new_validation_failed(
                    "Compression is not supported when format = 'bam'.",
                );
                self.compression.help = Some(
                    "Either set compression to 'uncompressed', or switch output formats"
                        .to_string(),
                );
            }
        }
    }

    fn verify_stdout(&mut self, config: &super::PartialConfig) {
        if let Some(true) = self.stdout.as_ref() {
            if let Some(Some(_)) = self.output.as_ref() {
                self.stdout.state = TomlValueState::new_validation_failed(
                    "Cannot specify both 'stdout' and 'output' options together.",
                );
                self.stdout.help = Some("Remove either one ".to_string());
            }
            if let Some(None) = self.interleave.as_ref()
                && let Some(input) = config.input.as_ref()
            {
                self.interleave = TomlValue::new_ok(
                    Some(
                        input
                            .get_segment_order()
                            .iter()
                            .map(|x| TomlValue::new_ok(x.clone(), 0..0))
                            .collect(),
                    ),
                    self.interleave.span(),
                );
            }
        } else if let Some(None) = self.output.as_ref() {
            if let Some(Some(_)) = self.interleave.as_ref() {
                self.output = TomlValue::new_ok(Some(Vec::new()), 0..0); // no extra output by default
            } else {
                if let Some(input) = config.input.as_ref() {
                    //default to output all targets
                    self.output = TomlValue::new_ok(
                        Some(
                            input
                                .get_segment_order()
                                .iter()
                                .map(|x| TomlValue::new_ok(x.clone(), 0..0))
                                .collect(),
                        ),
                        0..0,
                    );
                }
            }
        }
    }

    fn verify_output_segments(&mut self, config: &super::PartialConfig) {
        if let Some(input) = config.input.as_ref() {
            let valid_segments: HashSet<&String> = input.get_segment_order().iter().collect();

            if let Some(Some(output_segments)) = self.output.as_mut() {
                let mut seen_segments = HashSet::new();
                let mut any_failed = false;
                let all_seen: HashSet<String> = output_segments
                    .iter()
                    .filter_map(|x| x.as_ref())
                    .map(|x| x.to_string())
                    .collect();
                for segment in output_segments.iter_mut() {
                    if let Some(segment_str) = segment.as_ref() {
                        if !valid_segments.contains(segment_str) {
                            let available: Vec<&String> = valid_segments
                                .iter()
                                .filter_map(|x| {
                                    if !all_seen.contains(*x) {
                                        Some(*x)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            segment.help = Some(toml_pretty_deser::suggest_alternatives(
                                &segment_str,
                                &available,
                            ));
                            segment.state = TomlValueState::new_validation_failed(
                                "Not found in input segments",
                            );
                            any_failed = true;
                        } else {
                            if !seen_segments.insert(segment_str.to_string()) {
                                segment.help = Some(format!("Remove all but one '{segment_str}'",));
                                segment.state = TomlValueState::new_validation_failed(
                                    "Segment is duplicated in output segments",
                                );
                                any_failed = true;
                            }
                        }
                    }
                }
                if any_failed {
                    self.output.state = TomlValueState::Nested;
                }
            }

            if let Some(Some(interleave_order)) = self.interleave.as_mut() {
                let mut seen_segments = HashSet::new();
                let mut any_failed = false;
                let all_seen: HashSet<String> = interleave_order
                    .iter()
                    .filter_map(|x| x.as_ref())
                    .map(|x| x.to_string())
                    .collect();
                for segment in interleave_order.iter_mut() {
                    if let Some(segment_str) = segment.as_ref() {
                        if !valid_segments.contains(segment_str) {
                            let available: Vec<&String> = valid_segments
                                .iter()
                                .filter_map(|x| {
                                    if !all_seen.contains(*x) {
                                        Some(*x)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            segment.help = Some(suggest_alternatives(segment_str, &available));
                            segment.state = TomlValueState::new_validation_failed(
                                "Not found in input segments",
                            );
                            any_failed = true;
                        } else {
                            if !seen_segments.insert(segment_str.to_string()) {
                                segment.help = Some(format!("Remove all but one '{segment_str}'",));
                                segment.state = TomlValueState::new_validation_failed(
                                    "Segment is duplicated in interleave order",
                                );
                                any_failed = true;
                            }
                        }
                    }
                }
                if any_failed {
                    self.interleave.state = TomlValueState::Nested;
                } else {
                    if interleave_order.len() < 2 && !*self.stdout.unwrap_ref() {
                        self.interleave.state = TomlValueState::new_validation_failed(
                            "Must contain at least two segments to interleave.",
                        );
                        self.interleave.help = Some(format!(
                            "Either add another segment to interleave, or remove interleave, or output to files in stead of stdout"
                        ));
                        //     ));
                    } else {
                        //make sure there's no overlap between interleave and output
                        if let Some(Some(output_segments)) = self.output.as_ref() {
                            for segment in output_segments {
                                if let Some(segment_str) = segment.as_ref()
                                    && let Some(found) = interleave_order
                                        .iter_mut()
                                        .find(|x| x.as_ref() == Some(segment_str))
                                {
                                    let spans = vec![
                                        (found.span(), "Duplicate output & interleave".to_string()),
                                        (
                                            segment.span(),
                                            "Duplicate output & interleave".to_string(),
                                        ),
                                    ];

                                    found.state = TomlValueState::Custom { spans };
                                    found.help = Some(
                                        "Remove from either 'interleaved' or from 'output'"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Output {
    #[must_use]
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            suffix: None,
            format: FileFormat::Fastq,
            compression: CompressionFormat::Uncompressed,
            compression_level: None,
            compression_threads: None,
            report_html: false,
            report_json: false,
            report_timing: false,
            stdout: false,
            interleave: None,
            output: None,
            output_hash_uncompressed: false,
            output_hash_compressed: false,
            ix_separator: default_ix_separator(),
            chunksize: None,
        }
    }

    #[must_use]
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}

/// Validates that the compression level is within the expected range for the given compression format
pub fn validate_compression_level_u8(
    compression: &TomlValue<CompressionFormat>,
    compression_level: &mut TomlValue<Option<u8>>,
) {
    if let Some(Some(level)) = compression_level.as_ref() {
        match compression.as_ref() {
            Some(CompressionFormat::Uncompressed) => {
                if *level != 0 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: "Compression level {level} specified for uncompressed output"
                            .to_string(),
                    };
                    compression_level.help = Some("Remove compression_level".to_string());
                }
            }
            Some(CompressionFormat::Gzip) => {
                if *level > 9 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: "Invalid Value".to_string(),
                    };
                    compression_level.help = Some("Valid range is 0-9 for gzip.".to_string());
                }
            }
            Some(CompressionFormat::Zstd) => {
                if *level == 0 || *level > 22 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: ("Invalid Value".to_string()),
                    };
                    compression_level.help = Some("Valid range is 1-22 for zstd.".to_string());
                }
            }
            None => {
                //nothing to verify, compression not set
            }
        }
    } else {
        // No compression level specified - rapidgzip is still invalid for output
    }
}
