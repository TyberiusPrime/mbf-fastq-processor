#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::HitRegion;

/// Cut a fixed number of bases from the start of reads
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
    #[serde(default)]
    if_tag: Option<String>,
}

impl FromTomlTableNested for CutStart {
    fn from_toml_table(table: &toml_edit::Table, collector: &TableErrorHelper) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut helper = collector.local(table);
        let n = helper.get_clamped("n", Some(1), None);
        let segment: TomlResult<Segment> = helper.get::<String>("Segment").map(Into::into); //todo
        let if_tag = helper.get_opt("if_tag");
        helper.deny_unknown()?;

        Ok(CutStart {
            n: n?,
            segment: segment?,
            segment_index: None,
            if_tag: if_tag?
        })
    }
}

impl Step for CutStart {
    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
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

        block.apply_in_place(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.cut_start(self.n),
            condition.as_deref(),
        );

        block.filter_tag_locations(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
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
