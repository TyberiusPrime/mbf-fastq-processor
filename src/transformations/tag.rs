use std::collections::HashMap;

use crate::{
    config::Target,
    dna::{Anchor, Hit},
    io, Demultiplexed,
};
use anyhow::{bail, Context};

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
    query: Vec<u8>,
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
            |read| read.find_iupac(&self.query, self.anchor, self.max_mismatches),
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
                let seq: &[u8] = hit.as_ref().map(|x| x.get(read.seq())).unwrap_or(b"");
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
