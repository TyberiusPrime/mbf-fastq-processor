#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{config::SegmentIndex, dna::HitRegion};

/// Reverse complement a read
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReverseComplement {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    if_tag: Option<ConditionalTagLabel>,
}

impl VerifyIn<PartialConfig> for PartialReverseComplement {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialReverseComplement> {
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
            used_tags: vec![inner.if_tag.to_used_tag(&[][..])],
            must_see_all_tags: true,
            ..Default::default()
        }
    }
}

impl Step for ReverseComplement {
    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self
            .if_tag
            .as_ref()
            .map(|tag| get_bool_vec_from_tag(&block, tag));

        block.apply_in_place_wrapped(
            self.segment,
            |read| read.reverse_complement(),
            condition.as_deref(),
        );

        block.filter_tag_locations(
            self.segment,
            |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            segment_index: location.segment_index,
                        },
                        new_seq.into(),
                    )
                }
            },
            condition.as_deref(),
        );

        Ok((block, true))
    }
}
