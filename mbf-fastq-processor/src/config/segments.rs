use anyhow::{Context, Result, bail};
use schemars::JsonSchema;

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
pub struct Segment(pub String);

impl Default for Segment {
    fn default() -> Self {
        Segment(":::first_and_only_segment".to_string())
    }
}

impl From<String> for Segment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
pub struct SegmentOrAll(pub String);

impl Default for SegmentOrAll {
    fn default() -> Self {
        SegmentOrAll(":::first_and_only_segment".to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize),
}

impl Segment {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(&self, input_def: &crate::config::Input) -> Result<SegmentIndex> {
        if self.0 == ":::first_and_only_segment" {
            if input_def.segment_count() == 1 {
                return Ok(SegmentIndex(0));
            } else {
                let segment_names = input_def.get_segment_order().join(", ");
                bail!(
                    "Segment not specified but multiple segments available: [{segment_names}]. \
                     Please specify which segment to use with 'segment = \"segment_name\"'",
                );
            }
        }
        if self.0 == "all" || self.0 == "All" {
            bail!(
                "'all' (or 'All') is not a valid segment in this position. Choose one of these: [{}]",
                input_def.get_segment_order().join(", ")
            );
        }
        let name = &self.0;
        let idx = input_def.index(name).with_context(|| {
            let segment_names = input_def.get_segment_order().join(", ");
            format!("Unknown segment: {name}. Available [{segment_names}]")
        })?;
        Ok(SegmentIndex(idx))
    }
}

impl SegmentOrAll {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<SegmentIndexOrAll> {
        if self.0 == ":::first_and_only_segment" {
            if input_def.segment_count() == 1 {
                return Ok(SegmentIndexOrAll::Indexed(0));
            } else {
                let segment_names = input_def.get_segment_order().join(", ");
                bail!(
                    "Segment not specified but multiple segments available: [{segment_names}]. Also 'all' is valid here. \
                     Please specify which segment to use with 'segment = \"segment_name\"'",
                );
            }
        }
        if self.0 == "all" || self.0 == "All" {
            return Ok(SegmentIndexOrAll::All);
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
        Ok(SegmentIndexOrAll::Indexed(idx))
    }
}

impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }
}

impl TryInto<SegmentIndex> for SegmentIndexOrAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<SegmentIndex, Self::Error> {
        match self {
            SegmentIndexOrAll::Indexed(idx) => Ok(SegmentIndex(idx)),
            SegmentIndexOrAll::All => Err(()),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, JsonSchema)]
pub struct SegmentSequenceOrName(pub String);

/* impl Default for SegmentSequenceOrName {
    fn default() -> Self {
        SegmentSequenceOrName(":::first_and_only_segment".to_string())
    }
} */

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentOrNameIndex {
    Sequence(SegmentIndex),
    Name(SegmentIndex),
}

impl SegmentSequenceOrName {
    /// validate and turn into an indexed segment (either sequence or name)
    pub(crate) fn validate(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<SegmentOrNameIndex> {
        /* if self.0 == ":::first_and_only_segment" {
            if input_def.segment_count() == 1 {
                return Ok(SegmentOrNameIndex::Sequence(SegmentIndex(0)));
            } else {
                let segment_names = input_def.get_segment_order().join(", ");
                bail!(
                    "Source (segment/name) not specified but multiple segments available: [{segment_names}]. \
                     Please specify which segment to use with 'source = \"segment_name\"' or 'source = \"name:segment_name\"'",
                );
            }
        } */
        if self.0 == "all" || self.0 == "All" {
            bail!(
                "'all' (or 'All') is not a valid segment in this position. Choose one of these: [{}]",
                input_def.get_segment_order().join(", ")
            );
        }

        // Check for name: prefix
        if let Some(segment_name) = self.0.strip_prefix("name:") {
            let idx = input_def.index(segment_name).with_context(|| {
                let segment_names = input_def.get_segment_order().join(", ");
                format!("Unknown segment in 'name:{segment_name}'. Available [{segment_names}]")
            })?;
            Ok(SegmentOrNameIndex::Name(SegmentIndex(idx)))
        } else {
            // Regular segment reference (sequence)
            let idx = input_def.index(&self.0).with_context(|| {
                let segment_names = input_def.get_segment_order().join(", ");
                format!("Unknown segment: {}. Available [{segment_names}]", self.0)
            })?;
            Ok(SegmentOrNameIndex::Sequence(SegmentIndex(idx)))
        }
    }
}

impl SegmentOrNameIndex {
    #[must_use]
    pub fn get_segment_index(&self) -> SegmentIndex {
        match self {
            SegmentOrNameIndex::Sequence(idx) | SegmentOrNameIndex::Name(idx) => *idx,
        }
    }

    #[must_use]
    pub fn is_name(&self) -> bool {
        matches!(self, SegmentOrNameIndex::Name(_))
    }
}
