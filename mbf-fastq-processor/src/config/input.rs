use std::collections::{BTreeMap, HashSet};

use anyhow::Result;
use indexmap::IndexMap;
use schemars::JsonSchema;
use std::collections::HashMap;
use toml_pretty_deser::{Visitor, prelude::*};

use crate::config::deser::tpd_adapt_u8_from_byte_or_char;

use super::deser::{self};
use super::validate_segment_label;

fn is_default(opt: &InputOptions) -> bool {
    opt.fasta_fake_quality.is_none()
        && opt.bam_include_mapped.is_none()
        && opt.bam_include_unmapped.is_none()
        && opt.read_comment_character == deser::default_comment_insert_char()
}

pub const STDIN_MAGIC_PATH: &str = "--stdin--";

/// Input configuration
#[derive(serde::Serialize)]
#[tpd]
#[derive(Debug, Clone, JsonSchema)]
pub struct Input {
    /// whether you have input files with interleaved reads, or one file per segment
    /// If interleaved, define the name of the segments here.
    #[tpd(default)] // todo, with="tpd_adapt_trim_string")]
    interleaved: Option<Vec<String>>,

    /// Your segments. Define just one with any name for interlaveed input.
    #[schemars(with = "BTreeMap<String, Vec<String>>")]
    #[tpd(absorb_remaining)]
    #[serde(flatten)]
    segments: IndexMap<String, Vec<String>>,

    #[tpd(nested)]
    #[serde(skip_serializing_if = "is_default")]
    pub options: InputOptions,

    #[tpd(skip)]
    #[schemars(skip)]
    #[serde(skip_serializing)]
    pub structured: StructuredInput,
    // #[tpd(skip)]
    // #[serde(skip_serializing)]
    // pub stdin_stream: bool,
}

impl PartialInput {
    fn verify_same_number_of_input_segments(&mut self) {
        //first me make sure all segments have the same number of files
        if let Some(segments) = self.segments.as_ref() {
            let no_of_file_per_segment: BTreeMap<_, _> = segments
                .map
                .iter()
                .map(|(k, v)| (k, v.value.as_ref().expect("Parent was ok?").len()))
                .collect();
            let observed_no_of_segments: HashSet<_> = no_of_file_per_segment.values().collect();
            if observed_no_of_segments.len() > 1 {
                let spans: Vec<(std::ops::Range<usize>, String)> = segments
                    .map
                    .iter()
                    .map(|(_k, v)| {
                        (
                            v.span.clone(),
                            format!(
                                "{} segment(s)",
                                v.value.as_ref().expect("parent was ok?").len()
                            ),
                        )
                    })
                    .collect();
                self.segments.state = TomlValueState::Custom { spans };
                self.segments.help =
                    Some("Each segment must have the same number of files.".to_string());
            }
        }
    }

    fn verify_segment_names(&mut self, match_mode: toml_pretty_deser::FieldMatchMode) -> Result<(), ()> {
        let mut error = false;
        if let Some(segments) = self.segments.as_mut() {
            for key in &mut segments.keys {
                let segment_name = key.as_ref().expect("parent was ok");
                match validate_segment_label(segment_name, match_mode) {
                    Ok(()) => {}

                    Err(help) => {
                        key.state = TomlValueState::ValidationFailed {
                            message: "Invalid segment name".to_string(),
                        };
                        key.help = Some(help.to_string());
                        error = true;
                    }
                }
            }
        }
        //duplicate names can't happen here, it's a map

        if error { Err(()) } else { Ok(()) }
    }

    fn validate_stdin_usage(&mut self) -> Result<(), ()> {
        // let Some(structured) = self.structured.as_ref() else {
        //     return Ok(());
        // };
        match &self.structured {
            Some(StructuredInput::Interleaved { files, .. }) => {
                if files.iter().any(|f| f == STDIN_MAGIC_PATH) {
                    if files.len() != 1 {
                        self.interleaved.state = TomlValueState::ValidationFailed {
                            message: "Invalid use of stdin magic value".to_string(),
                        };
                        self.interleaved.help = Some(format!(
                            "When using '{STDIN_MAGIC_PATH}' as an input file, it must be the only file listed in the interleaved segment's input. Found {} times.",
                            files.len()
                        ));
                    }
                    return Ok(());
                }
            }
            Some(StructuredInput::Segmented {
                segment_files,
                segment_order,
            }) => {
                let segments_with_stdin: Vec<_> = segment_order
                    .iter()
                    .filter(|segment| {
                        segment_files
                            .get(*segment)
                            .is_some_and(|files| files.iter().any(|name| name == STDIN_MAGIC_PATH))
                    })
                    .collect();
                if segments_with_stdin.is_empty() {
                    return Ok(());
                }
                if segments_with_stdin.len() > 1 {
                    let spans: Vec<_> = self
                        .segments
                        .as_ref()
                        .expect("segments must exist")
                        .keys
                        .iter()
                        .filter(|toml_value| {
                            segments_with_stdin
                                .contains(&toml_value.as_ref().expect("parent was ok"))
                        })
                        .map(|x| {
                            (
                                x.span().clone(),
                                "Invalid use of {STDIN_MAGIC_PATH}".to_string(),
                            )
                        })
                        .collect();
                    self.segments.state = TomlValueState::Custom { spans };
                    self.segments.help = Some(format!(
                        "When using '{STDIN_MAGIC_PATH}' as an input file, it must be the only file listed in exactly one segment. Found in segments: {segments_with_stdin:?}"
                    ));
                    return Err(());
                }
                if segment_order.len() != 1 {
                    self.segments.state = TomlValueState::ValidationFailed {
                        message: "Invalid use of stdin magic value".to_string(),
                    };
                    self.segments.help = Some(format!(
                        "Using '{STDIN_MAGIC_PATH}' requires exactly one segment (and possibly interleaved)."
                    ));
                }
                let segment = segments_with_stdin[0];
                let files = segment_files.get(segment).expect("segment must exist");
                if files.len() != 1 {
                    self.segments.state = TomlValueState::ValidationFailed {
                        message: "Invalid use of stdin magic value".to_string(),
                    };
                    self.segments.help =
                        Some("'{STDIN_MAGIC_PATH}' requires exactly one input file.".to_string());
                }
                if files[0] != STDIN_MAGIC_PATH {
                    self.segments.state = TomlValueState::ValidationFailed {
                        message: "Invalid use of stdin magic value".to_string(),
                    };
                    self.segments.help = Some(format!(
                        "When using '{STDIN_MAGIC_PATH}' as an input file, it must be the only file listed in the segment. Found additional files: {files:?}"
                    ));
                }
            }
            None => {
                //no segments, no further checking :)
            }
        }
        Ok(())
    }
    #[allow(clippy::too_many_lines)] // I know, but it's two distingt if branches, 
    fn build_structured(&mut self, match_mode: FieldMatchMode) -> Result<(), ()> {
        if let Err(()) = self.verify_segment_names(match_mode) {
            self.segments.state = TomlValueState::Nested;
        }

        //We need to make it not fail on unset structrude
        //todo: refactor, but the borrow checking is annoying.
        if let Some(Some(interleaved)) = self.interleaved.as_mut()
            && let Some(segments) = self.segments.as_ref()
        {
            if segments.map.len() != 1 {
                let mut spans: Vec<_> = segments
                    .map
                    .iter()
                    .map(|(_k, v)| (v.span.clone(), "More than one segment defined".to_string()))
                    .collect();
                spans.push((
                    self.interleaved.span.clone(),
                    "Interleaved segment definition".to_string(),
                ));
                self.interleaved.state = TomlValueState::Custom { spans };
                self.interleaved.help = Some(
                    "Interleaved input can only have exactly one other key defining the segments."
                        .to_string(),
                );
                return Err(());
            }
            if interleaved.len() < 2 {
                self.interleaved.state = TomlValueState::ValidationFailed {
                    message: "Must define at least two segments".to_string(),
                };
                self.interleaved.help = Some(
                    "If you have single end reads, remove interleaved.
If you have paired end reads, name two 'virtual' segments, e.g. ['read1','read2']"
                        .to_string(),
                );
                return Err(());
            }
            //detect duplicate names in interleaved
            let mut seen: HashMap<String, Vec<std::ops::Range<usize>>> = HashMap::new();
            for segment_toml_value in interleaved.iter() {
                let segment_name = segment_toml_value.as_ref().expect("parent was ok").trim();
                match seen.entry(segment_name.to_string()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(vec![segment_toml_value.span.clone()]);
                    }
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        e.get_mut().push(segment_toml_value.span.clone());
                    }
                }
            }
            let mut reported = HashSet::new();
            for segment_toml_value in interleaved.iter_mut() {
                let segment_name = segment_toml_value.as_ref().expect("parent was ok").clone();
                if reported.insert(segment_name.clone()) {
                    let spans = seen.get(&segment_name).expect("We just built this map");
                    if spans.len() > 1 {
                        segment_toml_value.state = TomlValueState::Custom {
                            spans: spans
                                .iter()
                                .map(|span| (span.clone(), "Duplicate value".to_string()))
                                .collect(),
                        };
                        segment_toml_value.help = Some(
                            "Use each segment only once in interleaved. If you really want to use the same reads twice, define multiple segments, set input.accept_duplicate_files = true.".to_string()
                        );
                    }
                }
            }
            if !interleaved.can_concrete() {
                self.interleaved.state = TomlValueState::Nested;
                return Err(());
            }

            let files: Vec<String> = segments
                .map
                .values()
                .next()
                .expect("We ensured there was at least one segment")
                .as_ref()
                .expect("parent was ok")
                .iter()
                .map(|tv| tv.as_ref().expect("parent was ok?").clone())
                .collect();

            self.structured = Some(StructuredInput::Interleaved {
                files,
                segment_order: interleaved
                    .iter()
                    .map(|x| x.as_ref().expect("parent was ok").trim().to_string())
                    .collect(),
            });
        } else if let Some(segments) = self.segments.as_mut() {
            let mut segment_order: Vec<String> =
                segments.map.keys().map(|x| x.trim().to_string()).collect();
            segment_order.sort(); //always alphabetical...
            if segment_order.is_empty() {
                self.segments.state = TomlValueState::ValidationFailed {
                    message: "No segments defined in input.".to_string(),
                };
                self.segments.help = Some(
                    "At least one segment must be defined. Example: read1 = ['filename.fq']"
                        .to_string(),
                );
                return Err(());
            }
            if let Some(all_segment) = segments.keys.iter_mut().find(|tv| {
                tv.as_ref()
                    .expect("Parent was ok")
                    .eq_ignore_ascii_case("all")
            }) {
                all_segment.state = TomlValueState::ValidationFailed {
                    message: "Reserved segment name".to_string(),
                };
                all_segment.help = Some(
                    "Segment name 'all' is reserved and cannot be used as a segment name."
                        .to_string(),
                );
                self.segments.state = TomlValueState::Nested;
                return Err(());
            }
            if let Some(all_segment) = segments.keys.iter_mut().find(|tv| {
                tv.as_ref()
                    .expect("Parent was ok")
                    .to_ascii_lowercase()
                    .starts_with("_internal_")
            }) {
                all_segment.state = TomlValueState::ValidationFailed {
                    message: "Reserved segment name".to_string(),
                };
                all_segment.help = Some(
                    "Segment names starting with '_internal_' are reserved and cannot be used as a segment name. Choose something else."
                        .to_string(),
                );
                self.segments.state = TomlValueState::Nested;
                return Err(());
            }

            assert!(
                !segment_order
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case("options")),
                "Options should have been filtered by toml-pretty-deser"
            );
            let segment_files: IndexMap<String, Vec<String>> = segments
                .map
                .iter()
                .map(|(k, v)| {
                    let files = v
                        .as_ref()
                        .expect("Parent was ok")
                        .iter()
                        .map(|tv| tv.as_ref().expect("parent was ok?").clone())
                        .collect();
                    (k.trim().to_string(), files)
                })
                .collect();
            self.structured = Some(StructuredInput::Segmented {
                segment_files,
                segment_order,
            });
        }
        self.validate_stdin_usage()?;

        Ok(())
    }
}

impl VerifyIn<super::PartialConfig> for PartialInput {
    fn verify(
        &mut self,
        parent: &super::PartialConfig,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.options.or_with(|| {
            //this could be prettier...
            let default = InputOptions::default();
            PartialInputOptions {
                fasta_fake_quality: TomlValue::new_ok(default.fasta_fake_quality, 0..0),
                bam_include_mapped: TomlValue::new_ok(default.bam_include_mapped, 0..0),
                bam_include_unmapped: TomlValue::new_ok(default.bam_include_unmapped, 0..0),
                read_comment_character: TomlValue::new_ok(default.read_comment_character, 0..0),
                use_rapidgzip: TomlValue::new_ok(default.use_rapidgzip, 0..0),
                build_rapidgzip_index: TomlValue::new_ok(default.build_rapidgzip_index, 0..0),
                threads_per_segment: TomlValue::new_ok(default.threads_per_segment, 0..0),
                tpd_field_match_mode: parent.tpd_field_match_mode
            }
        });

        self.verify_same_number_of_input_segments();

        if let Err(()) = self.build_structured(parent.tpd_field_match_mode)
        //errors go into the fields
        {}

        Ok(())
    }
}

#[derive(serde::Serialize, Clone, PartialEq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct InputOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    // #[validate(minimum = 33)] TODO
    // #[validate(maximum = 126)] TODO
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub fasta_fake_quality: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_mapped: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_unmapped: Option<bool>,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub read_comment_character: u8,

    #[serde(skip_serializing)]
    pub use_rapidgzip: Option<bool>,

    #[serde(skip_serializing)]
    pub build_rapidgzip_index: Option<bool>,

    pub threads_per_segment: Option<usize>,
}

impl VerifyIn<PartialInput> for PartialInputOptions {
    fn verify(&mut self, _parent: &PartialInput) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.fasta_fake_quality.verify(|opt_v| {
            if let Some(v) = opt_v {
                if *v >= 33 && *v <= 126 {
                    Ok(())
                } else {
                    Err(ValidationFailure::new(
                        "Out of PHRED range (33..126)",
                        Some("'B' might be a good value"),
                    ))
                }
            } else {
                Ok(())
            }
        });
        self.read_comment_character
            .or_with(deser::default_comment_insert_char);

        // Validate index_gzip option
        if let Some(Some(true)) = self.build_rapidgzip_index.as_ref()
            && let Some(Some(false)) = self.use_rapidgzip.as_ref()
        {
            self.build_rapidgzip_index.state = TomlValueState::ValidationFailed {
                message: "Only valid when use_rapidgzip is set to true".to_string(),
            };
            self.build_rapidgzip_index.help =
                Some("Either set use_rapidgzip=true or unset build_rapidgzip_index".to_string());
        }

        Ok(())
    }
}

impl Default for InputOptions {
    fn default() -> Self {
        InputOptions {
            fasta_fake_quality: None,
            bam_include_mapped: None,
            bam_include_unmapped: None,
            read_comment_character: deser::default_comment_insert_char(),
            use_rapidgzip: None,
            build_rapidgzip_index: None,
            threads_per_segment: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StructuredInput {
    Interleaved {
        files: Vec<String>,
        segment_order: Vec<String>,
    },
    Segmented {
        segment_files: IndexMap<String, Vec<String>>,
        segment_order: Vec<String>,
    },
}

impl StructuredInput {
    #[must_use]
    pub fn is_interleaved(&self) -> bool {
        matches!(self, StructuredInput::Interleaved { .. })
    }
}

impl Input {
    #[must_use]
    pub fn is_interleaved(&self) -> bool {
        self.interleaved.is_some()
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        match &self.structured {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    #[must_use]
    #[mutants::skip] // only used to figure out thread count.
    pub fn parser_count(&self) -> usize {
        match &self.structured {
            StructuredInput::Interleaved { .. } => 1,
            StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    #[must_use]
    pub fn get_segment_order(&self) -> &Vec<String> {
        match &self.structured {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order,
        }
    }

    #[must_use]
    pub fn index(&self, segment_name: &str) -> Option<usize> {
        match &self.structured {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => {
                segment_order.iter().position(|s| s == segment_name)
            }
        }
    }
}

impl PartialInput {
    #[must_use]
    pub fn get_segment_order(&self) -> &Vec<String> {
        match self
            .structured
            .as_ref()
            .expect("structured input must be set after config parsing")
        {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
#[tpd]
pub enum CompressionFormat {
    #[tpd(alias = "uncompressed")]
    #[tpd(alias = "raw")]
    #[default]
    Uncompressed,
    #[tpd(alias = "gzip")]
    #[tpd(alias = "gz")]
    Gzip,
    #[tpd(alias = "zstd")]
    #[tpd(alias = "zst")]
    Zstd,
}

impl CompressionFormat {
    #[must_use]
    pub fn is_compressed(&self) -> bool {
        !matches!(self, CompressionFormat::Uncompressed)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, JsonSchema)]
#[tpd]
pub enum FileFormat {
    #[default]
    Fastq,
    Fasta,
    Bam,
    None,
}

impl FileFormat {
    #[must_use]
    pub fn default_suffix(&self) -> &'static str {
        match self {
            FileFormat::Fastq => "fq",
            FileFormat::Fasta => "fasta",
            FileFormat::Bam => "bam",
            FileFormat::None => "",
        }
    }

    #[must_use]
    pub fn get_suffix(
        &self,
        compression: CompressionFormat,
        custom_suffix: Option<&String>,
    ) -> String {
        if let Some(custom) = custom_suffix {
            return custom.clone();
        }

        match self {
            FileFormat::Fastq | FileFormat::Fasta => {
                let base = self.default_suffix();
                compression.apply_suffix(base)
            }
            FileFormat::Bam => self.default_suffix().to_string(),
            FileFormat::None => String::new(),
        }
    }
}

impl CompressionFormat {
    #[must_use]
    pub fn apply_suffix(self, base: &str) -> String {
        match self {
            CompressionFormat::Uncompressed => base.to_string(),
            CompressionFormat::Gzip => format!("{base}.gz"),
            CompressionFormat::Zstd => format!("{base}.zst"),
        }
    }
}
