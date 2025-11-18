#![allow(clippy::unnecessary_wraps)] // eserde false positive

use crate::transformations::prelude::*;
use anyhow::{Result, bail};
use bstr::BString;

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll, deser::bstring_from_string},
    transformations::TagValueType,
};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

const fn default_relative() -> bool {
    true
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BaseContent {
    pub out_label: String,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default = "default_relative")]
    pub relative: bool,
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    pub bases_to_count: BString,
    #[serde(default)]
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    pub bases_to_ignore: BString,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
    #[serde(default)]
    #[serde(skip)]
    bases_to_count_lookup: Vec<bool>,
    #[serde(default)]
    #[serde(skip)]
    bases_to_ignore_lookup: Vec<bool>,
}

impl BaseContent {
    fn ensure_lookups(&mut self) -> Result<()> {
        if self.bases_to_count_lookup.is_empty() {
            self.bases_to_count_lookup =
                Self::build_lookup(&self.bases_to_count, "bases_to_count", false)?;
        }
        if self.bases_to_ignore_lookup.is_empty() {
            self.bases_to_ignore_lookup =
                Self::build_lookup(&self.bases_to_ignore, "bases_to_ignore", true)?;
        }
        Ok(())
    }

    fn build_lookup(bases: &BString, field_name: &str, allow_empty: bool) -> Result<Vec<bool>> {
        let mut lookup = vec![false; 256];

        if bases.is_empty() {
            if allow_empty {
                return Ok(lookup);
            }
            bail!("{field_name} must contain at least one letter");
        }

        for ch in bases.as_slice() {
            if !ch.is_ascii_alphabetic() {
                bail!("{field_name} must only contain ASCII letters");
            }
            let idx = ch.to_ascii_uppercase() as usize;
            lookup[idx] = true;
        }

        Ok(lookup)
    }

    fn sequence_totals(
        sequence: &[u8],
        bases_to_count: &[bool],
        bases_to_ignore: &[bool],
    ) -> (usize, usize) {
        let mut considered = 0usize;
        let mut counted = 0usize;

        for &base in sequence {
            let idx = base.to_ascii_uppercase() as usize;
            if bases_to_ignore[idx] {
                continue;
            }
            considered += 1;
            if bases_to_count[idx] {
                counted += 1;
            }
        }

        (considered, counted)
    }

    fn percentage(counted: usize, considered: usize) -> f64 {
        if considered == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (counted as f64 / considered as f64) * 100.0
            }
        }
    }

    pub(crate) fn for_gc_replacement(
        out_label: String,
        segment: SegmentOrAll,
        segment_index: Option<SegmentIndexOrAll>,
    ) -> Self {
        Self {
            out_label,
            segment,
            relative: true,
            bases_to_count: BString::from("GC"),
            bases_to_ignore: BString::from("N"),
            segment_index,
            bases_to_count_lookup: Vec::new(),
            bases_to_ignore_lookup: Vec::new(),
        }
    }

    pub(crate) fn for_n_count(
        out_label: String,
        segment: SegmentOrAll,
        segment_index: Option<SegmentIndexOrAll>,
    ) -> Self {
        Self {
            out_label,
            segment,
            relative: false,
            bases_to_count: BString::from("N"),
            bases_to_ignore: BString::default(),
            segment_index,
            bases_to_count_lookup: Vec::new(),
            bases_to_ignore_lookup: Vec::new(),
        }
    }
}

impl Step for BaseContent {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        if !self.relative && !self.bases_to_ignore.is_empty() {
            bail!("ExtractBaseContent: bases_to_ignore cannot be used when relative = false");
        }
        if self.bases_to_count.is_empty() {
            bail!("bases_to_count must contain at least one letter");
        }
        if self.bases_to_count.iter().any(|x| !x.is_ascii_alphabetic()) {
            bail!("bases_to_count must only contain ASCII letters");
        }
        if self
            .bases_to_ignore
            .iter()
            .any(|x| !x.is_ascii_alphabetic())
        {
            bail!("bases_to_ignore must only contain ASCII letters");
        }
        //self.ensure_lookups()?;
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.out_label.clone(), TagValueType::Numeric))
    }

    #[allow(clippy::cast_precision_loss)]
    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        self.ensure_lookups()?;

        let segment = self
            .segment_index
            .expect("segment_index set during validation");
        let bases_to_count_single = self.bases_to_count_lookup.clone();
        let bases_to_ignore_single = self.bases_to_ignore_lookup.clone();
        let bases_to_count_all = self.bases_to_count_lookup.clone();
        let bases_to_ignore_all = self.bases_to_ignore_lookup.clone();
        let relative = self.relative;

        extract_numeric_tags_plus_all(
            segment,
            &self.out_label,
            move |read| {
                let sequence = read.seq();
                let (considered, counted) = Self::sequence_totals(
                    sequence,
                    &bases_to_count_single,
                    &bases_to_ignore_single,
                );
                if relative {
                    Self::percentage(counted, considered)
                } else {
                    counted as f64
                }
            },
            move |reads| {
                let mut total_considered = 0usize;
                let mut total_counted = 0usize;

                for read in reads {
                    let (considered, counted) = Self::sequence_totals(
                        read.seq(),
                        &bases_to_count_all,
                        &bases_to_ignore_all,
                    );
                    total_considered += considered;
                    total_counted += counted;
                }

                if relative {
                    Self::percentage(total_counted, total_considered)
                } else {
                    total_counted as f64
                }
            },
            &mut block,
        );

        Ok((block, true))
    }
}
