#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::path::{Path, PathBuf};

use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::{
    config::deser::bstring_from_string, config::CompressionFormat, dna::TagValue, Demultiplexed,
};
use anyhow::{bail, Result};

use super::super::{tag::default_region_separator, FinalizeReportResult, Step, Transformation};

#[derive(eserde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    infix: String,
    #[serde(default)]
    compression: CompressionFormat,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    full_output_paths: Vec<PathBuf>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    output_handles: Vec<Option<Box<csv::Writer<HashedAndCompressedWriter<'static, ex::fs::File>>>>>,
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
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        // Determine file extension based on compression
        let extension = match self.compression {
            CompressionFormat::Uncompressed => "tsv",
            CompressionFormat::Gzip => "tsv.gz",
            CompressionFormat::Zstd => "tsv.zst",
        };

        // Create output paths based on demultiplexing
        match demultiplex_info {
            Demultiplexed::No => {
                let base =
                    crate::join_nonempty([output_prefix, self.infix.as_str()], &self.ix_separator);
                self.full_output_paths = vec![output_directory.join(format!("{base}.{extension}"))];
                self.output_handles = vec![None];
            }
            Demultiplexed::Yes(info) => {
                for (_tag, barcode_name) in info.iter_outputs() {
                    let base = crate::join_nonempty(
                        [output_prefix, self.infix.as_str(), barcode_name],
                        &self.ix_separator,
                    );
                    self.full_output_paths
                        .push(output_directory.join(format!("{base}.{extension}")));
                }
                self.output_handles = (0..self.full_output_paths.len()).map(|_| None).collect();
            }
        }

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
        demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if let Some(tags) = block.tags.as_mut() {
            // Initialize output handles and tag list on first call
            if self.tags.is_none() || self.output_handles.is_empty() {
                // Sort tags for consistent column order
                if self.tags.is_none() {
                    let mut tag_list = tags.keys().cloned().collect::<Vec<String>>();
                    tag_list.sort();
                    self.tags = Some(tag_list);
                }

                // Ensure output_handles is the right size
                if self.output_handles.len() != self.full_output_paths.len() {
                    self.output_handles = (0..self.full_output_paths.len()).map(|_| None).collect();
                }

                // Create all output handles and write headers
                for (idx, path) in self.full_output_paths.iter().enumerate() {
                    let file_handle = ex::fs::File::create(path).unwrap_or_else(|err| {
                        panic!("Failed to open table output file: {path:?}: {err:?}")
                    });
                    let buffered_writer = HashedAndCompressedWriter::new(
                        file_handle,
                        self.compression,
                        false,
                        false,
                        None, // compression_level not exposed for StoreTagsInTable yet
                        None,
                    )
                    .expect("Failed to open table output file");
                    let mut writer = csv::WriterBuilder::new()
                        .delimiter(b'\t')
                        .from_writer(buffered_writer);

                    // Write header
                    let mut header = vec!["ReadName"];
                    for tag in self.tags.as_ref().unwrap() {
                        header.push(tag);
                    }
                    writer
                        .write_record(&header)
                        .expect("Failed to write header to table");

                    self.output_handles[idx] = Some(Box::new(writer));
                }
            }

            // Write records to appropriate output files
            match demultiplex_info {
                Demultiplexed::No => {
                    // No demultiplexing: write all records to single file
                    let mut ii = 0;
                    let mut iter = block.segments[0].get_pseudo_iter();
                    while let Some(read) = iter.pseudo_next() {
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
                        self.output_handles[0]
                            .as_mut()
                            .unwrap()
                            .write_record(record)
                            .expect("Failed to write record to table");
                    }
                }
                Demultiplexed::Yes(_) => {
                    // With demultiplexing: write each record to its barcode's file
                    let output_tags = block.output_tags.as_ref().unwrap();
                    let mut ii = 0;
                    let mut iter = block.segments[0].get_pseudo_iter();
                    while let Some(read) = iter.pseudo_next() {
                        let output_tag = output_tags[ii] as usize;
                        let mut record = vec![read.name_without_comment().to_vec()];
                        for tag in self.tags.as_ref().unwrap() {
                            record.push(match &(tags.get(tag).unwrap()[ii]) {
                                TagValue::Sequence(v) => {
                                    v.joined_sequence(Some(&self.region_separator))
                                }
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
                        self.output_handles[output_tag]
                            .as_mut()
                            .unwrap()
                            .write_record(record)
                            .expect("Failed to write record to table");
                    }
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
        _demultiplex_info: &Demultiplexed,
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
