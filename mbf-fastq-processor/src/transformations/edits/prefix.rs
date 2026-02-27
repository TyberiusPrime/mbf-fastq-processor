#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{
    config::deser::{tpd_adapt_bstring, tpd_adapt_dna_bstring_plus_n},
    dna::HitRegion,
};

/// add a fixed prefix to the start of reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Prefix {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    //todo
    //#[serde(deserialize_with = "dna_from_string")]
    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_dna_bstring_plus_n")]
    pub seq: BString,
    //#[serde(deserialize_with = "bstring_from_string")]
    //we don't check the quality. It's on you if you
    //write non phred values in there
    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring")] //todo: actually verify range
    pub qual: BString,

    if_tag: Option<String>,
}

impl VerifyIn<PartialConfig> for PartialPrefix {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
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

impl Step for Prefix {
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
    //to modify location tags
    fn must_see_all_tags(&self) -> bool {
        true
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
            |read| read.prefix(&self.seq, &self.qual),
            condition.as_deref(),
        );
        let prefix_len = self.seq.len();

        block.filter_tag_locations(
            self.segment,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                {
                    NewLocation::New(HitRegion {
                        start: location.start + prefix_len,
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
