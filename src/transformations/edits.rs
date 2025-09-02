use super::{
    NewLocation, Step, Target, Transformation, apply_in_place, apply_in_place_wrapped,
    filter_tag_locations, filter_tag_locations_all_targets,
    filter_tag_locations_beyond_read_length, validate_target,
};
use crate::{
    config::deser::{
        base_or_dot, dna_from_string, u8_from_char_or_number, u8_from_string, u8_regex_from_string,
    },
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::{Result, bail};
use serde_valid::Validate;

/* fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
} */

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

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        target: location.target,
                    })
                }
            },
        );

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
        filter_tag_locations_beyond_read_length(&mut block, self.target);

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
        filter_tag_locations_beyond_read_length(&mut block, self.target);
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
        let prefix_len = self.seq.len();

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                {
                    NewLocation::New(HitRegion {
                        start: location.start + prefix_len,
                        len: location.len,
                        target: location.target,
                    })
                }
            },
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
        // postfix doesn't change tags.
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

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, _pos, seq: &Vec<u8>, read_len: usize| -> NewLocation {
                {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            target: location.target,
                        },
                        new_seq,
                    )
                }
            },
        );

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
        //no tag change.
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
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
//TODO: Remove because of tags.
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
//todo: consider turning this into an extract and TrimATTag instead.
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
                read.trim_poly_base_suffix(
                    self.min_length,
                    self.max_mismatch_rate,
                    self.max_consecutive_mismatches,
                    self.base,
                );
            },
            &mut block,
        );
        filter_tag_locations_beyond_read_length(&mut block, self.target);
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
        let mut cut_off = Vec::new();
        {
            let edit_cut_off = &mut cut_off;
            apply_in_place_wrapped(
                self.target,
                |read| {
                    let read_len = read.len();
                    read.trim_quality_start(self.min);
                    let lost = read_len - read.len();
                    edit_cut_off.push(lost);
                },
                &mut block,
            );
        }

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, pos, _seq, _read_len: usize| -> NewLocation {
                let lost = cut_off[pos];
                if location.start < lost {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - lost,
                        len: location.len,
                        target: location.target,
                    })
                }
            },
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
        filter_tag_locations_beyond_read_length(&mut block, self.target);
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

        filter_tag_locations_all_targets(&mut block, |location: &HitRegion, _pos: usize| -> NewLocation {
            NewLocation::New(HitRegion {
                start: location.start,
                len: location.len,
                target: match location.target {
                    Target::Read1 => Target::Read2,
                    Target::Read2 => Target::Read1,
                    _ => location.target, // Indexes remain unchanged
                },
            })
        });

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseTag {
    label: String,
}

impl Step for LowercaseTag {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let hits = block
            .tags
            .as_mut()
            .and_then(|tags| tags.get_mut(&self.label))
            .expect("Tag missing. Should been caught earlier.");
        for hit in hits.iter_mut().flatten() {
            for hit_region in hit.0.iter_mut() {
                for ii in 0..hit_region.sequence.len() {
                    hit_region.sequence[ii] = hit_region.sequence[ii].to_ascii_lowercase();
                }
            }
        }

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UppercaseTag {
    label: String,
}

impl Step for UppercaseTag {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let hits = block
            .tags
            .as_mut()
            .and_then(|tags| tags.get_mut(&self.label))
            .expect("Tag missing. Should been caught earlier.");
        for hit in hits.iter_mut().flatten() {
            for hit_region in hit.0.iter_mut() {
                for ii in 0..hit_region.sequence.len() {
                    hit_region.sequence[ii] = hit_region.sequence[ii].to_ascii_uppercase();
                }
            }
        }

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseSequence {
    target: Target,
}

impl Step for LowercaseSequence {
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
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UppercaseSequence {
    target: Target,
}

impl Step for UppercaseSequence {
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
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_uppercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
        );

        (block, true)
    }
}
