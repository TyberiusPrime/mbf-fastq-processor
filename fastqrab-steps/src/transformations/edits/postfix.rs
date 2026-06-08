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
                must_see_all_tags: true,
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
        let first_read_len = block.segments[self.segment.as_index()]
            .seq_quals
            .iter_seq_lens()
            .next()
            .unwrap_or(0);
        let mut new =
            DualStringPodBuilder::with_capacity(block.len(), first_read_len + self.seq.len());
        let mut new_seq = BString::new(Vec::new());
        let mut new_qual = BString::new(Vec::new());
        for (idx, read) in block.segments[self.segment.as_index()].iter().enumerate() {
            if condition.as_ref().is_some_and(|cond| !cond[idx]) {
                new.push(&read.seq, &read.qual);
            } else {
                new_seq.extend_from_slice(&read.seq);
                new_seq.extend_from_slice(&self.seq);
                new_qual.extend_from_slice(&read.qual);
                new_qual.extend_from_slice(&self.qual);
                new.push(&new_seq, &new_qual);
                new_seq.clear();
                new_qual.clear();
            }
        }
        block.segments[self.segment.as_index()].seq_quals = new.finish();

        // postfix doesn't change tags.
        Ok((block, true))
    }
}
