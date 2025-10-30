#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::path::{Path, PathBuf};

use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::{
    Demultiplexed, config::CompressionFormat, config::deser::bstring_from_string, dna::TagValue,
};
use anyhow::{Result, bail};

use super::super::{FinalizeReportResult, Step, Transformation, tag::default_region_separator};

#[derive(eserde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    table_filename: String,
    #[serde(default)]
    compression: CompressionFormat,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    full_output_path: Option<PathBuf>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    output_handle: Option<Box<csv::Writer<HashedAndCompressedWriter<'static, ex::fs::File>>>>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    tags: Option<Vec<String>>,
}

impl std::fmt::Debug for StoreTagsInTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagsInTable")
            .field("table_filename", &self.table_filename)
            .field("compression", &self.compression)
            .field("region_separator", &self.region_separator)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
}

impl Clone for StoreTagsInTable {
    fn clone(&self) -> Self {
        Self {
            table_filename: self.table_filename.clone(),
            compression: self.compression,
            region_separator: self.region_separator.clone(),
            full_output_path: self.full_output_path.clone(),
            output_handle: None,
            tags: None,
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

    fn init(
        &mut self,
        _input_info: &super::super::InputInfo,
        _output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        self.full_output_path = Some(output_directory.join(&self.table_filename));

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
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if let Some(tags) = block.tags.as_mut() {
            if self.tags.is_none() {
                let file_handle = ex::fs::File::create(self.full_output_path.as_ref().unwrap())
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to open table output file: {:?}: {err:?}",
                            self.full_output_path.as_ref().unwrap()
                        )
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
                let writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(buffered_writer);
                self.output_handle = Some(Box::new(writer));

                self.tags = Some(
                    // that's the order we're going to keep
                    {
                        let mut tags = tags.keys().cloned().collect::<Vec<String>>();
                        tags.sort();
                        tags
                    },
                );
                let mut header = vec!["ReadName"];
                for tag in self.tags.as_ref().unwrap() {
                    header.push(tag);
                }
                self.output_handle
                    .as_mut()
                    .unwrap()
                    .write_record(&header)
                    .expect("Failed to write header to table");
            }
            let mut ii = 0;
            let mut iter = block.segments[0].get_pseudo_iter();
            while let Some(read) = iter.pseudo_next() {
                let mut record = vec![read.name_without_comment().to_vec()];
                for tag in self.tags.as_ref().unwrap() {
                    record.push(match &(tags.get(tag).unwrap()[ii]) {
                        TagValue::Sequence(v) => v.joined_sequence(Some(&self.region_separator)),
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
                self.output_handle
                    .as_mut()
                    .unwrap()
                    .write_record(record)
                    .expect("Failed to write record to table");
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
        self.output_handle
            .take()
            .unwrap() //since we fail in validation if there are no tags, we always have an output
            //handle
            .flush()
            .expect("Failed final csv flush");
        Ok(None)
    }
}
