#![allow(clippy::used_underscore_items)]
#![allow(non_camel_case_types)] // to make eserde and _Internal* shut up

use bstr::BString;
use enum_dispatch::enum_dispatch;
use serde_json::json;

use std::{path::Path, thread};

use anyhow::{bail, Result};
use serde_valid::Validate;

use crate::{
    config::{RegionDefinition, Target, TargetPlusAll},
    demultiplex::{DemultiplexInfo, Demultiplexed},
    dna::{HitRegion, TagValue},
    io,
};
use rand::Rng;
use rand::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

mod demultiplex;
mod edits;
mod extract;
mod filters;
mod reports;
mod tag;
mod validation;

/// turn a u64 seed into a 32 byte seed for chacha
fn extend_seed(seed: u64) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();

    // Extend the seed_bytes to 32 bytes
    let mut extended_seed = [0u8; 32];
    extended_seed[..8].copy_from_slice(&seed_bytes);
    extended_seed
}

fn reproducible_cuckoofilter<T: std::hash::Hash + ?Sized>(
    seed: u64,
    initial_capacity: usize,
    false_positive_probability: f64,
) -> ScalableCuckooFilter<T, scalable_cuckoo_filter::DefaultHasher, rand_chacha::ChaChaRng> {
    let rng = rand_chacha::ChaChaRng::from_seed(extend_seed(seed));
    scalable_cuckoo_filter::ScalableCuckooFilterBuilder::new()
        .initial_capacity(initial_capacity)
        .false_positive_probability(false_positive_probability)
        .rng(rng)
        .finish()
}

impl TryInto<Target> for TargetPlusAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<Target, Self::Error> {
        match self {
            TargetPlusAll::Read1 => Ok(Target::Read1),
            TargetPlusAll::Read2 => Ok(Target::Read2),
            TargetPlusAll::Index1 => Ok(Target::Index1),
            TargetPlusAll::Index2 => Ok(Target::Index2),
            TargetPlusAll::All => Err(()),
        }
    }
}

/// what's the default character that separates a read name from it's 'is it 1/2/index' illumina
/// style postfix
fn default_name_separator() -> BString {
    b"_".into()
}

#[derive(Debug)]
pub struct FinalizeReportResult {
    pub report_no: usize,
    pub contents: serde_json::Value,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct InputInfo {
    pub has_read1: bool,
    pub has_read2: bool,
    pub has_index1: bool,
    pub has_index2: bool,
}

#[enum_dispatch(Transformation)]
pub trait Step {
    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    // if it's a tag setting step, what tag does it set?
    fn sets_tag(&self) -> Option<String> {
        None
    }

    // if it's a tag removing step, what tag does it remove?
    fn removes_tag(&self) -> Option<String> {
        None
    }

    // what tags does this step use?
    fn uses_tags(&self) -> Option<Vec<String>> {
        None
    }

    // this tag provides a .location entry. (most do).
    fn tag_provides_location(&self) -> bool {
        true
    }

    // this tag only works with tags that have an is_some(.location)
    fn tag_requires_location(&self) -> bool {
        false
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        Ok(None)
    }
    fn finalize(
        &mut self,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        Ok(None)
    }
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool);

    /// does this transformation need to see all reads, or is it fine to run it in multiple
    /// threads in parallel?
    fn needs_serial(&self) -> bool {
        false
    }

    /// When we have a transformation that says 'Enough reads'
    /// like Head, that sends the 'end transmission' signal
    /// upstream by closing it's receiver.
    /// Then the next upstream stage detects that, and can
    /// close it's receiver in turn (and ending it's processing).
    /// Except if this returns false,
    /// because then we stop the breakage here,
    /// accepting all incoming reads and discarding the after processing
    fn transmits_premature_termination(&self) -> bool {
        true
    }
}

/// A transformation that delays processing
/// by a random amount.
/// Used to inject chaos into test cases.
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct _InternalDelay {
    #[serde(skip)]
    rng: Option<rand_chacha::ChaChaRng>,
}

impl Step for Box<_InternalDelay> {
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        if self.rng.is_none() {
            let seed = block_no; //needs to be reproducible, but different for each block
            let seed_bytes = seed.to_le_bytes();

            // Extend the seed_bytes to 32 bytes
            let mut extended_seed = [0u8; 32];
            extended_seed[..8].copy_from_slice(&seed_bytes);
            let rng = rand_chacha::ChaCha20Rng::from_seed(extended_seed);
            self.rng = Some(rng);
        }

        let rng = self.rng.as_mut().unwrap();
        let delay = rng.random_range(0..10);
        thread::sleep(std::time::Duration::from_millis(delay));
        (block, true)
    }
}

/// An internal read counter, similar to `report::_ReportCount`
/// but it does not block premature termination.
/// We use this to test the head->early termination -> premature termination logic
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct _InternalReadCount {
    label: String,
    #[serde(skip)]
    report_no: usize,
    #[serde(skip)]
    count: usize,
}

impl Step for Box<_InternalReadCount> {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        true // That's the magic as opposed to the usual reports
    }
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        self.count += block.read1.entries.len();
        (block, true)
    }
    fn finalize(
        &mut self,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        contents.insert("_InternalReadCount".to_string(), json!(self.count));

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}

type OurCuckCooFilter<T> = scalable_cuckoo_filter::ScalableCuckooFilter<
    T,
    scalable_cuckoo_filter::DefaultHasher,
    rand_chacha::ChaChaRng,
>;

#[derive(Hash, Debug)]
pub struct FragmentEntry<'a>(
    &'a [u8],
    Option<&'a [u8]>,
    Option<&'a [u8]>,
    Option<&'a [u8]>,
);

#[derive(Hash, Debug)]
pub struct FragmentEntryForCuckooFilter(FragmentEntry<'static>);

impl<'a> std::borrow::Borrow<FragmentEntry<'a>> for FragmentEntryForCuckooFilter {
    fn borrow(&self) -> &FragmentEntry<'a> {
        &self.0
    }
}

impl FragmentEntry<'_> {
    fn to_continuous_vec(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        res.extend(self.0);
        if let Some(read2) = self.1 {
            res.extend(read2.iter());
        }
        if let Some(index1) = self.2 {
            res.extend(index1.iter());
        }
        if let Some(index2) = self.3 {
            res.extend(index2.iter());
        }
        res
    }
}

#[derive(eserde::Deserialize, Debug, Validate, Clone, PartialEq, Eq, Copy)]
pub enum KeepOrRemove {
    #[serde(alias = "keep")]
    Keep,
    #[serde(alias = "remove")]
    Remove,
}

#[derive(eserde::Deserialize, Debug, Clone, strum_macros::Display)]
#[serde(tag = "action")]
#[enum_dispatch]
pub enum Transformation {
    //Edits
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    MaxLen(edits::MaxLen),
    Prefix(edits::Prefix),
    Postfix(edits::Postfix),
    ConvertPhred64To33(edits::Phred64To33),
    ReverseComplement(edits::ReverseComplement),
    Rename(edits::Rename),
    TrimQualityStart(edits::TrimQualityStart),
    TrimQualityEnd(edits::TrimQualityEnd),
    SwapR1AndR2(edits::SwapR1AndR2),
    LowercaseTag(edits::LowercaseTag),
    UppercaseTag(edits::UppercaseTag),
    LowercaseSequence(edits::LowercaseSequence),
    UppercaseSequence(edits::UppercaseSequence),
    TrimAtTag(edits::TrimAtTag),

    FilterByTag(filters::ByTag),
    FilterByNumericTag(filters::ByNumericTag),
    FilterByBoolTag(filters::ByBoolTag),

    //Filters
    Head(filters::Head),
    Skip(filters::Skip),
    FilterEmpty(filters::Empty),
    FilterSample(filters::Sample),
    //
    //Validation
    ValidateSeq(validation::ValidateSeq),
    ValidatePhred(validation::ValidatePhred),

    // tag based stuff
    ExtractIUPAC(extract::IUPAC),
    ExtractRegex(extract::Regex),
    ExtractRegion(extract::Region), //gets converted into ExtractRegions
    ExtractRegions(extract::Regions),
    ExtractAnchor(extract::Anchor),
    ExtractLength(extract::Length),
    ExtractMeanQuality(extract::MeanQuality),
    ExtractGCContent(extract::GCContent),
    ExtractNCount(extract::NCount),
    ExtractLowComplexity(extract::LowComplexity),
    ExtractQualifiedBases(extract::QualifiedBases),
    ExtractRegionsOfLowQuality(extract::RegionsOfLowQuality),
    ExtractPolyTail(extract::PolyTail),
    ExtractIUPACSuffix(extract::IUPACSuffix),
    // bool tags
    TagDuplicates(extract::tag::Duplicates),
    TagOtherFileByName(extract::tag::OtherFileByName),
    TagOtherFileBySequence(extract::tag::OtherFileBySequence),

    //edit
    StoreTagInSequence(tag::StoreTagInSequence),
    ReplaceTagWithLetter(tag::ReplaceTagWithLetter),

    //store
    RemoveTag(tag::RemoveTag),
    StoreTagInComment(tag::StoreTagInComment),
    StoreTagLocationInComment(tag::StoreTaglocationInComment),
    StoreTagsInTable(tag::StoreTagsInTable),
    //other
    QuantifyTag(tag::QuantifyTag),

    Progress(reports::Progress),
    Report(reports::Report),
    #[serde(skip)]
    _ReportCount(Box<reports::_ReportCount>),
    #[serde(skip)]
    _ReportLengthDistribution(Box<reports::_ReportLengthDistribution>),
    #[serde(skip)]
    _ReportDuplicateCount(Box<reports::_ReportDuplicateCount>),
    #[serde(skip)]
    _ReportDuplicateFragmentCount(Box<reports::_ReportDuplicateFragmentCount>),

    #[serde(skip)]
    _ReportBaseStatisticsPart1(Box<reports::_ReportBaseStatisticsPart1>),
    #[serde(skip)]
    _ReportBaseStatisticsPart2(Box<reports::_ReportBaseStatisticsPart2>),
    #[serde(skip)]
    _ReportCountOligos(Box<reports::_ReportCountOligos>),

    Inspect(reports::Inspect),

    Demultiplex(demultiplex::Demultiplex),

    _InternalDelay(Box<_InternalDelay>),
    _InternalReadCount(Box<_InternalReadCount>),
}

pub(crate) fn validate_target(target: Target, input_def: &crate::config::Input) -> Result<()> {
    match target {
        Target::Read1 => {}
        Target::Read2 => {
            if input_def.read2.is_none() {
                bail!("Read2 is not defined in the input section, but used by transformation");
            }
        }
        Target::Index1 => {
            if input_def.index1.is_none() {
                bail!("Index1 is not defined in the input section, but used by transformation");
            }
        }
        Target::Index2 => {
            if input_def.index2.is_none() {
                bail!("Index2 is not defined in the input section, but used by transformation");
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_target_plus_all(
    target: TargetPlusAll,
    input_def: &crate::config::Input,
) -> Result<()> {
    match target {
        TargetPlusAll::All | TargetPlusAll::Read1 => {}
        TargetPlusAll::Read2 => {
            if input_def.read2.is_none() {
                bail!("Read2 is not defined in the input section, but used by transformation");
            }
        }
        TargetPlusAll::Index1 => {
            if input_def.index1.is_none() {
                bail!("Index1 is not defined in the input section, but used by transformation");
            }
        }
        TargetPlusAll::Index2 => {
            if input_def.index2.is_none() {
                bail!("Index2 is not defined in the input section, but used by transformation");
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_dna(dna: &[u8]) -> Result<()> {
    for &base in dna {
        if !matches!(base, b'A' | b'T' | b'C' | b'G') {
            bail!("Invalid base in DNA sequence: {}", base as char);
        }
    }
    Ok(())
}

fn validate_regions(regions: &[RegionDefinition], input_def: &crate::config::Input) -> Result<()> {
    for region in regions {
        validate_target(region.source, input_def)?;

        if region.length == 0 {
            bail!("Length must be > 0");
        }
    }
    Ok(())
}

impl Transformation {
    /// convert the input transformations into those we actually process
    /// (they are mostly the same, but for example reports get split in two
    /// to take advantage of multicore)
    pub fn expand(transforms: Vec<Self>) -> (Vec<Self>, Vec<String>) {
        let mut res = Vec::new();
        let mut res_report_labels = Vec::new();
        let mut report_no = 0;
        for transformation in transforms {
            match transformation {
                Transformation::Report(config) => {
                    res_report_labels.push(config.label);
                    if config.count {
                        res.push(Transformation::_ReportCount(Box::new(
                            reports::_ReportCount::new(report_no),
                        )));
                    }
                    if config.length_distribution {
                        res.push(Transformation::_ReportLengthDistribution(Box::new(
                            reports::_ReportLengthDistribution::new(report_no),
                        )));
                    }
                    if config.duplicate_count_per_read {
                        res.push(Transformation::_ReportDuplicateCount(Box::new(
                            reports::_ReportDuplicateCount {
                                report_no,
                                data_per_read: Vec::default(),
                                debug_reproducibility: config.debug_reproducibility,
                            },
                        )));
                    }
                    if config.duplicate_count_per_fragment {
                        res.push(Transformation::_ReportDuplicateFragmentCount(Box::new(
                            reports::_ReportDuplicateFragmentCount {
                                report_no,
                                data: Vec::default(),
                                debug_reproducibility: config.debug_reproducibility,
                            },
                        )));
                    }

                    if config.base_statistics {
                        {
                            res.push(Transformation::_ReportBaseStatisticsPart1(Box::new(
                                reports::_ReportBaseStatisticsPart1::new(report_no),
                            )));
                            res.push(Transformation::_ReportBaseStatisticsPart2(Box::new(
                                reports::_ReportBaseStatisticsPart2::new(report_no),
                            )));
                        }
                    }
                    if let Some(count_oligos) = config.count_oligos.as_ref() {
                        res.push(Transformation::_ReportCountOligos(Box::new(
                            reports::_ReportCountOligos::new(
                                report_no,
                                count_oligos,
                                config.count_oligos_target,
                            ),
                        )));
                    }

                    report_no += 1;
                }
                Transformation::_InternalReadCount(config) => {
                    let mut config: Box<_> = config.clone();
                    config.report_no = report_no;
                    res_report_labels.push(config.label.clone());
                    report_no += 1;
                    res.push(Transformation::_InternalReadCount(config));
                }
                Transformation::ExtractRegion(config) => {
                    let regions = vec![RegionDefinition {
                        source: config.source,
                        start: config.start,
                        length: config.len,
                    }];
                    res.push(Transformation::ExtractRegions(extract::Regions {
                        label: config.label,
                        regions,
                        region_separator: b"-".into(),
                    }));
                }
                Transformation::FilterEmpty(config) => {
                    // Replace FilterEmpty with ExtractLength + FilterByNumericTag
                    let length_tag_label = format!("_internal_length_{}", res.len());
                    res.push(Transformation::ExtractLength(extract::Length {
                        label: length_tag_label.clone(),
                        target: config.target,
                    }));
                    res.push(Transformation::FilterByNumericTag(filters::ByNumericTag {
                        label: length_tag_label,
                        min_value: Some(1.0), // Non-empty means length >= 1
                        max_value: None,
                        keep_or_remove: KeepOrRemove::Keep,
                    }));
                }
                _ => res.push(transformation),
            }
        }
        (res, res_report_labels)
    }
}

fn extract_regions(
    read_no: usize,
    block: &io::FastQBlocksCombined,
    regions: &[RegionDefinition],
) -> Vec<BString> {
    let mut out: Vec<BString> = Vec::new();
    for region in regions {
        let read = match region.source {
            Target::Read1 => &block.read1,
            Target::Read2 => block.read2.as_ref().unwrap(),
            Target::Index1 => block.index1.as_ref().unwrap(),
            Target::Index2 => block.index2.as_ref().unwrap(),
        }
        .get(read_no);
        let here: BString = read
            .seq()
            .iter()
            .skip(region.start)
            .take(region.length)
            .copied()
            .collect();

        out.push(here);
    }
    out
}

fn apply_in_place(
    target: Target,
    f: impl Fn(&mut io::FastQRead),
    block: &mut io::FastQBlocksCombined,
) {
    match target {
        Target::Read1 => {
            for read in &mut block.read1.entries {
                f(read);
            }
        }
        Target::Read2 => {
            for read in &mut block.read2.as_mut().unwrap().entries {
                f(read);
            }
        }
        Target::Index1 => {
            for read in &mut block.index1.as_mut().unwrap().entries {
                f(read);
            }
        }
        Target::Index2 => {
            for read in &mut block.index2.as_mut().unwrap().entries {
                f(read);
            }
        }
    }
}

fn apply_in_place_wrapped(
    target: Target,
    f: impl FnMut(&mut io::WrappedFastQReadMut),
    block: &mut io::FastQBlocksCombined,
) {
    match target {
        Target::Read1 => block.read1.apply_mut(f),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
    }
}

fn apply_in_place_wrapped_plus_all(
    target: TargetPlusAll,
    mut f: impl FnMut(&mut io::WrappedFastQReadMut),
    block: &mut io::FastQBlocksCombined,
) {
    if let Ok(target) = target.try_into() as Result<Target, _> {
        apply_in_place_wrapped(target, f, block);
    } else {
        apply_in_place_wrapped(Target::Read1, &mut f, block);
        if block.read2.is_some() {
            apply_in_place_wrapped(Target::Read2, &mut f, block);
        }
        if block.index1.is_some() {
            apply_in_place_wrapped(Target::Index1, &mut f, block);
        }
        if block.index2.is_some() {
            apply_in_place_wrapped(Target::Index2, &mut f, block);
        }
    }
}

fn apply_filter(
    target: Target,
    block: &mut io::FastQBlocksCombined,
    f: impl FnMut(&mut io::WrappedFastQRead) -> bool,
) {
    let target = match target {
        Target::Read1 => &block.read1,
        Target::Read2 => block.read2.as_ref().unwrap(),
        Target::Index1 => block.index1.as_ref().unwrap(),
        Target::Index2 => block.index2.as_ref().unwrap(),
    };
    let keep: Vec<_> = target.apply(f);
    apply_bool_filter(block, &keep);
}

fn apply_bool_filter(block: &mut io::FastQBlocksCombined, keep: &[bool]) {
    let mut iter = keep.iter();
    block.read1.entries.retain(|_| *iter.next().unwrap());
    if let Some(ref mut read2) = block.read2 {
        let mut iter = keep.iter();
        read2.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index1) = block.index1 {
        let mut iter = keep.iter();
        index1.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index2) = block.index2 {
        let mut iter = keep.iter();
        index2.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(tags) = block.tags.as_mut() {
        for tag_entries in tags.values_mut() {
            let mut iter = keep.iter();
            tag_entries.retain(|_| *iter.next().unwrap());
        }
    }
}

/* fn apply_filter_all(
    block: &mut io::FastQBlocksCombined,
    mut f: impl FnMut(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> bool,
) {
    let mut keep: Vec<_> = Vec::new();
    let mut block_iter = block.get_pseudo_iter();
    while let Some(molecule) = block_iter.pseudo_next() {
        keep.push(f(
            &molecule.read1,
            molecule.read2.as_ref(),
            molecule.index1.as_ref(),
            molecule.index2.as_ref(),
        ));
    }
    apply_bool_filter(block, &keep);
} */
/*
///apply a filter to one target, or all targets
///Does a logical or - if the function filters in any
///target, the read is removed.
 fn apply_filter_plus_all(
    target: TargetPlusAll,
    block: &mut io::FastQBlocksCombined,
    mut f: impl FnMut(&mut io::WrappedFastQRead) -> bool,
) {
    if let Ok(target) = target.try_into() as Result<Target, _> {
        apply_filter(target, block, f);
    } else {
        apply_filter(Target::Read1, block, &mut f);
        if block.read2.is_some() {
            apply_filter(Target::Read2, block, &mut f);
        }
        if block.index1.is_some() {
            apply_filter(Target::Index1, block, &mut f);
        }
        if block.index2.is_some() {
            apply_filter(Target::Index2, block, &mut f);
        }
    }
} */
//apply one filter to the one target,
//or a different one if targetAll is specified
/* fn apply_filter_plus_all_ext(
    target: TargetPlusAll,
    block: &mut io::FastQBlocksCombined,
    f: impl FnMut(&mut io::WrappedFastQRead) -> bool,
    mut f_all: impl FnMut(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> bool,
) {
    if let Ok(target) = target.try_into() as Result<Target, _> {
        apply_filter(target, block, f);
    } else {
        apply_filter_all(block, &mut f_all);
    }
} */

pub enum NewLocation {
    Remove,
    Keep,
    New(HitRegion),
    NewWithSeq(HitRegion, BString),
}

fn filter_tag_locations(
    block: &mut io::FastQBlocksCombined,
    target: Target,
    f: impl Fn(&HitRegion, usize, &BString, usize) -> NewLocation,
) {
    let reads = match target {
        Target::Read1 => &block.read1.entries,
        Target::Read2 => &block.read2.as_ref().unwrap().entries,
        Target::Index1 => &block.index1.as_ref().unwrap().entries,
        Target::Index2 => &block.index2.as_ref().unwrap().entries,
    };
    if let Some(tags) = block.tags.as_mut() {
        for (_key, value) in tags.iter_mut() {
            for (ii, tag_val) in value.iter_mut().enumerate() {
                let read_length = reads[ii].seq.len();
                if let TagValue::Sequence(hits) = tag_val {
                    let mut any_none = false;
                    for hit in &mut hits.0 {
                        if let Some(location) = hit.location.as_mut() {
                            if location.target == target {
                                let sequence = &hit.sequence;
                                match f(location, ii, sequence, read_length) {
                                    NewLocation::Remove => {
                                        hit.location = None;
                                        any_none = true;
                                        break;
                                    }
                                    NewLocation::Keep => {}
                                    NewLocation::New(new) => *location = new,
                                    NewLocation::NewWithSeq(new_loc, new_seq) => {
                                        *location = new_loc;
                                        hit.sequence = new_seq;
                                    }
                                }
                            }
                        }
                    }
                    // if any are no longer present, remove all location spans
                    if any_none {
                        for hit in &mut hits.0 {
                            hit.location = None;
                        }
                    }
                } else {
                    // no hits, so no location to change
                }
            }
        }
    }
}

fn filter_tag_locations_beyond_read_length(
    block: &mut crate::io::FastQBlocksCombined,
    target: Target,
) {
    filter_tag_locations(
        block,
        target,
        |location: &HitRegion, _pos, _seq, read_len: usize| -> NewLocation {
            //we are already cut to size.
            if location.start + location.len > read_len {
                NewLocation::Remove
            } else {
                NewLocation::Keep
            }
        },
    );
}

fn filter_tag_locations_all_targets(
    block: &mut io::FastQBlocksCombined,
    mut f: impl FnMut(&HitRegion, usize) -> NewLocation,
) {
    //possibly we might need this to pass in all 4 reads.
    //but for now, it's only being used by r1/r2 swap.
    if let Some(tags) = block.tags.as_mut() {
        for (_key, value) in tags.iter_mut() {
            for (ii, tag_val) in value.iter_mut().enumerate() {
                if let TagValue::Sequence(hits) = tag_val {
                    let mut any_none = false;
                    for hit in &mut hits.0 {
                        if let Some(location) = hit.location.as_mut() {
                            match f(location, ii) {
                                NewLocation::Remove => {
                                    hit.location = None;
                                    any_none = true;
                                    break;
                                }
                                NewLocation::Keep => {}
                                NewLocation::New(new) => *location = new,
                                NewLocation::NewWithSeq(new_loc, new_seq) => {
                                    *location = new_loc;
                                    hit.sequence = new_seq;
                                }
                            }
                        }
                    }
                    // if any are no longer present, remove all location spans
                    if any_none {
                        for hit in &mut hits.0 {
                            hit.location = None;
                        }
                    }
                } else {
                    // no hits, so no location to change
                }
            }
        }
    }
}
