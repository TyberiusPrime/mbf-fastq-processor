use super::{
    apply_in_place, apply_in_place_wrapped, default_name_separator, extract_regions,
    validate_target, RegionDefinition, Step, Target, Transformation,
};
use crate::{
    config::deser::{
        base_or_dot, dna_from_string, u8_from_char_or_number, u8_from_string, u8_regex_from_string,
    },
    demultiplex::Demultiplexed,
};
use anyhow::{bail, Result};
use serde_valid::Validate;

fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    target: Target,
}

impl Step for CutStart {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.cut_start(self.n), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutEnd {
    n: usize,
    target: Target,
}

impl Step for CutEnd {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.cut_end(self.n), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MaxLen {
    n: usize,
    target: Target,
}

impl Step for MaxLen {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(self.target, |read| read.max_len(self.n), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub target: Target,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: Vec<u8>,
    #[serde(deserialize_with = "u8_from_string")] //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: Vec<u8>,
}

impl Step for Prefix {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!("Seq and qual must be the same length");
        }
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.prefix(&self.seq, &self.qual),
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Postfix {
    pub target: Target,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: Vec<u8>,
    #[serde(deserialize_with = "u8_from_string")] //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: Vec<u8>,
}

impl Step for Postfix {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,

        _all_transforms: &[Transformation],
    ) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!("Seq and qual must be the same length");
        }
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.postfix(&self.seq, &self.qual),
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReverseComplement {
    pub target: Target,
}

impl Step for ReverseComplement {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
                                                         // enough
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(self.target, |read| read.reverse_complement(), &mut block);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Phred64To33 {}

impl Step for Phred64To33 {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
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
}

#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct Rename {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "u8_from_string")]
    pub replacement: Vec<u8>,
}

impl Step for Rename {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let handle_name = |read: &mut crate::io::WrappedFastQReadMut| {
            let name = read.name();
            let new_name = self
                .search
                .replace_all(name, &self.replacement)
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
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct TrimAdapterMismatchTail {
    pub target: Target,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[serde(deserialize_with = "dna_from_string")]
    pub query: Vec<u8>,
}

impl Step for TrimAdapterMismatchTail {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        if self.max_mismatches > self.min_length {
            bail!("Max mismatches must be <= min length");
        }
        if self.min_length > self.query.len() {
            bail!("Min length must be <= query length");
        }
        validate_target(self.target, input_def)
    }
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| {
                read.trim_adapter_mismatch_tail(&self.query, self.min_length, self.max_mismatches);
            },
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct TrimPolyTail {
    pub target: Target,
    #[validate(minimum = 1)]
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 10.)]
    pub max_mismatch_rate: f32,
    pub max_consecutive_mismatches: usize,
}

impl Step for TrimPolyTail {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| {
                read.trim_poly_base(
                    self.min_length,
                    self.max_mismatch_rate,
                    self.max_consecutive_mismatches,
                    self.base,
                );
            },
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimQualityStart {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}

impl Step for TrimQualityStart {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.trim_quality_start(self.min),
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimQualityEnd {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}

impl Step for TrimQualityEnd {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.trim_quality_end(self.min),
            &mut block,
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ExtractToName {
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

impl Step for ExtractToName {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        super::validate_regions(&self.regions, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let rename_read = |read: &mut crate::io::WrappedFastQReadMut, extracted: &Vec<u8>| {
            let name = read.name();
            let mut split_pos = None;
            for letter in &self.readname_end_chars {
                if let Some(pos) = name.iter().position(|&x| x == *letter) {
                    split_pos = Some(pos);
                    break;
                }
            }
            let new_name = match split_pos {
                None => {
                    let mut new_name: Vec<u8> = name.into();
                    new_name.extend(self.separator.iter());
                    new_name.extend(extracted.iter());
                    new_name
                }
                Some(split_pos) => {
                    let mut new_name =
                        Vec::with_capacity(name.len() + self.separator.len() + extracted.len());
                    new_name.extend(name.iter().take(split_pos));
                    new_name.extend(self.separator.iter());
                    new_name.extend(extracted.iter());
                    new_name.extend(name.iter().skip(split_pos));
                    new_name
                }
            };
            read.replace_name(new_name);
        };

        for ii in 0..block.len() {
            let extracted = extract_regions(ii, &block, &self.regions, &self.separator);
            let mut read = block.read1.get_mut(ii);
            rename_read(&mut read, &extracted);
            if let Some(block2) = block.read2.as_mut() {
                let mut read = block2.get_mut(ii);
                rename_read(&mut read, &extracted);
            }
            if let Some(index1) = block.index1.as_mut() {
                let mut read = index1.get_mut(ii);
                rename_read(&mut read, &extracted);
            }

            if let Some(index2) = block.index2.as_mut() {
                let mut read = index2.get_mut(ii);
                rename_read(&mut read, &extracted);
            }
        }

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwapR1AndR2 {}

impl Step for SwapR1AndR2 {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        {
            if input_def.read2.is_none() {
                bail!(
                    "Read2 is not defined in the input section, but used by transformation SwapR1AndR2"
                );
            }
            Ok(())
        }
    }
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let read1 = block.read1;
        let read2 = block.read2.take().unwrap();
        block.read1 = read2;
        block.read2 = Some(read1);
        (block, true)
    }
}
