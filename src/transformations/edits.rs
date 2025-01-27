use super::{
    apply_in_place, apply_in_place_wrapped, default_name_separator, extract_regions,
    validate_target, ConfigTransformNAndTarget, ConfigTransformTarget, RegionDefinition, Step,
    Target,
};
use crate::config::deser::{
    base_or_dot, dna_from_string, u8_from_char_or_number, u8_from_string, u8_regex_from_string,
};
use anyhow::Result;
use serde_valid::Validate;

fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CutStart {
    n: usize,
    target: Target,
}

impl Step for CutStart {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.cut_start(self.n), &mut block);
        (block, true)
    }

    fn validate(&self, input_def: &crate::config::Input) -> Result<()> {
        validate_target(self.target, input_def)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CutEnd {
    n: usize,
    target: Target,
}

impl Step for CutEnd {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.cut_end(self.n), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct MaxLen {
    n: usize,
    target: Target,
}

impl Step for MaxLen {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.max_len(self.n), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Prefix {
    pub target: Target,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: Vec<u8>,

    #[serde(deserialize_with = "u8_from_string")] //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: Vec<u8>,
}

impl Step for Prefix {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.prefix(&self.seq, &self.qual),
            &mut block,
        );
        (block, true)
    }
}

/* pub fn transform_postfix(
    config: &mut ConfigTransformText,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| read.postfix(&config.seq, &config.qual),
        &mut block,
    );
    (block, true)
} */

pub fn transform_reverse_complement(
    config: &mut ConfigTransformTarget,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(config.target, |read| read.reverse_complement(), &mut block);
    (block, true)
}
pub fn transform_phred_64_to_33(
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    block.apply_mut(|read1, read2, index1, index2| {
        let qual = read1.qual();
        let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
        assert!(
            !new_qual.iter().any(|x| *x < 33),
            "Phred 64-33 conversion yielded values below 33 -> wasn't Phred 64 to begin with"
        );
        read1.replace_qual(new_qual);
        if let Some(inner_read2) = read2 {
            let qual = inner_read2.qual();
            let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
            inner_read2.replace_qual(new_qual);
        }
        if let Some(index1) = index1 {
            let qual = index1.qual();
            let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
            index1.replace_qual(new_qual);
        }
        if let Some(index2) = index2 {
            let qual = index2.qual();
            let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
            index2.replace_qual(new_qual);
        }
    });
    (block, true)
}

#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformRename {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "u8_from_string")]
    pub replacement: Vec<u8>,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformAdapterMismatchTail {
    pub target: Target,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[serde(deserialize_with = "dna_from_string")]
    pub query: Vec<u8>,
}

pub fn transform_trim_adapter_mismatch_tail(
    config: &mut ConfigTransformAdapterMismatchTail,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| {
            read.trim_adapter_mismatch_tail(
                &config.query,
                config.min_length,
                config.max_mismatches,
            );
        },
        &mut block,
    );
    (block, true)
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformPolyTail {
    pub target: Target,
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 10.)]
    pub max_mismatch_rate: f32,
    pub max_consecutive_mismatches: usize,
}

pub fn transform_trim_poly_tail(
    config: &mut ConfigTransformPolyTail,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| {
            read.trim_poly_base(
                config.min_length,
                config.max_mismatch_rate,
                config.max_consecutive_mismatches,
                config.base,
            );
        },
        &mut block,
    );
    (block, true)
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformQual {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}
pub fn trim_quality_start(
    config: &mut ConfigTransformQual,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| read.trim_quality_start(config.min),
        &mut block,
    );
    (block, true)
}

pub fn trim_quality_end(
    config: &mut ConfigTransformQual,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| read.trim_quality_end(config.min),
        &mut block,
    );
    (block, true)
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformToName {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,

    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_readname_end_chars"
    )]
    pub readname_end_chars: Vec<u8>,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_separator"
    )]
    pub separator: Vec<u8>,

    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_separator"
    )]
    pub region_separator: Vec<u8>,
}

pub fn transform_extract_to_name(
    config: &mut ConfigTransformToName,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    for ii in 0..block.len() {
        let extracted = extract_regions(ii, &block, &config.regions, &config.separator);
        let mut read1 = block.read1.get_mut(ii);

        let name = read1.name();
        let mut split_pos = None;
        for letter in &config.readname_end_chars {
            if let Some(pos) = name.iter().position(|&x| x == *letter) {
                split_pos = Some(pos);
                break;
            }
        }
        let new_name = match split_pos {
            None => {
                let mut new_name: Vec<u8> = name.into();
                new_name.extend(config.separator.iter());
                new_name.extend(extracted.iter());
                new_name
            }
            Some(split_pos) => {
                let mut new_name =
                    Vec::with_capacity(name.len() + config.separator.len() + extracted.len());
                new_name.extend(name.iter().take(split_pos));
                new_name.extend(config.separator.iter());
                new_name.extend(extracted.iter());
                new_name.extend(name.iter().skip(split_pos));
                new_name
            }
        };
        read1.replace_name(new_name);
    }
    (block, true)
}
pub fn transform_rename(
    config: &mut ConfigTransformRename,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let handle_name = |read: &mut crate::io::WrappedFastQReadMut| {
        let name = read.name();
        let new_name = config
            .search
            .replace_all(name, &config.replacement)
            .into_owned();
        read.replace_name(new_name);
    };
    apply_in_place_wrapped(Target::Read1, handle_name, &mut block);
    if block.read2.is_some() {
        apply_in_place_wrapped(Target::Read2, handle_name, &mut block);
    }
    if block.index1.is_some() {
        apply_in_place_wrapped(Target::Index1, handle_name, &mut block);
    }
    if block.index2.is_some() {
        apply_in_place_wrapped(Target::Index2, handle_name, &mut block);
    }

    (block, true)
}

pub fn transform_swap_r1_and_r2(
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let read1 = block.read1;
    let read2 = block.read2.take().unwrap();
    block.read1 = read2;
    block.read2 = Some(read1);
    (block, true)
}
