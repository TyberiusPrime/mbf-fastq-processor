//eserde false positives
#![allow(clippy::unnecessary_wraps)]
use super::extract_numeric_tags_plus_all;
use crate::transformations::prelude::*;

/// Calculate complexity score (# transitions / (len -1))
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Complexity {
    pub out_label: TagLabel,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialComplexity {
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

impl TagUser for PartialTaggedVariant<PartialComplexity> {
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
            declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric),
            ..Default::default()
        }
    }
}

impl Step for Complexity {

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment,
            &self.out_label,
            |read| {
                // Calculate the number of transitions
                let mut transitions = 0;
                let seq = read.seq();
                if seq.len() <= 1 {
                    return 0.0;
                }
                for ii in 0..seq.len() - 1 {
                    if seq[ii] != seq[ii + 1] {
                        transitions += 1;
                    }
                }
                f64::from(transitions) / (seq.len() - 1) as f64
            },
            |reads| {
                let mut total_transitions = 0usize;
                let mut total_positions = 0usize;

                // Process all reads
                for read in reads {
                    let seq = read.seq();
                    for ii in 0..seq.len() - 1 {
                        if seq[ii] != seq[ii + 1] {
                            total_transitions += 1;
                        }
                    }
                    total_positions += seq.len() - 1;
                }
                if total_positions == 0 {
                    0.0
                } else {
                    total_transitions as f64 / total_positions as f64
                }
            },
            &mut block,
        );

        Ok((block, true))
    }
}