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

    if_tag: Option<String>,
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

impl Step for Postfix {
    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            vec![(
                cond_tag.tag.clone(),
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )]
        })
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
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
