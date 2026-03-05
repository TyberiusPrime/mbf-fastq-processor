#![allow(clippy::unnecessary_wraps)]
use std::cell::RefCell;
use std::rc::Rc;

use crate::transformations::prelude::*;

use crate::{config::CompressionFormat, config::deser::tpd_adapt_bstring, dna::TagValue};

use super::super::tag::default_region_separator;

//otherwise clippy won't shut up, because we can't allow it for the derived serde / eserde fields
type OutputHandles = Arc<Mutex<DemultiplexedData<Option<csv::Writer<Box<OutputWriter>>>>>>;
type InLabels = Arc<Mutex<Option<Vec<TagLabel>>>>;

/// Store all currently defined tags in a TSV
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct StoreTagsInTable {
    #[tpd(default)]
    infix: String,
    #[tpd(default)]
    compression: CompressionFormat,

    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring")]
    region_separator: BString,

    #[schemars(skip)]
    #[tpd(skip, default)]
    output_handles: Option<OutputHandles>,

    #[allow(dead_code)] //only used in deser
    in_labels: Option<Vec<TagLabel>>,

    #[tpd(skip)]
    #[schemars(skip)]
    final_in_labels: InLabels,
}

impl VerifyIn<PartialConfig> for PartialStoreTagsInTable {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        // //test case says we accept this
        // self.infix.verify(|infix: &String| {
        //     if infix.is_empty() {
        //         Err(ValidationFailure::new("Infix must not be empty", None))
        //     } else {
        //         Ok(())
        //     }
        // });
        self.region_separator.or_with(default_region_separator);
        if let Some(Some(in_labels)) = self.in_labels.as_ref() {
            self.final_in_labels = Some(Arc::new(Mutex::new(Some(
                in_labels
                    .iter()
                    .map(|tv| {
                        tv.as_ref()
                            .expect("Parent was ok, child should be as well")
                            .clone()
                    })
                    .collect(),
            ))));
        } else {
            self.final_in_labels = Some(Arc::new(Mutex::new(None)));
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialStoreTagsInTable> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        if tags_available.is_empty() {
            self.toml_value.state = TomlValueState::ValidationFailed {
                message: "StoreTagsInTable needs at least one tag to be set before it in the transformation chain.".to_string(),
            };
            return TagUsageInfo {
                ..Default::default()
            };
        }

        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        let toml_source = Rc::new(RefCell::new((
            &mut self.toml_value.state,
            &mut self.toml_value.help,
        )));
        TagUsageInfo {
            must_see_all_tags: true, // while this means the apply() sees them all, it does not
            // register them as 'used tags'
            used_tags: tags_available //we still need to name them, so they don't appear unused
                .iter()
                .map(|(tag, _metadata)| {
                    Some(UsedTag {
                        name: tag.clone(),
                        accepted_tag_types: ANY_TAG_TYPE,
                        toml_source: toml_source.clone(),
                        further_help: None,
                    })
                })
                .collect(),
            ..Default::default()
        }
    }
}

impl Step for StoreTagsInTable {
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

        self.output_handles = Some(Arc::new(Mutex::new(
            buffered_writers
                .0
                .into_iter()
                .map(|(tag, opt_buffered_writer)| {
                    (
                        tag,
                        opt_buffered_writer.map(|buffered_writer| {
                            csv::WriterBuilder::new()
                                .delimiter(b'\t')
                                .from_writer(buffered_writer)
                        }),
                    )
                })
                .collect(),
        )));

        Ok(None)
    }

    // needed to ensure output order
    fn needs_serial(&self) -> bool {
        true
    }

    fn transmits_premature_termination(&self) -> bool {
        false // since we want to dump all the reads even if later on there's a Head
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize output handles and tag list on first call
        //TODO move into verify
        let mut in_label_lock = self.final_in_labels.lock().expect("lock poisoned");
        if in_label_lock.is_none() {
            // Sort tags for consistent column order

            let mut tag_list: Vec<_> = block.tags.keys().cloned().collect();
            tag_list.sort();
            // Write header
            {
                let mut header = vec!["ReadName"];
                for tag in &tag_list {
                    header.push(&tag.0);
                }

                for (_demultiplex_tag, writer) in self
                    .output_handles
                    .as_ref()
                    .expect("was set in init?")
                    .lock()
                    .expect("lock poisoned")
                    .iter_mut()
                {
                    if let Some(writer) = writer {
                        writer
                            .write_record(&header)
                            .expect("Failed to write header to table");
                    }
                }
            }

            in_label_lock.replace(tag_list);
        }

        let output_tags = block.output_tags.as_ref();
        let mut ii = 0;
        let mut iter = block.segments[0].get_pseudo_iter();
        let mut output_handles = self
            .output_handles
            .as_ref()
            .expect("was set in init?")
            .lock()
            .expect("lock poisoned");
        while let Some(read) = iter.pseudo_next() {
            let output_tag = output_tags.map_or(0, |x| x[ii]);
            if let Some(writer) = output_handles
                .get_mut(&output_tag)
                .expect("output_handle must exist for tag")
            {
                let mut record = vec![
                    read.name_without_comment(input_info.comment_insert_char)
                        .to_vec(),
                ];
                for tag in in_label_lock
                    .as_ref()
                    .expect("in_labels must be set during initialization")
                {
                    record.push(
                        match &(block.tags.get(tag).expect("tag must exist in block.tags")[ii]) {
                            TagValue::Location(v) => {
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
                        },
                    );
                }
                ii += 1;
                writer
                    .write_record(record)
                    .expect("Failed to write record to table");
            }
        }

        Ok((block, true))
    }
    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        // Flush all output handles
        for handle in self
            .output_handles
            .as_ref()
            .expect("was set in init")
            .lock()
            .expect("Locks poisened")
            .iter_mut()
        {
            if let Some(mut writer) = handle.1.take() {
                writer.flush().expect("Failed final csv flush");
            }
        }
        Ok(None)
    }
}
