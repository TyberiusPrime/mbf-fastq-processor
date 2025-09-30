#![allow(clippy::used_underscore_items)]
#![allow(non_camel_case_types)] // to make eserde and _Internal* shut up

use bstr::BString;
use enum_dispatch::enum_dispatch;
use serde_json::json;

use std::{path::Path, thread};

use anyhow::{bail, Result};
use serde_valid::Validate;

use crate::{
    config::{RegionDefinition, SegmentIndex, SegmentIndexOrAll, SegmentOrAll},
    demultiplex::{DemultiplexInfo, Demultiplexed},
    dna::{HitRegion, TagValue},
    io,
};
use rand::Rng;
use rand::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TagValueType {
    Location,
    Numeric,
    Bool,
}

mod demultiplex;
mod edits;
mod extract;
mod filters;
mod hamming_correct;
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
    pub segment_order: Vec<String>, //todo Reference?
}

#[enum_dispatch(Transformation)]
pub trait Step {
    /// validate just the segments. Needs mut to save their index.
    fn validate_segments(&mut self, _input_def: &crate::config::Input) -> Result<()> {
        Ok(())
    }

    /// validates all other aspects of the step
    /// Needs to see all other transforms to check for conflicts
    /// therefore can't be mut
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    /// Resolve config references like barcode sections
    /// This happens after validation but before init
    fn resolve_config_references(
        &mut self,
        _barcodes: &std::collections::HashMap<String, crate::config::Barcodes>,
    ) -> Result<()> {
        Ok(())
    }

    // if this step sets a tag, what type of tag does it declare?
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
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
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        Ok(None)
    }
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)>;

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
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    rng: Option<rand_chacha::ChaChaRng>,
}

impl Step for Box<_InternalDelay> {
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
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
        Ok((block, true))
    }
}

/// An internal read counter, similar to `report::_ReportCount`
/// but it does not block premature termination.
/// We use this to test the head->early termination -> premature termination logic
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct _InternalReadCount {
    label: String,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    report_no: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
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
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        self.count += block.segments[0].entries.len();
        Ok((block, true))
    }
    fn finalize(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
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
pub struct FragmentEntry<'a>(&'a [&'a [u8]]);

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
        for v in self.0 {
            res.extend(*v);
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
    Truncate(edits::Truncate),
    Prefix(edits::Prefix),
    Postfix(edits::Postfix),
    ConvertPhred(edits::ConvertPhred),
    ReverseComplement(edits::ReverseComplement),
    Rename(edits::Rename),
    Swap(edits::Swap),
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
    ExtractLowQualityStart(extract::LowQualityStart),
    ExtractLowQualityEnd(extract::LowQualityEnd),
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
    StoreTagInFastQ(tag::StoreTagInFastQ),
    StoreTagLocationInComment(tag::StoreTaglocationInComment),
    StoreTagsInTable(tag::StoreTagsInTable),
    //other
    QuantifyTag(tag::QuantifyTag),

    Progress(reports::Progress),
    Report(reports::Report),
    #[serde(skip)] // nodefault
    _ReportCount(Box<reports::_ReportCount>),
    #[serde(skip)] // nodefault
    _ReportLengthDistribution(Box<reports::_ReportLengthDistribution>),
    #[serde(skip)] // nodefault
    _ReportDuplicateCount(Box<reports::_ReportDuplicateCount>),
    #[serde(skip)] // nodefault
    _ReportDuplicateFragmentCount(Box<reports::_ReportDuplicateFragmentCount>),

    #[serde(skip)] // nodefault
    _ReportBaseStatisticsPart1(Box<reports::_ReportBaseStatisticsPart1>),
    #[serde(skip)] // nodefault
    _ReportBaseStatisticsPart2(Box<reports::_ReportBaseStatisticsPart2>),
    #[serde(skip)] // nodefault
    _ReportCountOligos(Box<reports::_ReportCountOligos>),

    Inspect(reports::Inspect),

    Demultiplex(demultiplex::Demultiplex),
    HammingCorrect(hamming_correct::HammingCorrect),

    _InternalDelay(Box<_InternalDelay>),
    _InternalReadCount(Box<_InternalReadCount>),
}

pub(crate) fn validate_dna(dna: &[u8]) -> Result<()> {
    for &base in dna {
        if !matches!(base, b'A' | b'T' | b'C' | b'G') {
            bail!("Invalid base in DNA sequence: {}", base as char);
        }
    }
    Ok(())
}

fn validate_regions(
    regions: &mut [RegionDefinition],
    input_def: &crate::config::Input,
) -> Result<()> {
    for region in regions {
        region.segment_index = Some(region.segment.validate(input_def)?);

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
                                config.count_oligos_segment_index.unwrap(),
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
                        segment: config.segment,
                        segment_index: config.segment_index,
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
                        segment: config.segment,
                        segment_index: config.segment_index,
                    }));
                    res.push(Transformation::FilterByNumericTag(filters::ByNumericTag {
                        label: length_tag_label,
                        min_value: Some(1.0), // Non-empty means length >= 1
                        max_value: None,
                        keep_or_remove: KeepOrRemove::Keep,
                    }));
                }
                Transformation::ConvertPhred(ref config) => {
                    //implies a check beforehand
                    res.push(Transformation::ValidatePhred(validation::ValidatePhred {
                        encoding: config.from,
                        segment: SegmentOrAll("all".to_string()),
                        segment_index: Some(SegmentIndexOrAll::All),
                    }));
                    res.push(transformation);
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
        let read = block.segments[region.segment_index.as_ref().unwrap().get_index()].get(read_no);
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
    segment: SegmentIndex,
    f: impl Fn(&mut io::FastQRead),
    block: &mut io::FastQBlocksCombined,
) {
    for read in &mut block.segments[segment.get_index()].entries {
        f(read);
    }
}

fn apply_in_place_wrapped(
    segment: SegmentIndex,
    f: impl FnMut(&mut io::WrappedFastQReadMut),
    block: &mut io::FastQBlocksCombined,
) {
    block.segments[segment.get_index()].apply_mut(f);
}

fn apply_in_place_wrapped_plus_all(
    segment: SegmentIndexOrAll,
    mut f: impl FnMut(&mut io::WrappedFastQReadMut),
    block: &mut io::FastQBlocksCombined,
) {
    if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
        apply_in_place_wrapped(target, f, block);
    } else {
        for read_block in &mut block.segments {
            read_block.apply_mut(&mut f);
        }
    }
}

/* fn apply_filter(
    segment: &SegmentIndex,
    block: &mut io::FastQBlocksCombined,
    f: impl FnMut(&mut io::WrappedFastQRead) -> bool,
) {
    let segment_block = &block.segments[segment.get_index()];
    let keep: Vec<_> = segment_block.apply(f);
    apply_bool_filter(block, &keep);
} */

fn apply_bool_filter(block: &mut io::FastQBlocksCombined, keep: &[bool]) {
    for segment_block in &mut block.segments {
        let mut iter = keep.iter();
        segment_block.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(tags) = block.tags.as_mut() {
        for tag_entries in tags.values_mut() {
            let mut iter = keep.iter();
            tag_entries.retain(|_| *iter.next().unwrap());
        }
    }
}

pub enum NewLocation {
    Remove,
    Keep,
    New(HitRegion),
    NewWithSeq(HitRegion, BString),
}

fn filter_tag_locations(
    block: &mut io::FastQBlocksCombined,
    segment: SegmentIndex,
    f: impl Fn(&HitRegion, usize, &BString, usize) -> NewLocation,
) {
    let reads = &block.segments[segment.get_index()].entries;
    if let Some(tags) = block.tags.as_mut() {
        for (_key, value) in tags.iter_mut() {
            for (ii, tag_val) in value.iter_mut().enumerate() {
                let read_length = reads[ii].seq.len();
                if let TagValue::Sequence(hits) = tag_val {
                    let mut any_none = false;
                    for hit in &mut hits.0 {
                        if let Some(location) = hit.location.as_mut() {
                            if location.segment_index == segment {
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
    segment: SegmentIndex,
) {
    filter_tag_locations(
        block,
        segment,
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
