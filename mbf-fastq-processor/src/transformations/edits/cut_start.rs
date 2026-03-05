#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::HitRegion;

/// Cut a fixed number of bases from the start of reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct CutStart {
    n: usize,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,
    #[tpd(default)]
    if_tag: Option<TagLabel>,
}

impl VerifyIn<PartialConfig> for PartialCutStart {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.n.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "n must be > 0",
                    Some("Set to a positive integer."),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialCutStart> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");

        TagUsageInfo {
            used_tags: vec![inner.if_tag.to_used_tag(
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )],
            must_see_all_tags: true,
            ..Default::default()
        }
    }
}

impl Step for CutStart {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self.if_tag.as_ref().map(|tag| {
            let cond_tag = ConditionalTag::from_tag_label(tag);
            get_bool_vec_from_tag(&block, &cond_tag)
        });

        block.apply_in_place(
            self.segment,
            |read| read.cut_start(self.n),
            condition.as_deref(),
        );

        block.filter_tag_locations(
            self.segment,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment_index: location.segment_index,
                    })
                }
            },
            condition.as_deref(),
        );

        Ok((block, true))
    }
}
