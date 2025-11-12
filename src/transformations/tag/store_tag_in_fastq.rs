#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

use bstr::BString;
use std::io::Write;
use std::path::Path;

use crate::{
    config::{
        deser::{bstring_from_string, u8_from_char_or_number},
        CompressionFormat, FileFormat, SegmentIndexOrAll, SegmentOrAll,
    },
    dna::TagValue,
    io::output::compressed_output::HashedAndCompressedWriter,
};

use super::{
    default_comment_insert_char, default_comment_separator, default_region_separator,
    default_segment_all, format_numeric_for_comment, store_tag_in_comment,
};

/// Store reads with specific tag values into separate FASTQ files.
/// Creates one FASTQ file per unique tag value found during processing.
///
/// Files are named using the pattern: `{output_prefix}_{infix}.tag.{tag_value}.fastq.{suffix}`
/// When demultiplexing: `{output_prefix}_{infix}_{barcode}.tag.{tag_value}.fastq.{suffix}`
///
/// Optionally adds comment tags to read names before writing, similar to `StoreTagInComment`.
#[derive(eserde::Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInFastQ {
    in_label: String,
    #[serde(default)]
    infix: String,
    #[serde(default = "default_segment_all")]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,

    // Optional read name comment fields (like StoreTagInComment)
    #[serde(default)]
    comment_tags: Vec<String>,
    // Optional location tags to add to read names
    #[serde(default)]
    comment_location_tags: Option<Vec<String>>,

    #[serde(default = "default_comment_separator")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_separator: u8,
    #[serde(default = "default_comment_insert_char")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_insert_char: u8,
    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    region_separator: BString,

    // Optional format configuration (defaults to Raw)
    #[serde(default)]
    format: FileFormat,
    #[serde(default)]
    compression: CompressionFormat,
    #[serde(default)]
    compression_level: Option<u8>,
    #[serde(default)]
    #[serde(skip)]
    ix_separator: String,

    // Internal state for collecting reads during apply
    #[serde(default)]
    #[serde(skip)]
    output_streams:
        DemultiplexedData<Option<Box<HashedAndCompressedWriter<'static, ex::fs::File>>>>,
}

impl Clone for StoreTagInFastQ {
    fn clone(&self) -> Self {
        StoreTagInFastQ {
            in_label: self.in_label.clone(),
            infix: self.infix.clone(),
            segment: self.segment.clone(),
            segment_index: self.segment_index.clone(),
            comment_tags: self.comment_tags.clone(),
            comment_location_tags: self.comment_location_tags.clone(),
            comment_separator: self.comment_separator,
            comment_insert_char: self.comment_insert_char,
            region_separator: self.region_separator.clone(),
            format: self.format,
            compression: self.compression,
            compression_level: self.compression_level,
            ix_separator: self.ix_separator.clone(),
            output_streams: DemultiplexedData::new(), // Do not clone output streams
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for StoreTagInFastQ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagInFastQ")
            .field("label", &self.in_label)
            .field("infix", &self.infix)
            .field("segment", &self.segment)
            .field("segment_index", &self.segment_index)
            .field("comment_tags", &self.comment_tags)
            .field("comment_location_tags", &self.comment_location_tags)
            .field("comment_separator", &self.comment_separator)
            .field("comment_insert_char", &self.comment_insert_char)
            .field("region_separator", &self.region_separator)
            .field("format", &self.format)
            .field("compression", &self.compression)
            .field("compression_level", &self.compression_level)
            .finish()
    }
}

impl Step for StoreTagInFastQ {
    fn needs_serial(&self) -> bool {
        true
    }
    fn move_inited(&mut self) -> Self {
        assert!(self.output_streams.len() > 0);
        let mut new = self.clone();
        new.output_streams = std::mem::replace(&mut self.output_streams, new.output_streams);
        new
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        output_def: Option<&crate::config::Output>,
        all_transforms: &[super::super::Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        // Check if output configuration is present
        if output_def.is_none() {
            bail!(
                "StoreTagInFastQ requires output configuration to determine file paths and formats"
            );
        }

        crate::config::validate_compression_level_u8(self.compression, self.compression_level)?;

        if !matches!(self.format, FileFormat::Fastq | FileFormat::Fasta) {
            bail!(
                "StoreTagInFastQ supports only 'fastq' or 'fasta' formats. Received: {:?}",
                self.format
            );
        }

        if self.in_label.is_empty() || self.in_label.trim().is_empty() {
            bail!("Tag name may not be empty (or just whitespace)");
        }
        if self.in_label.contains('/') || self.in_label.contains('\\') {
            bail!(
                "Tag name may not contain path separators like / and \\. Was '{}'",
                self.in_label
            );
        }
        if self.in_label.chars().any(|c| c.is_ascii_control()) {
            bail!(
                "Tag name may not contain control characters. {:?}",
                self.in_label
            );
        }

        // Check that there's only one StoreTagInFastQ using this tag
        for (idx, transform) in all_transforms.iter().enumerate() {
            if idx != this_transforms_index {
                if let crate::transformations::Transformation::StoreTagInFastQ(other) = transform {
                    if other.in_label == self.in_label {
                        bail!(
                            "Only one StoreTagInFastQ step per tag is allowed. Tag '{}' is used by multiple StoreTagInFastQ steps",
                            self.in_label
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        if self.comment_location_tags.is_none() {
            self.comment_location_tags = Some(vec![self.in_label.clone()]);
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        let mut tags: Vec<(String, &[TagValueType])> =
            vec![(self.in_label.clone(), &[TagValueType::Location])];
        tags.extend(self.comment_tags.iter().map(|x| {
            (
                x.clone(),
                &[
                    TagValueType::String,
                    TagValueType::Location,
                    TagValueType::Bool,
                    TagValueType::Numeric,
                ][..],
            )
        }));

        // Add location tags (deduplicated) - defaults to main label if not specified
        if let Some(location_tags) = self.comment_location_tags.as_ref() {
            for tag in location_tags {
                if !tags.iter().any(|(name, _)| name == tag) {
                    tags.push((tag.clone(), &[TagValueType::Location]));
                }
            }
        }
        Some(tags)
    }

    fn configure_output_separator(&mut self, ix_separator: &str) {
        self.ix_separator = ix_separator.to_string();
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        self.output_streams = demultiplex_info.open_output_streams(
            output_directory,
            output_prefix,
            &format!("tag.{}", self.in_label),
            self.format.default_suffix(),
            output_ix_separator,
            self.compression,
            None,
            false,
            false,
            allow_overwrite,
        )?;
        println!("init {}", self.output_streams.len());
        Ok(None)
    }

    #[allow(clippy::too_many_lines)]
    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        println!("apply {}", self.output_streams.len());
        let tags = block
            .tags
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Expected tags to be present"))?;

        let mut error_encountered = None;

        'outer: for (ii, tag) in &mut tags.get(&self.in_label).unwrap().iter().enumerate() {
            //presence & tag = location checked before hand.
            if let Some(tag) = tag.as_sequence() {
                let seq = tag.0.iter().fold(Vec::new(), |mut acc, hit| {
                    if !acc.is_empty() {
                        acc.extend_from_slice(&self.region_separator);
                    }
                    acc.extend_from_slice(&hit.sequence);
                    acc
                });
                if !seq.is_empty() {
                    let qual = vec![b'~'; seq.len()]; // Dummy quality scores
                    let segment_block =
                        &block.segments[tag.0[0].location.as_ref().unwrap().segment_index.0];
                    let wrapped = segment_block.get(ii);

                    // Determine which output stream to use based on demultiplexing
                    let output_idx = block.output_tags.as_ref().map(|x| x[ii]).unwrap_or(0);

                    if let Some(writer) = self.output_streams.get_mut(&output_idx).unwrap() {
                        //if we have demultiplex & no-unmatched-output, this happens
                        let mut name = wrapped.name().to_vec();
                        for tag in &self.comment_tags {
                            if let Some(tag_value) = tags.get(tag).unwrap().get(ii) {
                                let tag_bytes: Vec<u8> = match tag_value {
                                    TagValue::Location(hits) => {
                                        hits.joined_sequence(Some(&self.region_separator))
                                    }
                                    TagValue::String(value) => value.to_vec(),
                                    TagValue::Numeric(n) => {
                                        format_numeric_for_comment(*n).into_bytes()
                                    }
                                    TagValue::Bool(n) => {
                                        if *n {
                                            "1".into()
                                        } else {
                                            "0".into()
                                        }
                                    }
                                    TagValue::Missing => Vec::new(),
                                };
                                let new_name = store_tag_in_comment(
                                    &name,
                                    tag.as_bytes(),
                                    &tag_bytes,
                                    self.comment_separator,
                                    self.comment_insert_char,
                                );
                                match new_name {
                                    Err(err) => {
                                        error_encountered = Some(format!("{err}"));
                                        break 'outer;
                                    }
                                    Ok(new_name) => {
                                        name = new_name;
                                    }
                                }
                            }
                        }

                        // Process location tags - always set by validation logic.
                        for location_tag in self.comment_location_tags.as_ref().unwrap() {
                            if let Some(tag_value) = tags.get(location_tag).unwrap().get(ii) {
                                if let Some(hits) = tag_value.as_sequence() {
                                    let mut location_seq: Vec<u8> = Vec::new();
                                    let mut first = true;
                                    for hit in &hits.0 {
                                        if let Some(location) = hit.location.as_ref() {
                                            if !first {
                                                location_seq.push(b',');
                                            }
                                            first = false;
                                            location_seq.extend_from_slice(
                                                format!(
                                                    "{}:{}-{}",
                                                    input_info.segment_order
                                                        [location.segment_index.get_index()],
                                                    location.start,
                                                    location.start + location.len
                                                )
                                                .as_bytes(),
                                            );
                                        }
                                    }

                                    if !location_seq.is_empty() {
                                        let location_label = format!("{location_tag}_location");
                                        let new_name = store_tag_in_comment(
                                            &name,
                                            location_label.as_bytes(),
                                            &location_seq,
                                            self.comment_separator,
                                            self.comment_insert_char,
                                        );
                                        match new_name {
                                            Err(err) => {
                                                error_encountered = Some(format!("{err}"));
                                                break 'outer;
                                            }
                                            Ok(new_name) => {
                                                name = new_name;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        match self.format {
                            FileFormat::Fastq => {
                                writer.write_all(b"@")?;
                                writer.write_all(&name)?;
                                writer.write_all(b"\n")?;
                                writer.write_all(&seq)?;
                                writer.write_all(b"\n+\n")?;
                                writer.write_all(&qual)?;
                                writer.write_all(b"\n")?;
                            }
                            FileFormat::Fasta => {
                                writer.write_all(b">")?;
                                writer.write_all(&name)?;
                                writer.write_all(b"\n")?;
                                writer.write_all(&seq)?;
                                writer.write_all(b"\n")?;
                            }
                            FileFormat::Bam | FileFormat::None => {
                                unreachable!("Unsupported format encountered after validation")
                            }
                        }
                    }
                }
            }
        }
        if let Some(error_msg) = error_encountered {
            return Err(anyhow::anyhow!("{error_msg}"));
        }

        Ok((block, true))
    }

    fn finalize(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<crate::transformations::FinalizeReportResult>> {
        // Flush all output streams
        let output_streams = std::mem::replace(&mut self.output_streams, DemultiplexedData::new());
        for (_tag, writer) in output_streams.into_iter() {
            if let Some(writer) = writer {
                let (_, _) = writer.finish();
            }
            // Finalize the writer to ensure all data is flushed and hashes are computed
        }

        Ok(None)
    }
}
