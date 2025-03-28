use enum_dispatch::enum_dispatch;

use std::{
    path::Path,
    thread,
};

use anyhow::{bail, Result};
use serde_valid::Validate;

use crate::{
    config::{RegionDefinition, Target, TargetPlusAll},
    demultiplex::{DemultiplexInfo, Demultiplexed},
    io,
};
use rand::Rng;
use rand::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

mod demultiplex;
mod edits;
mod filters;
mod reports;
mod validation;

/// turn a u64 seed into a 32 byte seed for chacha
fn extend_seed(seed: u64) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();

    // Extend the seed_bytes to 32 bytes
    let mut extended_seed = [0u8; 32];
    extended_seed[..8].copy_from_slice(&seed_bytes);
    extended_seed
}

fn reproducible_cuckoofilter(
    seed: u64,
    initial_capacity: usize,
    false_positive_probability: f64,
) -> ScalableCuckooFilter<[u8], scalable_cuckoo_filter::DefaultHasher, rand_chacha::ChaChaRng> {
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
fn default_name_separator() -> Vec<u8> {
    vec![b'_']
}

#[derive(Debug)]
pub struct FinalizeReportResult {
    pub report_no: usize,
    pub contents: serde_json::Value,
}

#[derive(Debug, Clone)]
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
        _output_def: &Option<crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        Ok(())
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
    /// whether we spawn a new stage when we encounter this particular transformation
    /// (a performance thing, so it's it's own thread (start).)
    fn new_stage(&self) -> bool {
        false
    }

    /// whether the transformation must see all the reads,
    /// even if we had a Head( and would abort otherwise.
    fn must_run_to_completion(&self) -> bool {
        false
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
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
        let delay = rng.gen_range(0..10);
        thread::sleep(std::time::Duration::from_millis(delay));
        (block, true)
    }
}

type OurCuckCooFilter = scalable_cuckoo_filter::ScalableCuckooFilter<
    [u8],
    scalable_cuckoo_filter::DefaultHasher,
    rand_chacha::ChaChaRng,
>;

#[derive(serde::Deserialize, Debug, Validate, Clone)]
pub enum KeepOrRemove {
    Keep,
    Remove,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "action")]
#[enum_dispatch]
pub enum Transformation {
    CutStart(edits::CutStart),
    CutEnd(edits::CutEnd),
    MaxLen(edits::MaxLen),
    Prefix(edits::Prefix),
    Postfix(edits::Postfix),
    ConvertPhred64To33(edits::Phred64To33),
    ReverseComplement(edits::ReverseComplement),
    Rename(edits::Rename),
    TrimAdapterMismatchTail(edits::TrimAdapterMismatchTail),
    TrimPolyTail(edits::TrimPolyTail),
    TrimQualityStart(edits::TrimQualityStart),
    TrimQualityEnd(edits::TrimQualityEnd),
    SwapR1AndR2(edits::SwapR1AndR2),
    ExtractToName(edits::ExtractToName),

    Head(filters::Head),
    Skip(filters::Skip),
    FilterEmpty(filters::Empty),
    FilterMinLen(filters::MinLen),
    FilterMaxLen(filters::MaxLen),
    FilterMeanQuality(filters::MeanQuality),
    FilterQualifiedBases(filters::QualifiedBases),
    FilterTooManyN(filters::TooManyN),
    FilterSample(filters::Sample),
    FilterDuplicates(filters::Duplicates),
    FilterLowComplexity(filters::LowComplexity),
    FilterOtherFile(filters::OtherFile),
    ValidateSeq(validation::ValidateSeq),
    ValidatePhred(validation::ValidatePhred),

    Progress(reports::Progress),
    Report(reports::Report),
    #[serde(skip)]
    _ReportCount(Box<reports::_ReportCount>),
    #[serde(skip)]
    _ReportLengthDistribution(Box<reports::_ReportLengthDistribution>),
    #[serde(skip)]
    _ReportDuplicateCount(Box<reports::_ReportDuplicateCount>),
    #[serde(skip)]
    _ReportBaseStatisticsPart1(Box<reports::_ReportBaseStatisticsPart1>),
    #[serde(skip)]
    _ReportBaseStatisticsPart2(Box<reports::_ReportBaseStatisticsPart2>),

    Inspect(reports::Inspect),
    QuantifyRegions(reports::QuantifyRegions),

    Demultiplex(demultiplex::Demultiplex),

    _InternalDelay(Box<_InternalDelay>),
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
        for transformation in transforms.into_iter() {
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
                                data_per_read: Default::default(),
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

                    report_no += 1;
                    /* //split report into two parts so we can multicore it.
                    let coordinator: Arc<
                        Mutex<OnceCell<Vec<reports::ReportData<reports::ReportCollector1>>>>,
                    > = Arc::new(Mutex::new(OnceCell::new()));
                    let part1 = reports::_ReportPart1 {
                        data: Vec::new(),
                        to_part2: coordinator.clone(),
                    };
                    let part2 = reports::_ReportPart2 {
                        data: Vec::new(),
                        config,
                        from_part1: coordinator,
                    };
                    res.push(Transformation::_ReportPart1(Box::new(part1)));
                    res.push(Transformation::_ReportPart2(Box::new(part2))) */
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
    separator: &[u8],
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut first = true;
    for region in regions {
        let read = match region.source {
            Target::Read1 => &block.read1,
            Target::Read2 => block.read2.as_ref().unwrap(),
            Target::Index1 => block.index1.as_ref().unwrap(),
            Target::Index2 => block.index2.as_ref().unwrap(),
        }
        .get(read_no);
        if first {
            first = false;
        } else {
            out.extend(separator.iter());
        }
        out.extend(
            read.seq()
                .iter()
                .skip(region.start)
                .take(region.length)
                .copied(),
        );
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
    f: impl Fn(&mut io::WrappedFastQReadMut),
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
}

fn apply_filter_all(
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
}
