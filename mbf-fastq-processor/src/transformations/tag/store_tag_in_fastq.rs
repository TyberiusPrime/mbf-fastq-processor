#![allow(clippy::unnecessary_wraps)]
use crate::{
    config::deser::{tpd_adapt_bstring, tpd_adapt_u8_from_byte_or_char},
    transformations::prelude::*,
};

use std::io::Write;

use crate::{
    config::{CompressionFormat, FileFormat},
    dna::TagValue,
};

use super::{
    default_comment_insert_char, default_comment_separator, default_region_separator,
    format_numeric_for_comment, store_tag_in_comment,
};

/// Store tag values into FASTQ files.
///
/// Files are named using the pattern: `{output_prefix}_{infix}.tag.fastq.{suffix}`
/// When demultiplexing: `{output_prefix}_{infix}_{barcode}.tag.fastq.{suffix}`
///
/// Optionally adds comment tags to read names before writing, similar to `StoreTagInComment`.
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct StoreTagInFastQ {
    in_label: String,

    // Optional read name comment fields (like StoreTagInComment)
    #[tpd(default)]
    comment_tags: Vec<String>,
    //
    // Optional location tags to add to read names
    //#[tpd(default)]
    comment_location_tags: Vec<String>,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    comment_separator: u8,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    comment_insert_char: u8,

    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    region_separator: BString,

    // Optional format configuration (defaults to Raw)
    #[tpd(default)]
    format: FileFormat,
    #[tpd(default)]
    compression: CompressionFormat,
    #[tpd(default)]
    compression_level: Option<u8>,

    // Internal state for collecting reads during apply
    #[tpd(skip, default)]
    #[schemars(skip)]
    output_streams: Option<Arc<Mutex<DemultiplexedOutputFiles>>>,
}

impl VerifyIn<PartialConfig> for PartialStoreTagInFastQ {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.comment_separator.or_with(default_comment_separator);

        self.comment_insert_char
            .or_with(default_comment_insert_char);

        self.region_separator.or_with(default_region_separator);

        self.format.verify(|format| {
            if !matches!(format, FileFormat::Fastq | FileFormat::Fasta) {
                return Err(ValidationFailure::new(
                    "StoreTagInFastQ supports only 'fastq' or 'fasta' formats",
                    None,
                ));
            }
            Ok(())
        });

        crate::config::validate_compression_level_u8(
            &self.compression,
            &mut self.compression_level,
        );

        if parent.output.is_missing() {
            return Err(ValidationFailure::new(
                "Missing output configuration",
                Some(
                    "StoreTagInFastQ requires output configuration to determine file paths and formats",
                ),
            ));
        }
        if let Some(in_label) = self.in_label.as_ref() {
            self.comment_location_tags
                .or_with(|| vec![TomlValue::new_ok(in_label.clone(), 0..0)]);
        }
        Ok(())
    }
}

/* #[allow(clippy::missing_fields_in_debug)]
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
} */

impl Step for StoreTagInFastQ {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        false // since we want to dump all the reads even if later on there's a Head
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
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
        for tag in &self.comment_location_tags {
            if !tags.iter().any(|(name, _)| name == tag) {
                //prevent duplicates
                tags.push((tag.clone(), &[TagValueType::Location]));
            }
        }
        Some(tags)
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
        self.output_streams = Some(Arc::new(Mutex::new(demultiplex_info.open_output_streams(
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
        )?)));
        Ok(None)
    }

    #[allow(clippy::too_many_lines)]
    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let mut error_encountered = None;

        'outer: for (ii, tag) in &mut block
            .tags
            .get(&self.in_label)
            .expect("in_label tag must exist in block")
            .iter()
            .enumerate()
        {
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
                    let segment_block = &block.segments[tag.0[0]
                        .location
                        .as_ref()
                        .expect("location must be set for tag")
                        .segment_index
                        .0];
                    let wrapped = segment_block.get(ii);

                    // Determine which output stream to use based on demultiplexing
                    let output_idx = block.output_tags.as_ref().map_or(0, |x| x[ii]);

                    if let Some(writer) = self
                        .output_streams
                        .as_ref()
                        .expect("Should have been set in init")
                        .lock()
                        .expect("lock poisoned")
                        .0
                        .get_mut(&output_idx)
                        .expect("output stream must exist for index")
                    {
                        //if we have demultiplex & no-unmatched-output, this happens
                        let mut name = wrapped.name().to_vec();
                        for tag in &self.comment_tags {
                            if let Some(tag_value) = block
                                .tags
                                .get(tag)
                                .expect("tag must exist in block")
                                .get(ii)
                            {
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
                        for location_tag in &self.comment_location_tags {
                            if let Some(tag_value) = block
                                .tags
                                .get(location_tag)
                                .expect("location tag must exist in block. uses_tag mistake?")
                                .get(ii)
                                && let Some(hits) = tag_value.as_sequence()
                            {
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
        &self,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<crate::transformations::FinalizeReportResult>> {
        // Flush all output streams
        let output_streams = self
            .output_streams
            .as_ref()
            .expect("output streams should have been set in init")
            .lock()
            .expect("lock poisoned")
            .take();
        for (_tag, writer) in output_streams {
            if let Some(writer) = writer {
                let (_, _) = writer.finish();
            }
            // Finalize the writer to ensure all data is flushed and hashes are computed
        }

        Ok(None)
    }
}
