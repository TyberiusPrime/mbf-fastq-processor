use crate::transformations::prelude::*;

/// Verify that all reads have the same length
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateAllReadsSameLength {
    #[schemars(with = "String")]
    #[tpd(alias = "segment", adapt_in_verify(String))]
    source: ResolvedSourceAll,

    #[schemars(skip)]
    #[tpd(skip)]
    source_name: String,

    #[tpd(skip, default)]
    #[schemars(skip)]
    expected_length: std::sync::OnceLock<usize>,
}

impl VerifyIn<PartialConfig> for PartialValidateAllReadsSameLength {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);

        if let Some(MustAdapt::PostVerify(source)) = self.source.as_ref()
            && let Some(input_def) = parent.input.as_ref()
        {
            self.source_name = Some(source.get_name(input_def.get_segment_order()));
        } else {
            self.source_name = Some(String::new()); // $'well get an error anyway, just not
            // another one about source_name being unset
        }

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialValidateAllReadsSameLength> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = vec![];
            used_tags.extend(inner.source.to_used_tags());
            Some(TagUsageInfo {
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ValidateAllReadsSameLength {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        if !block.is_empty() {
            match &self.source {
                ResolvedSourceAll::Segment(segment_index_or_all) => match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        for molecule in block.molecules() {
                            let mut length_here = 0;
                            for read in &molecule {
                                length_here += read.seq.len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        for read in block.segments[segment_index.as_index()].iter() {
                            let length_here = read.seq.len();
                            self.check(length_here)?;
                        }
                    }
                },
                ResolvedSourceAll::Tag(name) => {
                    let col = block
                        .tags
                        .get(name)
                        .expect("Tag not set?! should have been caught earlier. bug");
                    match col {
                        TagColumn::Location(col) => {
                            for len in col.iter_row_lengths(None) {
                                if len > 0 {
                                    self.check(len)?;
                                }
                            }
                        }
                        TagColumn::String(items) => {
                            for opt_str in items.iter() {
                                if let Some(bstring) = opt_str {
                                    self.check(bstring.len())?;
                                }
                            }
                        }
                        // cov:excl-start
                        _ => unreachable!(),
                        // cov:excl-stop
                    }
                }
                ResolvedSourceAll::Name {
                    segment_index_or_all,
                    split_character,
                } => match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        for molecule in block.molecules() {
                            //todo: We want an iter_names?
                            let mut length_here = 0;
                            for read in &molecule {
                                let nn = split_name_and_comment(read.name, *split_character).0;
                                length_here += nn.len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        for name in &block.segments[segment_index.as_index()].names {
                            let nn = split_name_and_comment(name, *split_character).0;
                            self.check(nn.len())?;
                        }
                    }
                },
            }
        }

        Ok((block, true))
    }
}

impl ValidateAllReadsSameLength {
    fn check(&self, length_here: usize) -> Result<()> {
        self.expected_length.get_or_init(|| length_here);
        if *self
            .expected_length
            .get()
            .expect("Expected length just set")
            != length_here
        {
            bail!(
                "ValidateAllReadsSameLength: Observed differing read lengths for source '{}' ({}, {length_here}). Check your input FASTQ or remove the step if this is expected.",
                &self.source_name,
                self.expected_length.get().expect("just set above"),
            );
        }
        Ok(())
    }
}
