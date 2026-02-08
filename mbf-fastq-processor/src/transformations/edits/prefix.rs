#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{
    config::deser::{tpd_adapt_bstring, tpd_adapt_dna_bstring},
    dna::HitRegion,
};

/// add a fixed prefix to the start of reads
#[derive(Clone, JsonSchema)]
#[tpd(partial=false)]
#[derive(Debug)]
pub struct Prefix {
    #[tpd_default]
    segment: Segment,
    #[schemars(skip)]
    #[tpd_skip]
    segment_index: Option<SegmentIndex>,

    //todo
    //#[serde(deserialize_with = "dna_from_string")]
    #[schemars(with = "String")]
    #[tpd_with(tpd_adapt_bstring)]
    pub seq: BString,
    //#[serde(deserialize_with = "bstring_from_string")]
    //we don't check the quality. It's on you if you
    //write non phred values in there
    #[schemars(with = "String")]
    #[tpd_with(tpd_adapt_bstring)]
    pub qual: BString,

    if_tag: Option<String>,
}

impl VerifyFromToml for PartialPrefix {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.seq = self.seq.verify(helper, |s: &BString| {
            for c in s.iter() {
                if !matches!(c, b'A' | b'C' | b'G' | b'T' | b'N') {
                    return Err((
                        "Invalid DNA base".to_string(),
                        Some("Allowed characters are A, C, G, T, N".to_string()),
                    ));
                }
            }
            Ok(())
        });

        self
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

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!(
                "Prefix: 'seq' and 'qual' must be the same length. Sequence has {} characters but quality string has {} characters. Please ensure they match.",
                self.seq.len(),
                self.qual.len()
            );
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
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
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.prefix(&self.seq, &self.qual),
            condition.as_deref(),
        );
        let prefix_len = self.seq.len();

        block.filter_tag_locations(
            self.segment_index
                .expect("segment_index must be set during initialization"),
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
