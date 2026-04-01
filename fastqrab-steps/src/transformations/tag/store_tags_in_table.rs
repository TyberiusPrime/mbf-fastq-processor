use std::cell::RefCell;
use std::rc::Rc;

use crate::transformations::prelude::*;
use fastqrab_config::{default_region_separator, tpd_adapt_bstring};
use fastqrab_io::CompressionFormat;

type OutputHandles = Arc<Mutex<DemultiplexedData<Option<csv::Writer<Box<OutputWriter>>>>>>;
type InLabels = Vec<TagLabel>;

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
    #[tpd(alias = "tags")]
    #[tpd(adapt_in_verify(String))]
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

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialStoreTagsInTable> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            match inner.in_labels.value.as_mut() {
                Some(Some(in_labels)) => {
                    //they're not ok yet...
                    for tag in in_labels.iter_mut() {
                        tag.validate_incoming_tag_label(tags_available, segment_order);
                    }
                    if in_labels.is_empty() {
                        inner.in_labels.state = TomlValueState::ValidationFailed {
                            message: "in_labels may not be an empty list".to_string(),
                        };
                        inner.in_labels.help =
                            Some("Set to a non-empty list of tag labels.".to_string());
                        inner.final_in_labels = Some(Vec::new());
                        return None;
                    }
                    inner.final_in_labels = Some(
                        in_labels
                            .iter()
                            .filter_map(|x| x.as_ref())
                            .filter_map(|x| x.as_ref_post())
                            .map(|tv| tv.clone())
                            .collect(),
                    );
                }
                Some(None) | None => {
                    if tags_available.is_empty() {
                        self.toml_value.state = TomlValueState::ValidationFailed {
                message: "StoreTagsInTable needs at least one tag to be set before it in the transformation chain.".to_string(),
                };
                        inner.final_in_labels = Some(Vec::new());
                        return None;
                    }
                    let mut final_in_labels: Vec<_> = tags_available.keys().cloned().collect();
                    final_in_labels.sort_unstable();
                    inner.final_in_labels = Some(final_in_labels);
                }
            }

            let final_in_labels: Vec<_> = inner
                .final_in_labels
                .as_ref()
                .expect("set just above")
                .iter()
                .map(|tag_label| tag_label.clone())
                .collect();

            let toml_source = Rc::new(RefCell::new((
                &mut self.toml_value.state,
                &mut self.toml_value.help,
            )));
            let used_tags = final_in_labels
                .into_iter()
                .map(|tag| {
                    Some(UsedTag {
                        name: tag,
                        accepted_tag_types: ANY_TAG_TYPE,
                        toml_source: toml_source.clone(),
                        further_help: None,
                    })
                })
                .collect();

            Some(TagUsageInfo {
                must_see_all_tags: true, // while this means the apply() sees them all, it does not
                // register them as 'used tags'
                used_tags,
                ..Default::default()
            })
        } else {
            None
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
        block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        if block_no == 1 {
            // first block, output header

            let tag_list = &self.final_in_labels;
            // Write header
            {
                let mut header = vec!["ReadName"];
                for tag in tag_list {
                    header.push(tag.as_ref());
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
                for tag in &self.final_in_labels {
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
            } // cov:excl-line
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
