#![allow(clippy::unnecessary_wraps)]

use crate::config::{SegmentIndexOrAll, SegmentOrAll};
use crate::dna::TagValue;
use crate::transformations::ResolvedSource;
use crate::transformations::prelude::*;

fn default_source() -> String {
    //tha's first read segment if only one is set.
    SegmentOrAll::default().0
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateAllReadsSameLength {
    /// Segment to validate (default: read1)

    #[serde(default = "default_source")]
    #[serde(alias = "segment")]
    source: String,

    /// Optional tag name to validate - all reads must have the same tag value
    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSource>,

    #[serde(default)]
    #[serde(skip)]
    expected_length: std::sync::OnceLock<usize>,
}

impl Step for ValidateAllReadsSameLength {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.resolved_source = Some(ResolvedSource::parse(&self.source, input_def)?);
        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        self.resolved_source
            .as_ref()
            .expect("resolved_source must be set during initialization")
            .get_tags()
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        match self
            .resolved_source
            .as_ref()
            .expect("resolved_source must be set during initialization")
        {
            ResolvedSource::Segment(segment_index_or_all) => {
                let mut pseudo_iter = block.get_pseudo_iter();
                match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let mut length_here = 0;
                            for segment in &read.segments {
                                length_here += segment.seq().len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let length_here = read.segments[*segment_index].seq().len();
                            self.check(length_here)?;
                        }
                    }
                }
            }
            ResolvedSource::Tag(name) => {
                for value in block
                    .tags
                    .get(name)
                    .expect("Tag not set?! should have been caught earlier. bug")
                {
                    let length_here = match value {
                        TagValue::Missing => continue,
                        TagValue::Location(hits) => hits.covered_len(),
                        TagValue::String(bstring) => bstring.len(),
                        _ => unreachable!(),
                    };
                    self.check(length_here)?;
                }
            }
            ResolvedSource::Name {
                segment,
                split_character,
            } => {
                let mut pseudo_iter = block.get_pseudo_iter();
                while let Some(read) = pseudo_iter.pseudo_next() {
                    let name = read.segments[segment.0].name_without_comment(*split_character);
                    let length_here = name.len();
                    self.check(length_here)?;
                }
            }
        }

        Ok((block, true))
    }
}

impl ValidateAllReadsSameLength {
    fn check(&self, length_here: usize) -> Result<()> {
        self.expected_length.get_or_init(|| length_here);
        if *self
            .expected_length
            .get()
            .expect("Expected length just set")
            != length_here
        {
            bail!(
                "ValidateAllReadsSameLength: Observed differing read lengths for source '{}' ({}, {}). Check your input FASTQ or remove the step if this is expected.",
                self.source,
                self.expected_length.get().expect("just set above"),
                length_here,
            );
        }
        Ok(())
    }
}
