#![allow(clippy::unnecessary_wraps)]

use crate::dna::TagValue;
use crate::transformations::prelude::*;

/// Verify that all reads have the same length
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateAllReadsSameLength {
    source: String,

    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSourceAll>,

    #[serde(default)]
    #[serde(skip)]
    expected_length: std::sync::OnceLock<usize>,
}

impl FromTomlTableNested for ValidateAllReadsSameLength {
    fn from_toml_table(_table: &toml_edit::Table, mut helper: TableErrorHelper) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let resolved_source = helper.get_source_all(&["source", "segment"][..], true);
        helper.deny_unknown()?;

        let (source, resolved_source) = resolved_source?;

        Ok(ValidateAllReadsSameLength {
            source,
            resolved_source: Some(resolved_source), //todo: remove Option
            expected_length: std::sync::OnceLock::new(),
        })
    }
}

impl Step for ValidateAllReadsSameLength {
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
            ResolvedSourceAll::Segment(segment_index_or_all) => {
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
            ResolvedSourceAll::Tag(name) => {
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
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                let mut pseudo_iter = block.get_pseudo_iter();

                match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let mut length_here = 0;
                            for segment in &read.segments {
                                length_here += segment.name_without_comment(*split_character).len();
                            }
                            self.check(length_here)?;
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment_index) => {
                        while let Some(read) = pseudo_iter.pseudo_next() {
                            let length_here = read.segments[*segment_index]
                                .name_without_comment(*split_character)
                                .len();
                            self.check(length_here)?;
                        }
                    }
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
