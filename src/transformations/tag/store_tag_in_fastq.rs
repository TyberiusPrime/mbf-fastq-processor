#![allow(clippy::unnecessary_wraps)]
use anyhow::{bail, Result};
use bstr::BString;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::transformations::TagValueType;
use crate::{
    config::{
        deser::{bstring_from_string, u8_from_char_or_number},
        FileFormat, SegmentIndexOrAll, SegmentOrAll,
    },
    dna::TagValue,
    output::HashedAndCompressedWriter,
    Demultiplexed,
};

use super::super::Step;
use super::{
    default_comment_insert_char, default_comment_separator, default_region_separator,
    default_segment_all, format_numeric_for_comment, store_tag_in_comment,
};

/// Store reads with specific tag values into separate FASTQ files.
/// Creates one FASTQ file per unique tag value found during processing.
///
/// Files are named using the pattern: `{output_prefix}.tag.{tag_value}.fastq.{suffix}`
///
/// Optionally adds comment tags to read names before writing, similar to `StoreTagInComment`.
#[derive(eserde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInFastQ {
    label: String,
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
    region_separator: BString,

    // Optional format configuration (defaults to Raw)
    #[serde(default)]
    format: FileFormat,
    #[serde(default)]
    compression_level: Option<u8>,

    // Internal state for collecting reads during apply
    #[serde(default)]
    #[serde(skip)]
    output_stream: Option<Arc<Mutex<dyn std::io::Write + Send>>>,
}

impl StoreTagInFastQ {}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for StoreTagInFastQ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagInFastQ")
            .field("label", &self.label)
            .field("segment", &self.segment)
            .field("segment_index", &self.segment_index)
            .field("comment_tags", &self.comment_tags)
            .field("comment_location_tags", &self.comment_location_tags)
            .field("comment_separator", &self.comment_separator)
            .field("comment_insert_char", &self.comment_insert_char)
            .field("region_separator", &self.region_separator)
            .field("format", &self.format)
            .field("compression_level", &self.compression_level)
            .finish()
    }
}

impl Step for StoreTagInFastQ {
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

        crate::config::validate_compression_level_u8(self.format, self.compression_level)?;

        if self.label.is_empty() || self.label.trim().is_empty() {
            bail!("Tag name may not be empty (or just whitespace)");
        }
        if self.label.contains('/') || self.label.contains('\\') {
            bail!(
                "Tag name may not contain path separators like / and \\. Was '{}'",
                self.label
            );
        }
        if self.label.chars().any(|c| (c.is_ascii_control())) {
            bail!(
                "Tag name may not contain control characters. {:?}",
                self.label
            );
        }
        crate::transformations::filters::validate_tag_set_and_type(
            all_transforms,
            this_transforms_index,
            &self.label,
            TagValueType::Location,
        )?;

        // For comment tags, verify they exist
        for comment_tag in &self.comment_tags {
            crate::transformations::filters::validate_tag_set(
                all_transforms,
                this_transforms_index,
                comment_tag,
            )?;
        }

        for location_tag in self.comment_location_tags.as_ref().unwrap() {
            // always set by validate_segment
            crate::transformations::filters::validate_tag_set(
                all_transforms,
                this_transforms_index,
                location_tag,
            )?;
        }

        // Check that there's only one StoreTagInFastQ using this tag
        for (idx, transform) in all_transforms.iter().enumerate() {
            if idx != this_transforms_index {
                if let crate::transformations::Transformation::StoreTagInFastQ(other) = transform {
                    if other.label == self.label {
                        bail!(
                            "Only one StoreTagInFastQ step per tag is allowed. Tag '{}' is used by multiple StoreTagInFastQ steps",
                            self.label
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
            self.comment_location_tags = Some(vec![self.label.clone()]);
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        let mut tags = vec![self.label.clone()];
        tags.extend(self.comment_tags.clone());

        // Add location tags (deduplicated) - defaults to main label if not specified
        if let Some(location_tags) = self.comment_location_tags.as_ref() {
            for tag in location_tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
        Some(tags)
    }

    fn init(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        // Store output configuration for file creation
        let filename = output_directory.join(format!(
            "{output_prefix}.tag.{label}.{suffix}",
            label = self.label,
            suffix = self.format.get_suffix(None)
        ));
        //make sure the file is writable
        self.output_stream = {
            let file_handle = std::fs::File::create(&filename)?;
            let writer = HashedAndCompressedWriter::new(
                file_handle,
                self.format,
                false, // hash_uncompressed
                false, // hash_compressed
                self.compression_level,
            )?;
            Some(Arc::new(Mutex::new(writer)))
        };
        Ok(None)
    }

    #[allow(clippy::too_many_lines)]
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<(crate::io::FastQBlocksCombined, bool)> {
        //todo
        let tags = block
            .tags
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Expected tags to be present"))?;

        let mut writer = self
            .output_stream
            .as_ref()
            .expect("output stream not set")
            .lock()
            .expect("failed to lock");
        let mut error_encountered = None;

        'outer: for (ii, tag) in &mut tags.get(&self.label).unwrap().iter().enumerate() {
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

                    let mut name = wrapped.name().to_vec();
                    for tag in &self.comment_tags {
                        if let Some(tag_value) = tags.get(tag).unwrap().get(ii) {
                            let tag_bytes: Vec<u8> = match tag_value {
                                TagValue::Sequence(hits) => {
                                    hits.joined_sequence(Some(&self.region_separator))
                                }
                                TagValue::Numeric(n) => format_numeric_for_comment(*n).into_bytes(),
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
                    writer.write_all(b"@")?;
                    writer.write_all(&name)?;
                    writer.write_all(b"\n")?;
                    writer.write_all(&seq)?;
                    writer.write_all(b"\n+\n")?;
                    writer.write_all(&qual)?;
                    writer.write_all(b"\n")?;
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
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::transformations::FinalizeReportResult>> {
        // Write all collected reads to their respective files
        if let Some(writer) = self.output_stream.take() {
            writer.lock().unwrap().flush().unwrap();
            // Finalize the writer to ensure all data is flushed and hashes are computed
            //  let (_uncompressed_hash, _compressed_hash) = writer.finish();
            // Optionally, log or store the hashes if needed
        }

        Ok(None)
    }
}
