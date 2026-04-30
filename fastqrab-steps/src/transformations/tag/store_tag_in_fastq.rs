use std::io::Write;

use super::{format_numeric_for_comment, store_tag_in_comment};
use crate::transformations::prelude::*;
use fastqrab_config::{
    default_comment_insert_char, default_comment_separator, default_region_separator,
    tpd_adapt_bstring, tpd_adapt_u8_from_byte_or_char,
};
use fastqrab_io::{CompressionFormat, FileFormat};

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
    #[tpd(adapt_in_verify(String))]
    in_label: TagLabel,

    // Optional read name comment fields (like StoreTagInComment)
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "comment_labels")]
    comment_tags: Option<Vec<TagLabel>>,
    //
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
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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
            self.format.as_ref().unwrap_or(&FileFormat::Fastq), // Default to Fastq for validation
                                                                // purposes
        );

        if parent.output.is_ok()
            && let Some(None) = parent.output.value.as_ref()
        {
            return Err(ValidationFailure::new(
                "Missing output configuration",
                Some(
                    "StoreTagInFastQ requires output configuration to determine file paths and formats",
                ),
            ));
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialStoreTagInFastQ> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            inner
                .in_label
                .validate_incoming_tag_label(tags_available, segment_order);
            let in_label = inner
                .in_label
                .to_used_tag(&[TagValueType::Location, TagValueType::String]);

            let mut used_tags = vec![in_label];
            if let Some(Some(comment_tags)) = inner.comment_tags.value.as_mut() {
                for tag in comment_tags.iter_mut() {
                    tag.validate_incoming_tag_label(tags_available, segment_order);
                    if let Some(this_tag) = tag.as_ref().and_then(|x| x.as_ref_post())
                        && used_tags[0].as_ref().is_some_and(|x| x.name == *this_tag)
                    {
                        tag.state = TomlValueState::new_validation_failed(
                            "InLabel repeated in comment_tags",
                        );
                        tag.help = Some("The same tag cannot be used as both the in_label and a comment tag. Remove from comment_tags?".to_string());
                    } else {
                        used_tags.push(tag.to_used_tag(&[
                            TagValueType::Bool,
                            TagValueType::String,
                            TagValueType::Location,
                            TagValueType::Numeric((None, None)),
                        ]));
                    }
                }
            }

            Some(TagUsageInfo {
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for StoreTagInFastQ {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        false // since we want to dump all the reads even if later on there's a Head
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
            self.compression_level,
            false,
            false,
            allow_overwrite,
        )?))); // cov:excl-line
        Ok(None)
    }

    #[expect(clippy::too_many_lines, reason="It takes this many")]
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
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
                        if let Some(comment_tags) = self.comment_tags.as_ref() {
                            for tag in comment_tags {
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
                                        tag.as_ref().as_bytes(),
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
                                } // cov:excl-line
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
                            // cov:excl-start
                            FileFormat::Bam | FileFormat::None => {
                                unreachable!("Unsupported format encountered after validation")
                            } // cov:excl-stop
                        }
                    }
                } // cov:excl-line
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
