use anyhow::{bail, Context, Result};

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Segment(pub String);

impl Default for Segment {
    fn default() -> Self {
        Segment(":::first_and_only_segment".to_string())
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SegmentOrAll(pub String);

impl Default for SegmentOrAll {
    fn default() -> Self {
        SegmentOrAll(":::first_and_only_segment".to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize),
}

impl Segment {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(&mut self, input_def: &crate::config::Input) -> Result<SegmentIndex> {
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
            bail!("'all' (or 'All') is not a valid segment in this position.");
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
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
                    "Segment not specified but multiple segments available: [{segment_names}]. \
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
