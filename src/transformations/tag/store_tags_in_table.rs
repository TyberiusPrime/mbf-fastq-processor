#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::{
    Demultiplex, config::CompressionFormat, config::deser::bstring_from_string, dna::TagValue,
};
use anyhow::{Result, bail};

use super::super::{FinalizeReportResult, Step, Transformation, tag::default_region_separator};

#[derive(eserde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    #[serde(default)]
    infix: String,
    #[serde(default)]
    compression: CompressionFormat,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    full_output_paths: HashMap<u16, PathBuf>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    output_handles: Vec<Option<csv::Writer<Box<HashedAndCompressedWriter<'static, ex::fs::File>>>>>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    tags: Option<Vec<String>>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    ix_separator: String,
}

impl std::fmt::Debug for StoreTagsInTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagsInTable")
            .field("infix", &self.infix)
            .field("compression", &self.compression)
            .field("region_separator", &self.region_separator)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
}

impl Clone for StoreTagsInTable {
    fn clone(&self) -> Self {
        Self {
            infix: self.infix.clone(),
            compression: self.compression,
            region_separator: self.region_separator.clone(),
            full_output_paths: self.full_output_paths.clone(),
            output_handles: Vec::new(), // Handles will be created on first apply
            tags: None,                 // Tags will be determined on first apply
            ix_separator: self.ix_separator.clone(),
        }
    }
}

impl Step for StoreTagsInTable {
    fn move_inited(&mut self) -> StoreTagsInTable {
        let mut res = self.clone();
        res.output_handles = self.output_handles.drain(..).collect();
        res
    }

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

    fn configure_output_separator(&mut self, ix_separator: &str) {
        self.ix_separator = ix_separator.to_string();
    }

    fn init(
        &mut self,
        _input_info: &super::super::InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        demultiplex_info: &Demultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        // Determine file extension based on compression
        let buffered_writers = demultiplex_info.demultiplexed.open_output_streams(
            output_directory,
            output_prefix,
            self.infix.as_str(),
            "tsv",
            &self.ix_separator, //todo: ix-sepearator from config...
            self.compression,
            None,
            false,
            false,
            allow_overwrite,
        )?;

        self.output_handles = buffered_writers
            .into_iter()
            .map(|opt_buffered_writer| match opt_buffered_writer {
                Some(buffered_writer) => Some(
                    csv::WriterBuilder::new()
                        .delimiter(b'\t')
                        .from_writer(buffered_writer),
                ),
                None => None,
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
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if let Some(tags) = block.tags.as_mut() {
            // Initialize output handles and tag list on first call
            if self.tags.is_none() {
                // Sort tags for consistent column order
                if self.tags.is_none() {
                    let mut tag_list = tags.keys().cloned().collect::<Vec<String>>();
                    tag_list.sort();
                    self.tags = Some(tag_list);
                    // Write header
                    let mut header = vec!["ReadName"];
                    for tag in self.tags.as_ref().unwrap() {
                        header.push(tag);
                    }
                    for writer in self.output_handles.iter_mut() {
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
                let output_tag = output_tags.map(|x| x[ii] as usize).unwrap_or(0);
                if let Some(writer) = self.output_handles[output_tag].as_mut() {
                    let mut record = vec![read.name_without_comment().to_vec()];
                    for tag in self.tags.as_ref().unwrap() {
                        record.push(match &(tags.get(tag).unwrap()[ii]) {
                            TagValue::Sequence(v) => {
                                v.joined_sequence(Some(&self.region_separator))
                            }
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
        }

        Ok((block, true))
    }
    fn finalize(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        // Flush all output handles
        for handle in &mut self.output_handles {
            if let Some(mut writer) = handle.take() {
                writer.flush().expect("Failed final csv flush");
            }
        }
        Ok(None)
    }
}
