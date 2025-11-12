#![allow(clippy::unnecessary_wraps)]

use crate::config::{SegmentIndexOrAll, SegmentOrAll};
use crate::dna::TagValue;
use crate::transformations::prelude::*;
use anyhow::{Result, anyhow};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateAllReadsSameLength {
    /// Segment to validate (default: read1)
    #[serde(default)]
    segment: SegmentOrAll,

    /// Optional tag name to validate - all reads must have the same tag value
    #[serde(default)]
    pub tag: Option<String>,

    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,

    #[serde(default)]
    #[serde(skip)]
    expected_length: Option<usize>,

    #[serde(default)]
    #[serde(skip)]
    expected_tag_value: Option<TagValue>,
}

impl Step for ValidateAllReadsSameLength {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        self.tag.as_ref().map(|tag| {
            vec![(
                tag.clone(),
                &[
                    TagValueType::String,
                    TagValueType::Numeric,
                    TagValueType::Bool,
                    TagValueType::Location,
                ][..],
            )]
        })
    }

    fn needs_serial(&self) -> bool {
        // We need to track state across blocks to remember the first read's length/tag value
        true
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self.segment_index.unwrap();

        match segment_index {
            SegmentIndexOrAll::All => {
                // Check all segments
                for segment_idx in 0..block.segments.len() {
                    self.validate_segment(&block, segment_idx)?;
                }
            }
            SegmentIndexOrAll::Indexed(idx) => {
                // Check single segment
                self.validate_segment(&block, idx)?;
            }
        }

        // Also validate tag if specified
        if self.tag.is_some() {
            self.validate_tag(&block)?;
        }

        Ok((block, true))
    }
}

impl ValidateAllReadsSameLength {
    fn validate_segment(&mut self, block: &FastQBlocksCombined, segment_idx: usize) -> Result<()> {
        if segment_idx >= block.segments.len() {
            return Ok(());
        }

        let segment = &block.segments[segment_idx];

        for read_idx in 0..segment.entries.len() {
            let read = segment.get(read_idx);
            let current_length = read.seq().len();

            if let Some(expected) = self.expected_length {
                if current_length != expected {
                    return Err(anyhow!(
                        "Read length mismatch in segment {}: read '{}' has length {}, but expected length {} (from first read)",
                        segment_idx,
                        bstr::BString::from(read.name()),
                        current_length,
                        expected
                    ));
                }
            } else {
                // First read in first block - remember its length
                self.expected_length = Some(current_length);
            }
        }

        Ok(())
    }

    fn validate_tag(&mut self, block: &FastQBlocksCombined) -> Result<()> {
        let tag_name = self.tag.as_ref().unwrap();

        if let Some(tags) = &block.tags {
            if let Some(tag_values) = tags.get(tag_name) {
                for (idx, tag_value) in tag_values.iter().enumerate() {
                    // Skip Missing values
                    if matches!(tag_value, TagValue::Missing) {
                        continue;
                    }

                    if let Some(expected) = &self.expected_tag_value {
                        // Compare tag values
                        if !self.tag_values_equal(tag_value, expected) {
                            return Err(anyhow!(
                                "Tag '{}' value mismatch at read index {}: expected {:?}, but got {:?}",
                                tag_name,
                                idx,
                                expected,
                                tag_value
                            ));
                        }
                    } else {
                        // First non-missing tag value - remember it
                        self.expected_tag_value = Some(tag_value.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn tag_values_equal(&self, a: &TagValue, b: &TagValue) -> bool {
        match (a, b) {
            (TagValue::Missing, TagValue::Missing) => true,
            (TagValue::String(s1), TagValue::String(s2)) => s1 == s2,
            (TagValue::Numeric(n1), TagValue::Numeric(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (TagValue::Bool(b1), TagValue::Bool(b2)) => b1 == b2,
            (TagValue::Location(l1), TagValue::Location(l2)) => {
                // Compare locations - Hits is a newtype around Vec<Hit>
                // Hit has location: Option<HitRegion> and sequence: BString
                l1.0.len() == l2.0.len()
                    && l1
                        .0
                        .iter()
                        .zip(l2.0.iter())
                        .all(|(h1, h2)| h1.location == h2.location && h1.sequence == h2.sequence)
            }
            _ => false,
        }
    }
}
