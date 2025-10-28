use anyhow::{Context, Result, bail};
use serde::Deserialize as SerdeDeserialize;
use serde::de::Error as SerdeError;

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SegmentOrName {
    Sequence(SegmentOrAll),
    Name(String),
}

impl Default for SegmentOrName {
    fn default() -> Self {
        SegmentOrName::Sequence(SegmentOrAll::default())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentSequenceOrAllOrName {
    Sequence(SegmentIndex),
    All,
    Name(SegmentIndex),
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

impl SegmentOrName {
    pub(crate) fn from_raw(value: String) -> Result<Self> {
        if let Some(rest) = value.strip_prefix("name:") {
            if rest.is_empty() {
                bail!("Segment name for 'name:' prefix may not be empty");
            }
            Ok(SegmentOrName::Name(rest.to_string()))
        } else {
            Ok(SegmentOrName::Sequence(SegmentOrAll(value)))
        }
    }

    pub(crate) fn validate(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<SegmentSequenceOrAllOrName> {
        match self {
            SegmentOrName::Sequence(segment) => match segment.validate(input_def)? {
                SegmentIndexOrAll::Indexed(idx) => {
                    Ok(SegmentSequenceOrAllOrName::Sequence(SegmentIndex(idx)))
                }
                SegmentIndexOrAll::All => Ok(SegmentSequenceOrAllOrName::All),
            },
            SegmentOrName::Name(segment_name) => {
                if segment_name == "all" || segment_name == "All" {
                    bail!("'name:all' (or 'name:All') is not supported");
                }
                let name = segment_name.as_str();
                let idx = input_def.index(name).with_context(|| {
                    let segment_names = input_def.get_segment_order().join(", ");
                    format!(
                        "Unknown segment for name extraction: {name}. Available [{segment_names}]"
                    )
                })?;
                Ok(SegmentSequenceOrAllOrName::Name(SegmentIndex(idx)))
            }
        }
    }
}

impl<'de> SerdeDeserialize<'de> for SegmentOrName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <String as SerdeDeserialize>::deserialize(deserializer)?;
        SegmentOrName::from_raw(raw).map_err(SerdeError::custom)
    }
}

impl<'de> eserde::EDeserialize<'de> for SegmentOrName {
    fn deserialize_for_errors<D>(_deserializer: D) -> Result<(), ()>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(())
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
