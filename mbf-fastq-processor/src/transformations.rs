#![allow(clippy::used_underscore_items)]
#![allow(non_camel_case_types)] // to make eserde and _Internal* shut up

use bstr::BString;
use enum_dispatch::enum_dispatch;
use prelude::TagMetadata;
use schemars::JsonSchema;

use std::{collections::BTreeMap, path::Path};

use anyhow::{Result, bail};

use crate::{
    config::{
        self, Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll,
        deser::{
            ErrorCollector, ErrorCollectorExt, FromTomlTable, FromTomlTableNested, TomlResult,
        },
    },
    demultiplex::{DemultiplexBarcodes, OptDemultiplex},
    dna::TagValue,
    io,
};
use rand::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

pub(crate) mod calc;
mod conditional_tag;
pub(crate) mod convert;
pub(crate) mod demultiplex;
pub(crate) mod edits;
pub(crate) mod extract;
pub(crate) mod filters;
pub(crate) mod hamming_correct;
mod internal_steps;
pub(crate) mod prelude;
pub(crate) mod reports;
pub(crate) mod tag;
pub(crate) mod validation;
pub use conditional_tag::ConditionalTag;
pub use internal_steps::{_InduceFailure, _InternalDelay, _InternalReadCount};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[serde(alias = "segment")]
    pub source: String,
    #[serde(default)]
    #[serde(skip)]
    pub resolved_source: Option<ResolvedSourceNoAll>,

    pub start: isize,
    pub length: usize,

    /// Whether the start position is anchored to the start (default) or end of the region
    pub anchor: RegionAnchor,
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
pub enum RegionAnchor {
    #[serde(alias = "start")]
    #[serde(alias = "Left")]
    #[serde(alias = "left")]
    Start,
    #[serde(alias = "end")]
    #[serde(alias = "Right")]
    #[serde(alias = "right")]
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TagValueType {
    Location, // string + in-sequence-location
    String,   // just a piece of text
    Numeric,
    Bool,
}

impl TagValueType {
    pub fn compatible(self, other: TagValueType) -> bool {
        self == other
    }
}

impl std::fmt::Display for TagValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValueType::Location => write!(f, "Location"),
            TagValueType::String => write!(f, "String"),
            TagValueType::Numeric => write!(f, "Numeric"),
            TagValueType::Bool => write!(f, "Boolean"),
        }
    }
}

/// turn a u64 seed into a 32 byte seed for chacha
fn extend_seed(seed: u64) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();

    // Extend the seed_bytes to 32 bytes
    let mut extended_seed = [0u8; 32];
    extended_seed[..8].copy_from_slice(&seed_bytes);
    extended_seed
}

pub(crate) fn reproducible_cuckoofilter<T: std::hash::Hash + ?Sized>(
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

pub(crate) type OurCuckCooFilter<T> = scalable_cuckoo_filter::ScalableCuckooFilter<
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

#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, Copy, JsonSchema)]
pub enum KeepOrRemove {
    #[serde(alias = "keep")]
    Keep,
    #[serde(alias = "remove")]
    Remove,
}

#[derive(Debug)]
pub struct FinalizeReportResult {
    pub report_no: usize,
    pub contents: serde_json::Value,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct InputInfo {
    pub segment_order: Vec<String>,
    pub barcodes_data: std::collections::BTreeMap<String, crate::config::Barcodes>,
    pub comment_insert_char: u8,
    pub initial_filter_capacity: Option<usize>,
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
    /// happens before expansion
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    fn store_progress_output(&mut self, _progress: &crate::transformations::reports::Progress) {
        //default does nothing
    }

    // if this step sets a tag, what type of tag does it declare?
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        None
    }

    // if it's a tag removing step, what tag does it remove?
    fn removes_tags(&self) -> Vec<String> {
        vec![]
    }

    /// Indicates that this step removes every tag currently available.
    fn removes_all_tags(&self) -> bool {
        false
    }

    // what tags does this step use? What types are acceptable
    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        None
    }

    /// does this step do something to all tags, even if it's not 'using' them in the user sense?
    fn must_see_all_tags(&self) -> bool {
        false
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        Ok(None)
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        Ok(None)
    }
    fn apply(
        &self,
        block: crate::io::FastQBlocksCombined,
        input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)>;

    /// does this transformation need to see all reads, or is it fine to run it in multiple
    /// threads in parallel?
    #[mutants::skip] // since setting this to true will not lead to an error, just longer runtime
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
    /// accepting all incoming reads and discarding them after processing
    fn transmits_premature_termination(&self) -> bool {
        true
    }
}

#[derive(eserde::Deserialize, Debug, strum_macros::Display, JsonSchema)]
#[serde(tag = "action")]
#[enum_dispatch]
pub enum Transformation {
    //Edits
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    Truncate(edits::Truncate),
    Prefix(edits::Prefix),
    Postfix(edits::Postfix),
    ConvertQuality(edits::ConvertQuality),
    ReverseComplement(edits::ReverseComplement),
    Rename(edits::Rename),
    Swap(edits::Swap),
    Lowercase(edits::Lowercase),
    Uppercase(edits::Uppercase),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ChangeCase(edits::_ChangeCase), // public interface is Lowercase/Uppercase
    TrimAtTag(edits::TrimAtTag),
    MergeReads(edits::MergeReads),

    FilterByTag(filters::ByTag),
    FilterByNumericTag(filters::ByNumericTag),

    //Filters
    Head(filters::Head),
    Skip(filters::Skip),
    FilterEmpty(filters::Empty),
    FilterSample(filters::Sample),
    FilterReservoirSample(filters::ReservoirSample),
    //
    //Validation
    #[serde(alias = "SpotCheckReadNames")]
    SpotCheckReadPairing(validation::SpotCheckReadPairing),
    ValidateSeq(validation::ValidateSeq),
    ValidateQuality(validation::ValidateQuality),
    ValidateName(validation::ValidateName),
    ValidateAllReadsSameLength(validation::ValidateAllReadsSameLength),

    // tag based stuff
    ExtractIUPAC(extract::IUPAC),
    ExtractIUPACWithIndel(extract::IUPACWithIndel),
    ExtractRegex(extract::Regex),
    ExtractRegion(extract::Region), //gets converted into ExtractRegions
    ExtractRegions(extract::Regions),
    CalcLength(calc::Length),
    CalcBaseContent(calc::BaseContent),
    CalcGCContent(calc::GCContent),
    CalcNCount(calc::NCount),
    CalcComplexity(calc::Complexity),
    CalcQualifiedBases(calc::QualifiedBases),
    CalcExpectedError(calc::ExpectedError),
    CalcKmers(calc::Kmers),

    ConvertRegionsToLength(convert::RegionsToLength),
    #[serde(alias = "EvaluateExpression")]
    EvalExpression(convert::EvalExpression),
    ExtractRegionsOfLowQuality(extract::RegionsOfLowQuality),
    ExtractLongestPolyX(extract::LongestPolyX),
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
    ConcatTags(tag::ConcatTags),
    ForgetAllTags(tag::ForgetAllTags),
    ForgetTag(tag::ForgetTag),
    StoreTagInComment(tag::StoreTagInComment),
    #[serde(alias = "StoreTagInFASTQ")]
    StoreTagInFastQ(tag::StoreTagInFastQ),
    StoreTagLocationInComment(tag::StoreTagLocationInComment),
    StoreTagsInTable(tag::StoreTagsInTable),
    //other
    QuantifyTag(tag::QuantifyTag),

    Progress(reports::Progress),
    Report(reports::Report),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ReportCount(Box<reports::_ReportCount>),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ReportLengthDistribution(Box<reports::_ReportLengthDistribution>),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ReportDuplicateCount(Box<reports::_ReportDuplicateCount>),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ReportDuplicateFragmentCount(Box<reports::_ReportDuplicateFragmentCount>),

    #[serde(skip)] // nodefault
    _ReportBaseStatisticsPart1(Box<reports::_ReportBaseStatisticsPart1>),
    #[serde(skip)] // nodefault
    _ReportBaseStatisticsPart2(Box<reports::_ReportBaseStatisticsPart2>),
    #[serde(skip)] // nodefault
    _ReportCountOligos(Box<reports::_ReportCountOligos>),
    #[serde(skip)] // nodefault
    #[schemars(skip)]
    _ReportTagHistogram(Box<reports::_ReportTagHistogram>),

    Inspect(reports::Inspect),

    Demultiplex(demultiplex::Demultiplex),
    HammingCorrect(hamming_correct::HammingCorrect),

    #[schemars(skip)]
    _InternalDelay(Box<_InternalDelay>),
    #[schemars(skip)]
    _InternalReadCount(Box<_InternalReadCount>),

    #[schemars(skip)]
    _InduceFailure(Box<_InduceFailure>),
}

impl FromTomlTable for Transformation {
    fn from_toml_table(table: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut helper = collector.local(table);
        let action: String = helper.get("action")?;

        let trafo = match action.as_ref() {
            "CutStart" => {
                Transformation::CutStart(edits::CutStart::from_toml_table(table, helper)?)
            }
            "ValidateSeq" => Transformation::ValidateSeq(validation::ValidateSeq::from_toml_table(
                table, helper,
            )?),

            "ValidateQuality" => Transformation::ValidateQuality(
                validation::ValidateQuality::from_toml_table(table, helper)?,
            ),
            _ => {
                todo!("Need to implement: {}", &action)
            }
        };
        Ok(trafo)
    }
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
    if regions.is_empty() {
        bail!("At least one region must be defined.");
    }
    for region in regions {
        // Handle source vs segment compatibility
        region.resolved_source = Some(ResolvedSourceNoAll::parse(&region.source, input_def)?);
        // if region.length == 0 {
        //     bail!("Length must be > 0");
        // }
        if matches!(
            region
                .resolved_source
                .as_ref()
                .expect("resolved just above"),
            ResolvedSourceNoAll::Segment(_) | ResolvedSourceNoAll::Name { .. }
        ) {
            match region.anchor {
                RegionAnchor::Start => {
                    if region.start < 0 {
                        bail!(
                            "Start position cannot be negative when anchored to start of segment or read name"
                        );
                    }
                }
                RegionAnchor::End => {
                    if region.start >= 0 {
                        bail!(
                            "Start position must be negative when anchored to end of segment or read name"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Coords {
    pub segment_index: SegmentIndex,
    pub start: usize,
    pub length: usize,
}

fn extract_regions(
    read_no: usize,
    block: &io::FastQBlocksCombined,
    regions: &[RegionDefinition],
) -> Vec<Option<(BString, Option<Coords>)>> {
    let mut out: Vec<_> = Vec::new();
    for region in regions {
        let extracted_seq = extract_from_resolved_source(
            read_no,
            block,
            region
                .resolved_source
                .as_ref()
                .expect("Region needs to be resolved first"),
            region.start,
            region.length,
            &region.anchor,
        );

        out.push(extracted_seq);
    }
    out
}

#[allow(clippy::too_many_lines)]
fn extract_from_resolved_source(
    read_no: usize,
    block: &io::FastQBlocksCombined,
    resolved_source: &ResolvedSourceNoAll,
    start: isize,
    length: usize,
    anchor: &RegionAnchor,
) -> Option<(BString, Option<Coords>)> {
    let res: (Option<BString>, Option<Coords>) = match resolved_source {
        ResolvedSourceNoAll::Segment(segment_index) => {
            let read = block.segments[segment_index.get_index()].get(read_no);
            let read_seq = read.seq();
            if let Some((seq, start, length)) =
                extract_from_sequence(read_seq, 0, read_seq.len(), start, length, anchor)
            {
                (
                    Some(seq),
                    Some(Coords {
                        segment_index: *segment_index,
                        start,
                        length,
                    }),
                )
            } else {
                (None, None)
            }
        }
        ResolvedSourceNoAll::Tag(tag_name) => {
            // Extract from tag - we need to get the sequence from the tag
            if let Some(tag_values) = block.tags.get(tag_name) {
                if let Some(tag_value) = tag_values.get(read_no) {
                    match tag_value {
                        TagValue::Location(hits) => {
                            // For location tags, extract from the read sequence!
                            if let Some(hit) = hits.0.first() {
                                if let Some(loc) = &hit.location {
                                    let segment_block =
                                        &block.segments[loc.segment_index.get_index()];
                                    let seq = segment_block.entries[read_no]
                                        .seq
                                        .get(&segment_block.block);
                                    if let Some((seq, start, length)) = extract_from_sequence(
                                        seq,
                                        loc.start,
                                        loc.start + loc.len,
                                        start,
                                        length,
                                        anchor,
                                    ) {
                                        let segment_index =
                                            hit.location.as_ref().map(|loc| loc.segment_index);
                                        (
                                            Some(seq),
                                            segment_index.map(|segment_index| Coords {
                                                segment_index,
                                                start,
                                                length,
                                            }),
                                        )
                                    } else {
                                        (None, None)
                                    }
                                } else {
                                    (None, None)
                                }
                            } else {
                                // has no hits. Fall back to string value if possible
                                let seq = hits.joined_sequence(None);
                                (
                                    extract_from_sequence(
                                        &seq,
                                        0,
                                        seq.len(),
                                        start,
                                        length,
                                        anchor,
                                    )
                                    .map(|x| x.0),
                                    None,
                                )
                            }
                        }
                        TagValue::String(string_val) => {
                            // For string tags, extract from the string value
                            (
                                extract_from_sequence(
                                    string_val.as_ref(),
                                    0,
                                    string_val.len(),
                                    start,
                                    length,
                                    anchor,
                                )
                                .map(|x| x.0),
                                None,
                            )
                        }
                        _ => (None, None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
        ResolvedSourceNoAll::Name {
            segment_index,
            split_character: _,
        } => {
            // Extract from read name
            let read = block.segments[segment_index.get_index()].get(read_no);
            let name = read.name();
            (
                extract_from_sequence(name, 0, name.len(), start, length, anchor).map(|x| x.0),
                None,
            )
        }
    };
    res.0.map(|seq| (seq, res.1))
}

#[allow(clippy::cast_sign_loss)]
fn extract_from_sequence(
    sequence: &[u8],
    sub_sequence_start: usize,
    sub_sequence_end: usize,
    out_start: isize,
    out_length: usize,
    anchor: &RegionAnchor,
) -> Option<(BString, usize, usize)> {
    let seq_len = sequence
        .len()
        .try_into()
        .expect("sequence length does not fit into isize");
    let sub_sequence_start: isize = sub_sequence_start
        .try_into()
        .expect("sub_sequence_start did not fit into isize");
    let sub_sequence_end: isize = sub_sequence_end
        .try_into()
        .expect("sub_sequence_end did not fit into isize");

    // Calculate the actual start position
    let actual_start: isize = match anchor {
        RegionAnchor::Start => {
            // For start anchoring, negative values count from the beginning
            (out_start + sub_sequence_start).min(seq_len)
        }
        RegionAnchor::End => {
            // For end anchoring, negative values count from the end
            sub_sequence_end + out_start
        }
    };
    if actual_start < 0 {
        return None;
    }
    let actual_start = actual_start as usize; // verified to be >= 0

    if actual_start >= sequence.len() {
        None
    } else {
        // Ensure we don't go beyond sequence bounds
        let end_pos = actual_start + out_length;
        if end_pos > sequence.len() {
            return None;
        }
        let length = end_pos - actual_start;
        Some((
            sequence[actual_start..end_pos].iter().copied().collect(),
            actual_start,
            length,
        ))
    }
}

pub fn read_name_canonical_prefix(name: &[u8], readname_end_char: Option<u8>) -> &[u8] {
    if let Some(separator) = readname_end_char {
        if let Some(position) = memchr::memchr(separator, name) {
            &name[..position]
        } else {
            name
        }
    } else {
        name
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceNoAll {
    Segment(SegmentIndex),
    Tag(String),
    Name {
        segment_index: SegmentIndex,
        split_character: u8,
    },
}

impl ResolvedSourceNoAll {
    pub fn parse(
        source: &str,
        input_def: &config::Input,
    ) -> Result<ResolvedSourceNoAll, anyhow::Error> {
        let source = source.trim();
        let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
            let trimmed = tag_name.trim();
            if trimmed.is_empty() {
                bail!("Source tag:<name> may not have an empty name.");
            }
            ResolvedSourceNoAll::Tag(trimmed.to_string())
        } else if let Some(segment_name) = source.strip_prefix("name:") {
            let trimmed = segment_name.trim();
            if trimmed.is_empty() {
                bail!("TagDuplicates name source requires a segment name");
            }
            let segment = Segment(trimmed.to_string());
            let segment_index = segment.validate(input_def)?;
            ResolvedSourceNoAll::Name {
                segment_index,
                split_character: input_def.options.read_comment_character,
            }
        } else {
            let segment = Segment(source.to_string());
            ResolvedSourceNoAll::Segment(segment.validate(input_def)?)
        };
        Ok(resolved)
    }

    //that's the ones we're going to use
    pub fn get_tags(&self) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        match &self {
            ResolvedSourceNoAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[
                    crate::transformations::TagValueType::String,
                    crate::transformations::TagValueType::Location,
                ],
            )]),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceAll {
    Segment(SegmentIndexOrAll),
    Tag(String),
    Name {
        segment_index_or_all: SegmentIndexOrAll,
        split_character: u8,
    },
}

impl ResolvedSourceAll {
    pub fn parse(
        source: &str,
        input_def: &config::Input,
    ) -> Result<ResolvedSourceAll, anyhow::Error> {
        let source = source.trim();
        let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
            let trimmed = tag_name.trim();
            if trimmed.is_empty() {
                bail!("Source/target tag:<name> may not have an empty name.");
            }
            ResolvedSourceAll::Tag(trimmed.to_string())
        } else if let Some(segment_name) = source.strip_prefix("name:") {
            let trimmed = segment_name.trim();
            if trimmed.is_empty() {
                bail!("Source/target name:<segment> requires a segment name");
            }
            let mut segment = SegmentOrAll(trimmed.to_string());
            let segment_index_or_all = segment.validate(input_def)?;
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character: input_def.options.read_comment_character,
            }
        } else {
            let mut segment = SegmentOrAll(source.to_string());
            ResolvedSourceAll::Segment(segment.validate(input_def)?)
        };
        Ok(resolved)
    }

    //that's the ones we're going to use
    pub fn get_tags(&self) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        match &self {
            ResolvedSourceAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[
                    crate::transformations::TagValueType::String,
                    crate::transformations::TagValueType::Location,
                ],
            )]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_name_canonical_prefix;

    #[test]
    fn canonical_prefix_stops_at_first_separator() {
        assert_eq!(
            read_name_canonical_prefix(b"Sample_1_2", Some(b'_')),
            b"Sample"
        );
    }

    #[test]
    fn canonical_prefix_uses_full_name_when_separator_missing() {
        assert_eq!(read_name_canonical_prefix(b"Sample", None), b"Sample");
    }

    #[test]
    fn custom_separator_is_respected() {
        assert_eq!(read_name_canonical_prefix(b"Run/42", Some(b'/')), b"Run");
    }

    #[test]
    fn missing_separator_configuration_defaults_to_exact_match() {
        assert_eq!(read_name_canonical_prefix(b"Exact", None), b"Exact");
    }
}
