use crate::transformations::prelude::*;
use fastqrab_config::{
    fileformats::PhredEncoding, tpd_adapt_bstring, tpd_adapt_dna_bstring_plus_n,
};

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

    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring")]
    pub qual: BString,
    pub encoding: PhredEncoding,

    #[tpd(alias = "if_label")]
    if_tag: Option<ConditionalTagLabel>,
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
        self.encoding.or(PhredEncoding::Sanger);
        if let Some(encoding) = self.encoding.as_ref() {
            let (lower, upper) = encoding.limits();
            self.qual.verify(|v| {
                if v.iter().all(|&x| x >= lower && x <= upper) {
                    Ok(())
                } else {
                    Err(ValidationFailure::new(
                        format!(
                            "Quality values must be in the range ({lower}..{upper}) ('{encoding}')"
                        ),
                        None,
                    ))
                }
            });
        } // cov:excl-line
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialPostfix> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.if_tag.to_used_tag(&[])],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Postfix {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self
            .if_tag
            .as_ref()
            .map(|tag| get_bool_vec_from_tag(&block, tag));

        // Route through `DualStringPod::postfix` so the column's edit history
        // survives the rebuild. Appending at the end leaves existing tag
        // locations where they are, but a blind manual rebuild would reset the
        // edit log and strand any tag born before this step.
        let seg = &mut block.segments[self.segment.as_index()];
        let pod = std::mem::replace(&mut seg.seq_quals, DualStringPod::empty());
        seg.seq_quals = pod.postfix(&self.seq, &self.qual, condition.as_deref());

        Ok((block, true))
    }
}
