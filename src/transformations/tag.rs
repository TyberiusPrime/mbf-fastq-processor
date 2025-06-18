use std::collections::HashMap;


use crate::{
    config::{
        deser::{u8_from_string, u8_regex_from_string},
        Target,
    },
    dna::{Anchor, Hit},
    io, Demultiplexed,
};

use super::Step;

fn extract_tags(
    target: Target,
    label: &str,
    f: impl Fn(&mut io::WrappedFastQRead) -> Option<Hit>,
    block: &mut io::FastQBlocksCombined,
) {
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }
    let mut out = Vec::new();

    let f2 = |read: &mut io::WrappedFastQRead| out.push(f(read));

    match target {
        Target::Read1 => block.read1.apply(f2),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
    };
    block.tags.as_mut().unwrap().insert(label.to_string(), out);
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractIUPAC {
    #[serde(deserialize_with = "crate::config::deser::iupac_from_string")]
    search: Vec<u8>,
    pub target: Target,
    anchor: Anchor,
    label: String,
    #[serde(default)] // 0 is fine.
    max_mismatches: u8,
}

impl Step for ExtractIUPAC {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
    ) -> anyhow::Result<()> {
        super::validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(
            self.target,
            &self.label,
            |read| read.find_iupac(&self.search, self.anchor, self.max_mismatches, self.target),
            &mut block,
        );

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegex {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "u8_from_string")]
    pub replacement: Vec<u8>,
    label: String,
    pub target: Target,
}

impl Step for ExtractRegex {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
    ) -> anyhow::Result<()> {
        super::validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(
            self.target,
            &self.label,
            |read| {
                let re_hit = self.search.captures(read.seq());
                if let Some(hit) = re_hit {
                    let mut replacement = Vec::new();
                    let g = hit.get(0).expect("Regex should always match");
                    hit.expand(&self.replacement, &mut replacement);
                    Some(Hit {
                        start: g.start(),
                        len: g.end() - g.start(),
                        target: self.target,
                        replacement: Some(replacement),
                    })
                } else {
                    None
                }
            },
            &mut block,
        );

        (block, true)
    }
}

fn apply_in_place_wrapped_with_tag(
    target: Target,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &Option<Hit>),
) {
    match target {
        Target::Read1 => block
            .read1
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TagSequenceToName {
    label: String,
}

impl Step for TagSequenceToName {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_with_tag(
            Target::Read1,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, hit: &Option<Hit>| {
                let name = std::str::from_utf8(read.name()).expect("Invalid UTF-8 in read name");
                let seq: &[u8] = hit.as_ref().map(|x| x.replacement_or_seq(read.seq())).unwrap_or(b"");
                let seq = std::str::from_utf8(seq).expect("Invalid UTF-8 in DNA sequence");
                let new_name = format!(
                    "{name} {label}={value}",
                    name = name,
                    label = self.label,
                    value = seq
                );
                read.replace_name(new_name.as_bytes().to_vec());
            },
        );

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowerCaseTag {
    label: String,
}

impl Step for LowerCaseTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_with_tag(
            Target::Read1,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, hit: &Option<Hit>| {
                if let Some(hit) = hit {
                    let s = read.seq_mut();
                    for ii in hit.start..hit.start + hit.len {
                        s[ii] = s[ii].to_ascii_lowercase();
                    }
                }
            },
        );

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FilterTag {
    label: String,
    keep_or_remove: super::KeepOrRemove,
}

impl Step for FilterTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let mut keep: Vec<bool> = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .map_or_else(
                || vec![false; block.read1.len()],
                |hits| hits.iter().map(|hit| hit.is_some()).collect(),
            );
        if self.keep_or_remove == super::KeepOrRemove::Remove {
            keep.iter_mut().for_each(|x| *x = !*x);
        }
        super::apply_bool_filter(&mut block, keep);

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Eq, PartialEq, Copy)]
enum Direction {
    Start,
    End,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimTag {
    label: String,
    direction: Direction,
    keep_tag: bool,
}

impl Step for TrimTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        //TODO: This must be target specific!
        block.apply_mut_with_tag(
            self.label.as_str(),
            |read1, read2, index1, index2, tag_hit| {
                if let Some(hit) = tag_hit {
                    let read = match hit.target {
                        Target::Read1 => read1,
                        Target::Read2 => read2
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                        Target::Index1 => index1
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                        Target::Index2 => index2
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                    };
                    match (self.direction, self.keep_tag) {
                        (Direction::Start, true) => read.cut_start(hit.start),
                        (Direction::Start, false) => read.cut_start(hit.start + hit.len),
                        (Direction::End, true) => read.max_len(hit.start + hit.len),
                        (Direction::End, false) => read.max_len(hit.start),
                    }
                }
            },
        );
        (block, true)
    }
}
