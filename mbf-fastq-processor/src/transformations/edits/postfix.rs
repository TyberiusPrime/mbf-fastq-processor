#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::config::deser::{tpd_adapt_bstring, tpd_adapt_dna_bstring_plus_n};

/// Add a fixed sequence to the end of reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Postfix {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndex,

    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_dna_bstring_plus_n")]
    pub seq: BString,
    //we don't check the quality. It's on you if you
    //write non phred values in there
    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring")] //TODO: actually verify quality range
    pub qual: BString,

    if_tag: Option<TagLabel>,
}

impl VerifyIn<PartialConfig> for PartialPostfix {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.segment.validate_segment(parent);
        if let Some(seq) = self.seq.as_ref()
            && let Some(qual) = self.qual.as_ref()
            && seq.len() != qual.len()
        {
            let spans = vec![
                (self.seq.span(), format!("{} characters", seq.len())),
                (self.qual.span(), format!("{} characters", qual.len())),
            ];
            self.seq.state = TomlValueState::Custom { spans };
            self.seq.help = Some("'seq' and 'qual' must be the same length".to_string());
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialPostfix> {
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

impl Step for Postfix {

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

        block.apply_in_place_wrapped(
            self.segment,
            |read| read.postfix(&self.seq, &self.qual),
            condition.as_deref(),
        );
        // postfix doesn't change tags.
        Ok((block, true))
    }
}
