#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::path::Path;

use crate::transformations::prelude::*;

use crate::{config::deser::bstring_from_string, config::CompressionFormat, dna::TagValue};

use super::super::{tag::default_region_separator, FinalizeReportResult};

#[derive(eserde::Deserialize, JsonSchema, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    #[serde(default)]
    infix: String,
    #[serde(default)]
    compression: CompressionFormat,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    region_separator: BString,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    output_handles: DemultiplexedData<Option<csv::Writer<Box<OutputWriter>>>>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    tags: Option<Vec<String>>,
}

/* impl std::fmt::Debug for StoreTagsInTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagsInTable")
            .field("infix", &self.infix)
            .field("compression", &self.compression)
            .field("region_separator", &self.region_separator)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
} */

impl Step for StoreTagsInTable {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transform_index: usize,
    ) -> Result<()> {
        let any_before = all_transforms[..this_transform_index]
            .iter()
            .any(|trafo| trafo.declares_tag_type().is_some());
        if !any_before {
            bail!(
                "StoreTagsInTable needs at least one tag to be set before it in the transformation chain."
            );
        }
        Ok(())
    }

    fn uses_all_tags(&self) -> bool {
        true
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
        // Determine file extension based on compression
        let buffered_writers = demultiplex_info.open_output_streams(
            output_directory,
            output_prefix,
            self.infix.as_str(),
            "tsv",
            output_ix_separator,
            self.compression,
            None,
            false,
            false,
            allow_overwrite,
        )?;

        self.output_handles = buffered_writers
            .0
            .into_iter()
            .map(|(tag, opt_buffered_writer)| {
                (
                    tag,
                    match opt_buffered_writer {
                        Some(buffered_writer) => Some(
                            csv::WriterBuilder::new()
                                .delimiter(b'\t')
                                .from_writer(buffered_writer),
                        ),
                        None => None,
                    },
                )
            })
            .collect();

        Ok(None)
    }

    fn needs_serial(&self) -> bool {
        true
    }

    fn transmits_premature_termination(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize output handles and tag list on first call
        if self.tags.is_none() {
            // Sort tags for consistent column order
            if self.tags.is_none() {
                let mut tag_list = block.tags.keys().cloned().collect::<Vec<String>>();
                tag_list.sort();
                self.tags = Some(tag_list);
                // Write header
                let mut header = vec!["ReadName"];
                for tag in self.tags.as_ref().unwrap() {
                    header.push(tag);
                }
                for (_demultiplex_tag, writer) in self.output_handles.iter_mut() {
                    if let Some(writer) = writer {
                        writer
                            .write_record(&header)
                            .expect("Failed to write header to table");
                    }
                }
            }
        }

        let output_tags = block.output_tags.as_ref();
        let mut ii = 0;
        let mut iter = block.segments[0].get_pseudo_iter();
        while let Some(read) = iter.pseudo_next() {
            let output_tag = output_tags.map(|x| x[ii]).unwrap_or(0);
            if let Some(writer) = self.output_handles.get_mut(&output_tag).unwrap() {
                let mut record = vec![read
                    .name_without_comment(input_info.comment_insert_char)
                    .to_vec()];
                for tag in self.tags.as_ref().unwrap() {
                    record.push(match &(block.tags.get(tag).unwrap()[ii]) {
                        TagValue::Location(v) => v.joined_sequence(Some(&self.region_separator)),
                        TagValue::String(value) => value.to_vec(),
                        TagValue::Numeric(n) => n.to_string().into_bytes(),
                        TagValue::Bool(n) => {
                            if *n {
                                "1".into()
                            } else {
                                "0".into()
                            }
                        }
                        TagValue::Missing => Vec::new(),
                    });
                }
                ii += 1;
                writer
                    .write_record(record)
                    .expect("Failed to write record to table");
            }
        }

        Ok((block, true))
    }
    fn finalize(
        &mut self,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        // Flush all output handles
        for handle in &mut self.output_handles {
            if let Some(mut writer) = handle.1.take() {
                writer.flush().expect("Failed final csv flush");
            }
        }
        Ok(None)
    }
}
