use std::{collections::HashMap, path::Path};

use crate::{
    config::{
        deser::{u8_from_string, u8_regex_from_string},
        Target,
    },
    dna::{Anchor, Hit},
    io, Demultiplexed,
};
use anyhow::Result;
use serde_valid::Validate;

use super::{extract_regions, FinalizeReportResult, RegionDefinition, Step, Transformation};

fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
}

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
                    Some(Hit::new(
                        g.start(),
                        g.end() - g.start(),
                        self.target,
                        replacement,
                    ))
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
        block.apply_mut_with_tag(&self.label, |read1, read2, index1, index2, hit| {
            if let Some(hit) = hit {
                for region in &hit.regions {
                    let read: &mut crate::io::WrappedFastQReadMut = match region.target {
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
                    read.cut_start(region.start);
                    /* let seq = read.seq_mut();
                    for ii in region.start..region.start + region.len {
                        seq[ii] = seq[ii].to_ascii_lowercase();
                    } */
                }
            }
        });

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FilterByTag {
    label: String,
    keep_or_remove: super::KeepOrRemove,
}

impl Step for FilterByTag {
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
pub struct TrimAtTag {
    label: String,
    direction: Direction,
    keep_tag: bool,
}

impl Step for TrimAtTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.apply_mut_with_tag(
            self.label.as_str(),
            |read1, read2, index1, index2, tag_hit| {
                if let Some(hit) = tag_hit {
                    assert_eq!(hit.regions.len(), 1, "TrimAtTag only supports Tags that cover one single region. Could be extended to multiple tags within one target, but not to multiple hits in multiple targets.");
                    let region = &hit.regions[0];
                    let read = match region.target {
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
                        (Direction::Start, true) => read.cut_start(region.start),
                        (Direction::Start, false) => read.cut_start(region.start + region.len),
                        (Direction::End, true) => read.max_len(region.start + region.len),
                        (Direction::End, false) => read.max_len(region.start),
                    }
                }
            },
        );
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegion {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,

    pub label: String,

    /* #[serde(
        deserialize_with = "u8_from_string",
        default = "default_readname_end_chars"
    )]
    pub readname_end_chars: Vec<u8>,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_separator"
    )]
    pub separator: Vec<u8>,
    */
    #[serde(
        deserialize_with = "u8_from_string",
        default = "super::default_name_separator"
    )]
    pub region_separator: Vec<u8>,
}

impl Step for ExtractRegion {
    fn uses_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

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
        /* let rename_read = |read: &mut crate::io::WrappedFastQReadMut, extracted: &Vec<u8>| {
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
        }; */
        if block.tags.is_none() {
            block.tags = Some(HashMap::new());
        }
        let mut out = Vec::new();
        let mod_region_def = self
            .regions
            .iter()
            .map(|x| crate::dna::HitRegion {
                target: x.source,
                start: x.start,
                len: x.length,
            })
            .collect::<Vec<_>>();

        for ii in 0..block.len() {
            //todo: handling if the read is shorter than the regions
            //todo: add test case if read is shorter than the regions
            let extracted = extract_regions(ii, &block, &self.regions, &self.region_separator);
            out.push(Some(Hit::new_with_regions_and_replacement(
                mod_region_def.clone(),
                extracted,
            )));
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label.to_string(), out);

        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInR1Comment {
    label: String,
}

impl Step for StoreTagInR1Comment {
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
                //todo: This is wrong, we need to check the target per hit...
                let seq: &[u8] = hit.as_ref().map(|x| &x.replacement[..]).unwrap_or(b"");
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
pub struct RemoveTag {
    label: String,
}

impl Step for RemoveTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
    }

    fn removes_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.tags.as_mut().map(|tags| {
            tags.remove(&self.label);
        });
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
enum SupportedTableFormats {
    TSV,
    JSON,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    table_filename: String,
    format: SupportedTableFormats,

    #[serde(skip)]
    store: HashMap<String, Vec<String>>,
}

impl Step for StoreTagsInTable {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.tags.as_mut().map(|tags| {
            //store read names...
            {
                if self.store.is_empty() {
                    self.store.insert("ReadName".to_string(), Vec::new());
                }
                let target = self.store.get_mut("ReadName").unwrap();
                let mut iter = block.read1.get_pseudo_iter();
                while let Some(read) = iter.pseudo_next() {
                    target.push(
                        std::str::from_utf8(read.name())
                            .expect("Invalid UTF-8 in read name")
                            .to_string(),
                    );
                }
            }
            for (key, values) in tags.iter() {
                if !self.store.contains_key(key) {
                    self.store.insert(key.clone(), Vec::new());
                }
                let target = self.store.get_mut(key).expect("Key should be present");
                for value in values {
                    if let Some(hit) = value {
                        target.push(
                            std::string::String::from_utf8_lossy(&hit.replacement).to_string(),
                        );
                    } else {
                        target.push(String::new());
                    }
                }
            }
        });

        (block, true)
    }
    fn finalize(
        &mut self,
        _output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let order = ["ReadName".to_string()]
            .into_iter()
            .chain(self.store.keys().filter(|x| *x != "ReadName"))
            .cloned()
            .collect::<Vec<_>>();

        let file_handle = std::fs::File::create(output_directory.join(&self.table_filename))?;
        let buffered_writer = std::io::BufWriter::new(file_handle);

        match self.format {
            SupportedTableFormats::TSV => {
                let mut writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(buffered_writer);
                writer.write_record(&order)?;
                for i in 0..self.store.values().next().map_or(0, |v| v.len()) {
                    let mut record = Vec::new();
                    for key in &order {
                        if let Some(values) = self.store.get(key) {
                            if i < values.len() {
                                record.push(values[i].clone());
                            } else {
                                record.push(String::new());
                            }
                        } else {
                            record.push(String::new());
                        }
                    }
                    writer.write_record(record)?;
                }
                writer.flush()?;
            }
            SupportedTableFormats::JSON => {
                serde_json::to_writer(buffered_writer, &self.store)?;
            }
        }

        todo!();
        Ok(None)
    }
}
