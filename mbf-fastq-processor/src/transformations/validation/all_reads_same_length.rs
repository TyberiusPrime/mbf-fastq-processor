#![allow(clippy::unnecessary_wraps)]

use crate::dna::TagValue;
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
            self.source_name = Some("".to_string()); // just supress the error message.
        }

        if self.source.is_missing() {
            self.source.help = Some(format!(
                "Please provide a source, that is a <segment name>, a <name:segment_name> or tag name. Available segments: {}",
                toml_pretty_deser::format_quoted_list(
                    &(parent.input.as_ref().map_or_else(
                        || vec![""],
                        |input_def| input_def
                            .get_segment_order()
                            .iter()
                            .map(String::as_str)
                            .collect()
                    ))
                )
            ));
        }

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialValidateAllReadsSameLength> {
    fn get_tag_usage(&mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        let mut used_tags = vec![];
        used_tags.extend(inner.source.to_used_tags());

        TagUsageInfo {
            used_tags,
            ..Default::default()
        }
    }
}

impl Step for ValidateAllReadsSameLength {

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        match &self.source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                let mut pseudo_iter = block.get_pseudo_iter();
                match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let mut length_here = 0;
                            for segment in &read.segments {
                                length_here += segment.seq().len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let length_here = read.segments[*segment_index].seq().len();
                            self.check(length_here)?;
                        }
                    }
                }
            }
            ResolvedSourceAll::Tag(name) => {
                for value in block
                    .tags
                    .get(name)
                    .expect("Tag not set?! should have been caught earlier. bug")
                {
                    let length_here = match value {
                        TagValue::Missing => continue,
                        TagValue::Location(hits) => hits.covered_len(),
                        TagValue::String(bstring) => bstring.len(),
                        _ => unreachable!(),
                    };
                    self.check(length_here)?;
                }
            }
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                let mut pseudo_iter = block.get_pseudo_iter();

                match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let mut length_here = 0;
                            for segment in &read.segments {
                                length_here += segment.name_without_comment(*split_character).len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let length_here = read.segments[*segment_index]
                                .name_without_comment(*split_character)
                                .len();
                            self.check(length_here)?;
                        }
                    }
                }
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