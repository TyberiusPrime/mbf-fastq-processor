use enum_dispatch::enum_dispatch;

use once_cell::sync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    io::BufWriter,
    path::Path,
    sync::{Arc, Mutex},
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

#[enum_dispatch(Transformation)]
pub trait Step {
    fn validate(&self, input_def: &crate::config::Input,
        output_def: &Option<crate::config::Output>,
        all_transforms: &[Transformation],
    ) -> Result<()> {
        Ok(())
    }
    fn init(&mut self, demultiplex_info: &Demultiplexed) -> Result<Option<DemultiplexInfo>> {
        Ok(None)
    }
    fn finalize(&mut self, 
        output_prefix: &str,
        output_directory: &Path,
        demultiplex_info: &Demultiplexed)  -> Result<()>{Ok(())}
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        (block, true)
    }
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
        // ie. must see all the reads.
        /* matches!(
            self,
            Transformation::Report(_)
                | Transformation::Inspect(_)
                | Transformation::Progress(_)
                | Transformation::QuantifyRegions(_)
        ) */
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
        mut block: crate::io::FastQBlocksCombined,
        block_no: usize,
        demultiplex_info: &Demultiplexed,
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
    PreFix(edits::Prefix),
    PostFix(edits::Postfix),
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
    _ReportPart1(Box<reports::_ReportPart1>),
    #[serde(skip)]
    _ReportPart2(Box<reports::_ReportPart2>),
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
        pub fn check_config(
        &self,
        input_def: &crate::config::Input,
        output_def: &Option<crate::config::Output>,
        all_transforms: &[Transformation],
    ) -> Result<()> {
        for trafo in all_transforms {
            trafo.validate(input_def, output_def, all_transforms)?;
        }
        Ok(())
    }

    /// convert the input transformations into those we actually process
    /// (they are mostly the same, but for example reports get split in two
    /// to take advantage of multicore)
    pub fn expand(transforms: Vec<Self>) -> Vec<Self> {
        /* let mut res = Vec::new();
        for transformation in transforms.into_iter() {
            match transformation {
                Transformation::Report(config) => {
                    //split report into two parts so we can multicore it.
                    let coordinator: Arc<
                        Mutex<OnceCell<Vec<reports::ReportData<reports::ReportPart1>>>>,
                    > = Arc::new(Mutex::new(OnceCell::new()));
                    let part1 = reports::ConfigTransformReportPart1 {
                        data: Vec::new(),
                        to_part2: coordinator.clone(),
                    };
                    let part2 = reports::ConfigTransformReportPart2 {
                        data: Vec::new(),
                        config,
                        from_part1: coordinator,
                    };
                    res.push(Transformation::_ReportPart1(Box::new(part1)));
                    res.push(Transformation::_ReportPart2(Box::new(part2)))
                }
                Transformation::FilterEmpty(c) => {
                    res.push(Transformation::FilterMinLen(ConfigTransformNAndTarget {
                        n: 1,
                        target: c.target,
                    }))
                }
                _ => res.push(transformation),
            }
        }
        res */
        transforms
    }

    // todo: break this into separate functions
    #[allow(clippy::too_many_lines)]
    pub fn transform(
        &mut self,
        block: io::FastQBlocksCombined,
        block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> (io::FastQBlocksCombined, bool) {
        self.apply(block, block_no, demultiplex_info)
    }

    pub fn initialize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        self.init(demultiplex_info);
        /* match self {
            Transformation::Progress(config) => {
                if let Some(output_infix) = &config.output_infix {
                    config.filename = Some(
                        output_directory.join(format!("{output_prefix}_{output_infix}.progress")),
                    );
                    //create empty file so we are sure we can write there
                    let _ = std::fs::File::create(config.filename.as_ref().unwrap())?;
                }
            }
            Transformation::Demultiplex(config) => {
                return Ok(Some(config.init()?));
            }

            Transformation::Report(_) => {
                unreachable!()
            }
            Transformation::_ReportPart1(config) => {
                reports::init_report_part1(config, demultiplex_info)
            }
            Transformation::_ReportPart2(config) => {
                reports::init_report_part2(config, demultiplex_info)
            }
            Transformation::FilterDuplicates(config) => filters::init_filter_duplicates(config),
            Transformation::FilterOtherFile(config) => filters::init_filter_other_file(config)?,
            _ => {}
        } */
        Ok(None)
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    pub fn finalize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        demultiplex_info: &Demultiplexed,
    ) -> Result<()> {
        //happens on the same thread as the processing.
        Ok(())

        /* match self {
            Transformation::Report(_) => {
                unreachable!()
            }
            Transformation::_ReportPart1(config) => {
                reports::finalize_report_part1(config, demultiplex_info);
                Ok(())
            }
            Transformation::_ReportPart2(config) => Ok(reports::finalize_report_part2(
                config,
                output_prefix,
                output_directory,
                demultiplex_info,
            )?),

            Transformation::Inspect(config) => Ok(reports::finalize_inspect(
                config,
                output_prefix,
                output_directory,
                demultiplex_info,
            )?),

            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            Transformation::Progress(config) => {
                let elapsed = config.start_time.unwrap().elapsed().as_secs_f64();
                let count: usize = *config.total_count.lock().unwrap();
                let msg = format!("Took {:.2} s ({}) to process {} molecules for an effective rate of {:.2} molecules/s",
                    elapsed,
                    crate::format_seconds_to_hhmmss(elapsed as u64),
                    count,
                    count as f64 / elapsed

                );
                config.output(&msg);

                Ok(())
            }
            Transformation::QuantifyRegions(config) => output_quantification(
                output_directory,
                output_prefix,
                &config.infix,
                &config.collector,
            ),
            _ => Ok(()),
        } */
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

fn output_quantification(
    output_directory: &Path,
    output_prefix: &str,
    infix: &str,
    collector: &HashMap<Vec<u8>, usize>,
) -> Result<()> {
    use std::io::Write;
    let report_file =
        std::fs::File::create(output_directory.join(format!("{output_prefix}_{infix}.qr.json")))?;
    let mut bufwriter = BufWriter::new(report_file);
    let str_collector: HashMap<String, usize> = collector
        .iter()
        .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
        .collect();
    let json = serde_json::to_string_pretty(&str_collector)?;
    bufwriter.write_all(json.as_bytes())?;
    Ok(())
}

/// for the cases where the actual data is irrelevant.
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

/// for the cases where the actual data is relevant.
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
