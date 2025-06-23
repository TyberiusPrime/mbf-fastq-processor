use std::{collections::HashMap, io::BufWriter, path::Path};

use crate::{
    config::{
        deser::{u8_from_string, u8_regex_from_string},
        Target, TargetPlusAll,
    },
    dna::{Anchor, Hit, HitRegion},
    io, Demultiplexed,
};
use anyhow::{bail, Result};
use serde_valid::Validate;

use super::{extract_regions, FinalizeReportResult, RegionDefinition, Step, Transformation};
/*
fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
} */

fn default_region_separator() -> Vec<u8> {
    b"-".to_vec()
}
fn default_target_read1() -> TargetPlusAll {
    TargetPlusAll::Read1
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

///Extract a IUPAC described sequence from the read. E.g. an adapter.
///Can be at the start (anchor = Left, the end (anchor = Right),
///or anywhere (anchor = Anywhere) within the read.
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
    target: TargetPlusAll,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &Option<Hit>),
) {
    match target {
        TargetPlusAll::Read1 => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f)
        }
        TargetPlusAll::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::All => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            if let Some(read2) = &mut block.read2 {
                read2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index1) = &mut block.index1 {
                index1.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index2) = &mut block.index2 {
                index2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseTag {
    label: String,
}

impl Step for LowercaseTag {
    fn uses_tag(&self) -> Option<String> {
        self.label.clone().into()
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
        for hit in hits.iter_mut() {
            if let Some(hit) = hit {
                for hit_region in hit.0.iter_mut() {
                    //lowercase the region
                    for ii in 0..hit_region.sequence.len() {
                        hit_region.sequence[ii] = hit_region.sequence[ii].to_ascii_lowercase();
                    }
                }
            }
        }

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

    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
    ) -> Result<()> {
        for transformation in all_transforms {
            if let Transformation::ExtractRegion(extract_region_config) = transformation {
                if extract_region_config.label == self.label {
                    if extract_region_config.regions.len() != 1 {
                        bail!(
                            "ExtractRegion and TrimAtTag only work together on single-entry regions. Label involved: {}", self.label
                        );
                    }
                }
            }
        }
        Ok(())
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
                    assert_eq!(hit.0.len(), 1, "TrimAtTag only supports Tags that cover one single region. Could be extended to multiple tags within one target, but not to multiple hits in multiple targets.");
                    let region = &hit.0[0];
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

///Extract regions, that is by (target|source, 0-based start, length)
///defined triplets, joined with (possibly empty) separator.
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
    fn sets_tag(&self) -> Option<String> {
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
        //todo: handling if the read is shorter than the regions
        //todo: add test case if read is shorter than the regions
        if block.tags.is_none() {
            block.tags = Some(HashMap::new());
        }
        let mut out = Vec::new();
        for ii in 0..block.len() {
            let extracted = extract_regions(ii, &block, &self.regions);
            let mut h: Vec<HitRegion> = Vec::new();
            for (region, seq) in self.regions.iter().zip(extracted) {
                h.push(HitRegion {
                    target: region.source,
                    start: region.start,
                    len: region.length,
                    sequence: seq,
                });
            }
            out.push(Some(Hit::new_multiple(h)));
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label.to_string(), out);

        (block, true)
    }
}

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///Does work with ExtractRegion and multiple regions.
///
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInSequence {
    label: String,
}

impl Step for StoreTagInSequence {
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
                for region in &hit.0 {
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
                    let seq = read.seq();
                    let mut new_seq: Vec<u8> = Vec::new();
                    new_seq.extend_from_slice(&seq[..region.start]);
                    new_seq.extend_from_slice(&region.sequence);
                    new_seq.extend_from_slice(&seq[region.start + region.len..]);

                    let mut new_qual: Vec<u8> = Vec::new();
                    new_qual.extend_from_slice(&read.qual()[..region.start]);
                    if region.sequence.len() == region.len {
                        //if the sequence is the same length as the region excised, we can just copy the quality
                        new_qual.extend_from_slice(
                            &read.qual()[region.start..region.start + region.len],
                        );
                    } else {
                        //otherwise, we need replace it with the average quality, repeated
                        let avg_qual = if !region.is_empty() {
                            let avg_qual = read.qual()[region.start..region.start + region.len]
                                .iter()
                                .map(|&x| x as u32)
                                .sum::<u32>() as f64
                                / region.len as f64;
                            avg_qual.round() as u8
                        } else {
                            b'B'
                        };
                        new_qual.extend_from_slice(&vec![avg_qual; region.sequence.len()]);
                    }
                    new_qual.extend_from_slice(&read.qual()[region.start + region.len..]);

                    read.replace_seq(new_seq, new_qual)
                }
            }
        });
        (block, true)
    }
}

/// Store currently present tags as comments on read1's name.
/// Comments are key=value pairs, separated by spaces
/// from each other, and from the read name.
///
/// (Aligners often keep only the read name).
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInComment {
    label: String,
    #[serde(default = "default_target_read1")]
    target: TargetPlusAll,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "u8_from_string")]
    region_separator: Vec<u8>,
}

impl Step for StoreTagInComment {
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
            self.target,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, hit: &Option<Hit>| {
                let name = std::str::from_utf8(read.name()).expect("Invalid UTF-8 in read name");
                let seq: Vec<u8> = hit
                    .as_ref()
                    .map(|x| x.joined_sequence(Some(&self.region_separator)))
                    .unwrap_or_else(|| {
                        //if the tag is not present, we use an empty sequence
                        Vec::new()
                    });
                let seq = std::str::from_utf8(&seq).expect("Invalid UTF-8 in DNA sequence");
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

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "u8_from_string")]
    region_separator: Vec<u8>,
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
                            std::string::String::from_utf8_lossy(
                                &hit.joined_sequence(Some(&self.region_separator)),
                            )
                            .to_string(),
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
        let mut order: Vec<_> = ["ReadName"]
            .into_iter()
            .chain(
                self.store
                    .keys()
                    .map(|x| x.as_str())
                    .filter(|x| *x != "ReadName"),
            )
            .to_owned()
            .collect();
        order.sort();
        let order: Vec<String> = order.into_iter().map(|x| x.to_string()).collect();

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

        Ok(None)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct QuantifyTag {
    pub infix: String,
    pub label: String,

    #[serde(skip)]
    pub collector: HashMap<Vec<u8>, usize>,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "u8_from_string")]
    region_separator: Vec<u8>,
}

impl Step for QuantifyTag {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn uses_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let collector = &mut self.collector;
        let hits = block
            .tags
            .as_ref()
            .expect("No tags in block: bug")
            .get(&self.label)
            .expect("Tag not found. Should have been caught in validation");
        for hit in hits {
            if let Some(hit) = hit {
                *collector
                    .entry(hit.joined_sequence(Some(&self.region_separator)))
                    .or_insert(0) += 1;
            }
        }
        (block, true)
    }

    fn finalize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        use std::io::Write;
        let infix = &self.infix;
        let report_file = std::fs::File::create(
            output_directory.join(format!("{output_prefix}_{infix}.qr.json")),
        )?;
        let mut bufwriter = BufWriter::new(report_file);
        let str_collector: HashMap<String, usize> = self
            .collector
            .iter()
            .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
            .collect();
        let json = serde_json::to_string_pretty(&str_collector)?;
        bufwriter.write_all(json.as_bytes())?;
        Ok(None)
    }
}
