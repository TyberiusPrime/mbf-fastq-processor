use fastqrab_config::{TagLabel, offer_alternatives, tpd_adapt_u8_from_byte_or_char};
use fastqrab_io::{CompressionFormat, FileFormat};
use indexmap::IndexMap;
use schemars::JsonSchema;
use std::collections::HashSet;
use toml_pretty_deser::prelude::*;

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[must_use]
pub fn default_bam_comment_separation_char() -> u8 {
    b' '
}

/// A validated two-character BAM auxiliary tag name (e.g. `"BC"`).
///
/// Only ASCII alphanumeric characters are accepted.
#[derive(Clone, Debug, JsonSchema)]
#[schemars(with = "String")]
pub struct BamTag(pub [u8; 2]);

impl TryFrom<&str> for BamTag {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(format!(
                "BAM tag must be exactly 2 characters; got '{}' ({} chars). \
                 BAM auxiliary tag names are exactly 2 ASCII alphanumeric characters.",
                s,
                bytes.len()
            ));
        }
        if !bytes.iter().all(|&b| b.is_ascii_alphanumeric()) {
            return Err(format!(
                "BAM tag must be 2 alphanumeric ASCII characters; got '{}'. \
                 Only [A-Za-z0-9] are allowed.",
                s
            ));
        }
        Ok(BamTag([bytes[0], bytes[1]]))
    }
}

toml_pretty_deser::impl_visitor_for_try_from_str!(BamTag, "Invalid BAM tag");

/// Alias so the `#[tpd]` macro can find the "partial" type for `BamTag`
/// (which is its own visitor – no separate Partial struct is generated).

pub type PartialBamTag = BamTag;

/// Source for reference sequences used by `tag_to_reference` (Feature B).
///
/// Exactly one of `barcodes` or `from_bam` must be set.
#[tpd(no_verify)]
#[derive(Clone, Debug, JsonSchema)]
pub struct TagToReference {
    /// The fastqrab tag label whose value selects the reference sequence name.
    pub tag: Option<String>,

    /// Name of a `[barcodes.<name>]` section whose keys become reference names.
    #[tpd(default, alias = "from_barcodes")]
    pub references_from_barcodes: Option<String>,

    /// Path to a BAM file whose `@SQ` header lines define the reference sequences.
    #[tpd(default, alias = "from_bam", alias = "template")]
    pub references_from_bam: Option<String>,
}

#[must_use]

/// BAM-specific output options.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct BamOutputOptions {
    /// Character used to split read names into a BAM name field and a `CO` auxiliary tag.
    /// Defaults to `' '` (space).  Reads whose names contain this character are split; the
    /// part after the character is placed in a `CO` tag so it can exceed the 254-byte limit.
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub comment_separation_char: u8,

    /// Map of fastqrab tag labels to BAM auxiliary tag names
    ///
    /// Each key is a fastqrab tag label; each value is the two-character BAM
    /// auxiliary tag name to write (e.g. `BC`).
    #[tpd(nested, alias = "tags")]
    #[schemars(skip)]
    pub tag_to_bam_tag: IndexMap<TagLabel, BamTag>,

    /// Export a fastqrab tag value as the BAM reference name
    #[tpd(nested)]
    pub tag_to_reference: Option<TagToReference>,

    /// In-label of a Demultiplex step whose outputs should be concatenated into merged BAM
    /// files.  All demultiplexed files for that step are merged; any additional demultiplex
    /// levels produce one merged file per combination of the remaining levels.
    #[tpd(default)]
    pub merge_demultiplexed: Option<TagLabel>,

    /// Write a BAI index alongside each merged BAM file (default: true).
    #[tpd(default)]
    pub index_merged: Option<bool>,
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
    #[tpd(default, alias = "interleaved")]
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

    /// BAM-specific output options (comment separator, tag exports, reference assignment).
    #[tpd(nested)]
    pub bam: Option<BamOutputOptions>,
}

impl VerifyIn<PartialOutput> for PartialBamOutputOptions {
    fn verify(
        &mut self,
        parent: &PartialOutput,
        _options: &VerifyOptions,
    ) -> Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.comment_separation_char
            .or_with(default_bam_comment_separation_char);
        self.tag_to_bam_tag
            .or_with(|| toml_pretty_deser::MapAndKeys {
                map: indexmap::IndexMap::new(),
                keys: vec![],
            });

        // Validate tag_to_reference: exactly one of barcodes or from_bam must be set.
        if let Some(Some(tag_to_ref)) = self.tag_to_reference.as_mut() {
            let has_barcodes = tag_to_ref
                .references_from_barcodes
                .as_ref()
                .and_then(|x| x.as_ref())
                .is_some();
            let has_from_bam = tag_to_ref
                .references_from_bam
                .as_ref()
                .and_then(|x| x.as_ref())
                .is_some();
            if !has_barcodes && !has_from_bam {
                tag_to_ref.references_from_barcodes.state = TomlValueState::new_validation_failed(
                    "Either 'reference_from_barcodes' or 'reference_from_bam' must be specified",
                );
                tag_to_ref.references_from_barcodes.help = Some(
                    "Set 'reference_from_barcodes' to a barcode section name, or 'references_from_bam' to a BAM file path."
                        .to_string(),
                );
            } else if has_barcodes && has_from_bam {
                tag_to_ref.references_from_bam.state = TomlValueState::Custom {
                    spans: vec![
                        (
                            tag_to_ref.references_from_barcodes.span(),
                            "Conflicts with from_bam".to_string(),
                        ),
                        (
                            tag_to_ref.references_from_bam.span(),
                            "Conflicts with barcodes".to_string(),
                        ),
                    ],
                };
                tag_to_ref.references_from_bam.help =
                    Some("Set only one of 'barcodes' or 'from_bam'.".to_string());
            }
        }

        if self.merge_demultiplexed.as_ref().is_some()
            && let Some(Some(output_segments)) = parent.output.as_ref()
            && output_segments.is_empty()
        {
            let spans = vec![
                (
                    self.merge_demultiplexed.span(),
                    "Incompatible with empty outputs".to_string(),
                ),
                (
                    parent.output.span(),
                    "These output segments are empty".to_string(),
                ),
            ];
            self.merge_demultiplexed.state = TomlValueState::Custom { spans };
            self.merge_demultiplexed.help = Some(
                "Either remove 'merge_demultiplexed' or specify some output segments.".to_string(),
            );
        }

        Ok(())
    }
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
        self.prefix.verify(|prefix| {
            if prefix.contains("/../")
                || prefix.contains("\\..\\")
                || prefix.contains(':')
                || prefix.starts_with('/')
                || prefix.starts_with('\\')
                || prefix.starts_with("../")
                || prefix.starts_with("..\\")
            {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some(
                        "Must not contain '/../', '\\..\\' or ':', nor be an absolute path. \
                        fastqrab only outputs below the current directory.",
                    ),
                ))
            } else if prefix.is_empty() {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not be empty"),
                ))
            } else {
                Ok(())
            }
        });
        self.suffix.verify(|suffix| {
            if let Some(suffix) = suffix.as_ref() {
                if suffix.contains('/') || suffix.contains('\\') || suffix.contains(':') {
                    Err(ValidationFailure::new(
                        "Invalid value",
                        Some("Must not contain '/', '\\' or ':'."),
                    ))
                } else if suffix.is_empty() {
                    Err(ValidationFailure::new(
                        "Invalid value",
                        Some("Must not be empty"),
                    ))
                } else {
                    Ok(())
                }
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
            validate_compression_level_u8(
                &self.compression,
                &mut self.compression_level,
                &self.format.as_ref().unwrap_or(&FileFormat::Fastq),
            );
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
                    "You probably want 'output_hash_compressed=true'. Or disable output_hash_uncompressed, or switch output formats".to_string(),
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
                let spans = vec![
                    (
                        self.stdout.span(),
                        "Conflict with 'output' option".to_string(),
                    ),
                    (
                        self.output.span(),
                        "Conflict with 'stdout' option".to_string(),
                    ),
                ];
                self.stdout.state = TomlValueState::Custom { spans };
                self.stdout.help = Some("Remove either `output` or `stdout` ".to_string());
                return; // Don't auto-set interleave when there's a conflict
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
            } else if let Some(input) = config.input.as_ref() {
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

    fn verify_output_segments(&mut self, config: &super::PartialConfig) {
        if let Some(input) = config.input.as_ref() {
            let valid_segments: HashSet<&String> = input.get_segment_order().iter().collect();

            if let Some(Some(output_segments)) = self.output.as_mut() {
                let mut seen_segments = HashSet::new();
                let mut any_failed = false;
                let all_seen: HashSet<String> = output_segments
                    .iter()
                    .filter_map(|x| x.as_ref())
                    .cloned()
                    .collect();
                for segment in output_segments.iter_mut() {
                    if let Some(segment_str) = segment.as_ref() {
                        if valid_segments.contains(segment_str) {
                            if !seen_segments.insert(segment_str.clone()) {
                                segment.help = Some(format!("Remove all but one '{segment_str}'",));
                                segment.state = TomlValueState::new_validation_failed(
                                    "Segment is duplicated in output segments",
                                );
                                any_failed = true;
                            }
                        } else {
                            let available: Vec<&String> = valid_segments
                                .iter()
                                .filter_map(|x| {
                                    if all_seen.contains(*x) {
                                        None
                                    } else {
                                        Some(*x)
                                    }
                                })
                                .collect();
                            segment.help = Some(offer_alternatives(segment_str, &available));
                            segment.state = TomlValueState::new_validation_failed(
                                "Not found in input segments",
                            );
                            any_failed = true;
                        }
                    } // cov:excl-line
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
                    .cloned()
                    .collect();
                for segment in interleave_order.iter_mut() {
                    if let Some(segment_str) = segment.as_ref() {
                        if valid_segments.contains(segment_str) {
                            if !seen_segments.insert(segment_str.clone()) {
                                segment.help = Some(format!("Remove all but one '{segment_str}'",));
                                segment.state = TomlValueState::new_validation_failed(
                                    "Segment is duplicated in interleave order",
                                );
                                any_failed = true;
                            }
                        } else {
                            let available: Vec<&String> = valid_segments
                                .iter()
                                .filter_map(|x| {
                                    if all_seen.contains(*x) {
                                        None
                                    } else {
                                        Some(*x)
                                    }
                                })
                                .collect();
                            segment.help = Some(offer_alternatives(segment_str, &available));
                            segment.state = TomlValueState::new_validation_failed(
                                "Not found in input segments",
                            );
                            any_failed = true;
                        }
                    } // cov:excl-line
                }
                if any_failed {
                    if matches!(self.interleave.state, TomlValueState::Ok) {
                        self.interleave.state = TomlValueState::Nested;
                    }
                } else if interleave_order.len() < 2 && !*self.stdout.unwrap_ref() {
                    self.interleave.state = TomlValueState::new_validation_failed(
                        "Must contain at least two segments to interleave.",
                    );
                    self.interleave.help = Some("Either add another segment to interleave, or remove interleave, or output to files in stead of stdout".to_string());
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
                                    (segment.span(), "Duplicate output & interleave".to_string()),
                                ];

                                found.state = TomlValueState::Custom { spans };
                                found.help = Some(
                                    "Remove from either 'interleave' or from 'output'".to_string(),
                                );
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
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}

/// Validates that the compression level is within the expected range for the given compression format
pub fn validate_compression_level_u8(
    compression: &TomlValue<CompressionFormat>,
    compression_level: &mut TomlValue<Option<u8>>,
    output_format: &FileFormat,
) {
    if let Some(Some(level)) = compression_level.as_ref() {
        match compression.as_ref() {
            None | Some(CompressionFormat::Uncompressed) => {
                if output_format == &FileFormat::Bam {
                    if *level > 9 || *level < 1 {
                        compression_level.state = TomlValueState::ValidationFailed {
                            message: "Invalid compression level specified for BAM output"
                                .to_string(),
                        };
                        compression_level.help =
                            Some("Valid range is 1-9 for BAM (and our compressor)".to_string());
                    }
                } else {
                    if *level != 0 {
                        compression_level.state = TomlValueState::ValidationFailed {
                            message: "Compression level specified for uncompressed output"
                                .to_string(),
                        };
                        compression_level.help = Some(
                            "Remove compression_level, or set compressed='gzip' or 'zstd'"
                                .to_string(),
                        );
                    }
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
        }
    } else {
        // No compression level specified - rapidgzip is still invalid for output
    }
}
