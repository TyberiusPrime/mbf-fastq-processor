use std::{
    collections::HashMap,
    io::BufWriter,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{bail, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_valid::Validate;

use crate::{
    io::{self, WrappedFastQReadMut},
    FastQRead,
};
use rand::Rng;
use rand::SeedableRng;
use scalable_cuckoo_filter::ScalableCuckooFilter;

const PHRED33OFFSET: u8 = 33;

// phred score (33 sanger encoding) to probability of error
// python: ([1.0] * 32 + [10**(q/-10) for q in range(0,256)])[:256]
const Q_LOOKUP: [f64; 256] = [
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    0.7943282347242815,
    0.6309573444801932,
    0.5011872336272722,
    0.3981071705534972,
    0.31622776601683794,
    0.251188643150958,
    0.19952623149688797,
    0.15848931924611134,
    0.12589254117941673,
    0.1,
    0.07943282347242814,
    0.06309573444801933,
    0.05011872336272722,
    0.039810717055349734,
    0.03162277660168379,
    0.025118864315095794,
    0.0199526231496888,
    0.015848931924611134,
    0.012589254117941675,
    0.01,
    0.007943282347242814,
    0.00630957344480193,
    0.005011872336272725,
    0.003981071705534973,
    0.0031622776601683794,
    0.0025118864315095794,
    0.001995262314968879,
    0.001584893192461114,
    0.0012589254117941675,
    0.001,
    0.0007943282347242813,
    0.000630957344480193,
    0.0005011872336272725,
    0.00039810717055349735,
    0.00031622776601683794,
    0.00025118864315095795,
    0.00019952623149688788,
    0.00015848931924611142,
    0.00012589254117941674,
    0.0001,
    7.943282347242822e-05,
    6.309573444801929e-05,
    5.011872336272725e-05,
    3.9810717055349695e-05,
    3.1622776601683795e-05,
    2.5118864315095822e-05,
    1.9952623149688786e-05,
    1.584893192461114e-05,
    1.2589254117941661e-05,
    1e-05,
    7.943282347242822e-06,
    6.30957344480193e-06,
    5.011872336272725e-06,
    3.981071705534969e-06,
    3.162277660168379e-06,
    2.5118864315095823e-06,
    1.9952623149688787e-06,
    1.584893192461114e-06,
    1.2589254117941661e-06,
    1e-06,
    7.943282347242822e-07,
    6.30957344480193e-07,
    5.011872336272725e-07,
    3.981071705534969e-07,
    3.162277660168379e-07,
    2.5118864315095823e-07,
    1.9952623149688787e-07,
    1.584893192461114e-07,
    1.2589254117941662e-07,
    1e-07,
    7.943282347242822e-08,
    6.30957344480193e-08,
    5.011872336272725e-08,
    3.981071705534969e-08,
    3.162277660168379e-08,
    2.511886431509582e-08,
    1.9952623149688786e-08,
    1.5848931924611143e-08,
    1.2589254117941661e-08,
    1e-08,
    7.943282347242822e-09,
    6.309573444801943e-09,
    5.011872336272715e-09,
    3.981071705534969e-09,
    3.1622776601683795e-09,
    2.511886431509582e-09,
    1.9952623149688828e-09,
    1.584893192461111e-09,
    1.2589254117941663e-09,
    1e-09,
    7.943282347242822e-10,
    6.309573444801942e-10,
    5.011872336272714e-10,
    3.9810717055349694e-10,
    3.1622776601683795e-10,
    2.511886431509582e-10,
    1.9952623149688828e-10,
    1.584893192461111e-10,
    1.2589254117941662e-10,
    1e-10,
    7.943282347242822e-11,
    6.309573444801942e-11,
    5.011872336272715e-11,
    3.9810717055349695e-11,
    3.1622776601683794e-11,
    2.5118864315095823e-11,
    1.9952623149688828e-11,
    1.5848931924611107e-11,
    1.2589254117941662e-11,
    1e-11,
    7.943282347242821e-12,
    6.309573444801943e-12,
    5.011872336272715e-12,
    3.9810717055349695e-12,
    3.1622776601683794e-12,
    2.5118864315095823e-12,
    1.9952623149688827e-12,
    1.584893192461111e-12,
    1.258925411794166e-12,
    1e-12,
    7.943282347242822e-13,
    6.309573444801942e-13,
    5.011872336272715e-13,
    3.981071705534969e-13,
    3.162277660168379e-13,
    2.511886431509582e-13,
    1.9952623149688827e-13,
    1.584893192461111e-13,
    1.2589254117941663e-13,
    1e-13,
    7.943282347242822e-14,
    6.309573444801943e-14,
    5.0118723362727144e-14,
    3.9810717055349693e-14,
    3.1622776601683796e-14,
    2.5118864315095823e-14,
    1.9952623149688828e-14,
    1.584893192461111e-14,
    1.2589254117941662e-14,
    1e-14,
    7.943282347242822e-15,
    6.309573444801943e-15,
    5.0118723362727146e-15,
    3.9810717055349695e-15,
    3.1622776601683794e-15,
    2.511886431509582e-15,
    1.995262314968883e-15,
    1.584893192461111e-15,
    1.2589254117941663e-15,
    1e-15,
    7.943282347242821e-16,
    6.309573444801943e-16,
    5.011872336272715e-16,
    3.9810717055349695e-16,
    3.1622776601683793e-16,
    2.511886431509582e-16,
    1.995262314968883e-16,
    1.5848931924611109e-16,
    1.2589254117941662e-16,
    1e-16,
    7.943282347242789e-17,
    6.309573444801943e-17,
    5.0118723362727144e-17,
    3.9810717055349855e-17,
    3.1622776601683796e-17,
    2.5118864315095718e-17,
    1.9952623149688827e-17,
    1.584893192461111e-17,
    1.2589254117941713e-17,
    1e-17,
    7.94328234724279e-18,
    6.309573444801943e-18,
    5.011872336272715e-18,
    3.981071705534985e-18,
    3.1622776601683795e-18,
    2.5118864315095718e-18,
    1.995262314968883e-18,
    1.5848931924611109e-18,
    1.2589254117941713e-18,
    1e-18,
    7.943282347242789e-19,
    6.309573444801943e-19,
    5.011872336272715e-19,
    3.9810717055349853e-19,
    3.162277660168379e-19,
    2.5118864315095717e-19,
    1.995262314968883e-19,
    1.584893192461111e-19,
    1.2589254117941713e-19,
    1e-19,
    7.94328234724279e-20,
    6.309573444801943e-20,
    5.011872336272715e-20,
    3.9810717055349855e-20,
    3.162277660168379e-20,
    2.511886431509572e-20,
    1.9952623149688828e-20,
    1.5848931924611108e-20,
    1.2589254117941713e-20,
    1e-20,
    7.943282347242789e-21,
    6.309573444801943e-21,
    5.011872336272714e-21,
    3.981071705534986e-21,
    3.1622776601683792e-21,
    2.511886431509572e-21,
    1.9952623149688827e-21,
    1.5848931924611108e-21,
    1.2589254117941713e-21,
    1e-21,
    7.943282347242789e-22,
    6.309573444801943e-22,
    5.011872336272715e-22,
    3.9810717055349856e-22,
    3.1622776601683793e-22,
    2.511886431509572e-22,
    1.9952623149688828e-22,
    1.584893192461111e-22,
    1.2589254117941713e-22,
    1e-22,
    7.943282347242789e-23,
    6.309573444801943e-23,
    5.011872336272715e-23,
];

fn extend_seed(seed: u64) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();

    // Extend the seed_bytes to 32 bytes
    let mut extended_seed = [0u8; 32];
    extended_seed[..8].copy_from_slice(&seed_bytes);
    extended_seed
}

fn u8_from_string<'de, D>(deserializer: D) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.as_bytes().to_vec())
}

fn dna_from_string<'de, D>(deserializer: D) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    //check whether it's DNA bases...
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N') {
            return Err(serde::de::Error::custom(format!("Invalid DNA base: {}", c)));
        }
    }
    Ok(s.as_bytes().to_vec())
}

fn base_or_dot<'de, D>(deserializer: D) -> core::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    if s.len() != 1 {
        return Err(serde::de::Error::custom(format!(
            "Single DNA base or '.' only): was '{}'",
            s
        )));
    }
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N' | '.') {
            return Err(serde::de::Error::custom(format!(
                "Invalid DNA base (. for any also allowed): {}",
                c
            )));
        }
    }
    let out = s.as_bytes()[0];
    Ok(out)
}

pub fn u8_from_char_or_number<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = u8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("either a character or a number")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v as u8)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v.len() {
                0 => Err(E::custom("empty string")),
                1 => Ok(v.bytes().next().unwrap() as u8),
                _ => Err(E::custom("string should be exactly one character long")),
            }
        }
    }

    deserializer.deserialize_any(Visitor)
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

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub enum Target {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub enum TargetPlusAll {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
    #[serde(alias = "all")]
    All,
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

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformN {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformNAndTarget {
    pub n: usize,
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformTarget {
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformValidate {
    #[serde(deserialize_with = "u8_from_string")]
    pub allowed: Vec<u8>,
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformText {
    pub target: Target,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: Vec<u8>,

    #[serde(deserialize_with = "u8_from_string")] //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: Vec<u8>,
}

fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
}

fn default_name_seperator() -> Vec<u8> {
    vec![b'_']
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformToName {
    pub source: Target,
    pub start: usize,
    pub length: usize,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_readname_end_chars"
    )]
    pub readname_end_chars: Vec<u8>,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_seperator"
    )]
    pub separator: Vec<u8>,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformPolyTail {
    pub target: Target,
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 10.)]
    pub max_mismatch_rate: f32,
    pub max_consecutive_mismatches: usize,
}
#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformAdapterMismatchTail {
    pub target: Target,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[serde(deserialize_with = "dna_from_string")]
    pub query: Vec<u8>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformProgress {
    #[serde(skip)]
    pub total_count: Arc<Mutex<usize>>,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    pub n: usize,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformQual {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformQualFloat {
    pub target: Target,
    pub min: f32,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformQualifiedBases {
    #[serde(deserialize_with = "u8_from_char_or_number")]
    min_quality: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    min_percentage: f32,
    target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformFilterTooManyN {
    target: Target,
    n: usize,
}
#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformFilterLowComplexity {
    target: Target,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    threshold: f32,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformSample {
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    p: f32,
    seed: u64,
}
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformInternalDelay {
    #[serde(skip)]
    rng: Option<rand_chacha::ChaChaRng>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformInspect {
    n: usize,
    target: Target,
    infix: String,
    #[serde(skip)]
    collector: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformQuantifyRegion {
    target: Target,
    infix: String,
    start: usize,

    #[validate(minimum = 1)]
    length: usize,
    #[serde(skip)]
    collector: HashMap<Vec<u8>, usize>,
}

#[derive(serde::Serialize, Debug, Clone, Default)]
struct PositionCounts {
    a: Vec<usize>,
    g: Vec<usize>,
    c: Vec<usize>,
    t: Vec<usize>,
    n: Vec<usize>,
}

#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct ReportPart {
    total_bases: usize,
    q20_bases: usize,
    q30_bases: usize,
    gc_bases: usize,
    per_position_counts: PositionCounts,
    length_distribution: Vec<usize>,
    expected_errors_from_quality_curve: Vec<f64>,
    duplicate_count: usize,
    #[serde(skip)]
    duplication_filter: Option<OurCuckCooFilter>,
    //#[serde(skip)]
    //    kmers: HashMap<Kmer, usize>
}

unsafe impl Send for ReportPart {} //fine as long as duplication_filter is None

#[derive(serde::Serialize, Debug, Clone)]
pub struct ReportData {
    program_version: String,
    read_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    read1: Option<ReportPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read2: Option<ReportPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index1: Option<ReportPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index2: Option<ReportPart>,
}

impl Default for ReportData {
    fn default() -> Self {
        ReportData {
            program_version: env!("CARGO_PKG_VERSION").to_string(),
            read_count: 0,
            read1: None,
            read2: None,
            index1: None,
            index2: None,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, Clone)]
pub struct ConfigTransformReport {
    infix: String,
    json: bool,
    html: bool,
    #[serde(skip)]
    data: ReportData,
    #[serde(default)]
    debug_reproducibility: bool,
}

type OurCuckCooFilter = scalable_cuckoo_filter::ScalableCuckooFilter<
    [u8],
    scalable_cuckoo_filter::DefaultHasher,
    rand_chacha::ChaChaRng,
>;

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformFilterDuplicates {
    target: TargetPlusAll,
    #[serde(default)]
    invert: bool,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    false_positive_rate: f64,
    seed: u64,
    #[serde(skip)]
    filter: Option<OurCuckCooFilter>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum Transformation {
    Head(ConfigTransformN),
    Skip(ConfigTransformN),

    CutStart(ConfigTransformNAndTarget),
    CutEnd(ConfigTransformNAndTarget),
    MaxLen(ConfigTransformNAndTarget),

    PreFix(ConfigTransformText),
    PostFix(ConfigTransformText),

    Reverse(ConfigTransformTarget),
    SwapR1AndR2,
    ConvertPhred64To33,
    ValidateSeq(ConfigTransformValidate),
    ValidatePhred(ConfigTransformTarget),
    ExtractToName(ConfigTransformToName),

    TrimAdapterMismatchTail(ConfigTransformAdapterMismatchTail),
    TrimPolyTail(ConfigTransformPolyTail),
    TrimQualityStart(ConfigTransformQual),
    TrimQualityEnd(ConfigTransformQual),

    FilterEmpty(ConfigTransformTarget),
    FilterMinLen(ConfigTransformNAndTarget),
    FilterMaxLen(ConfigTransformNAndTarget),
    FilterMeanQuality(ConfigTransformQualFloat),
    FilterQualifiedBases(ConfigTransformQualifiedBases),
    FilterTooManyN(ConfigTransformFilterTooManyN),
    FilterSample(ConfigTransformSample),
    FilterDuplicates(ConfigTransformFilterDuplicates),
    FilterLowComplexity(ConfigTransformFilterLowComplexity),

    Progress(ConfigTransformProgress),
    Report(ConfigTransformReport),
    Inspect(ConfigTransformInspect),
    QuantifyRegion(ConfigTransformQuantifyRegion),

    InternalDelay(ConfigTransformInternalDelay),
}

fn verify_target(target: Target, input_def: &crate::config::ConfigInput) -> Result<()> {
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

impl Transformation {
    pub fn needs_serial(&self) -> bool {
        // ie. must see all the reads.
        match self {
            Transformation::Report(_) | //todo: I guess I could make it multithreaded
            Transformation::Inspect(_) | //todo: I guess I could make it multithreaded
            Transformation::QuantifyRegion(_) | //todo: I guess I could make it multithreaded
            Transformation::Head(_) |
            Transformation::Skip(_) => true,
            _ => false,
        }
    }

    pub fn check_config(&self, input_def: &crate::config::ConfigInput) -> Result<()> {
        return match self {
            Transformation::CutStart(c) | Transformation::CutEnd(c) | Transformation::MaxLen(c) => {
                verify_target(c.target, input_def)
            }
            Transformation::PreFix(c) | Transformation::PostFix(c) => {
                verify_target(c.target, input_def)?;
                if c.seq.len() != c.qual.len() {
                    bail!("Seq and qual must be the same length");
                }
                Ok(())
            }
            Transformation::Reverse(c) => verify_target(c.target, input_def),
            Transformation::Inspect(c) => verify_target(c.target, input_def),
            Transformation::QuantifyRegion(c) => verify_target(c.target, input_def),
            Transformation::ExtractToName(c) => {
                verify_target(c.source, input_def)?;
                if c.length == 0 {
                    bail!("Length must be > 0");
                }
                Ok(())
            }
            Transformation::TrimAdapterMismatchTail(c) => {
                verify_target(c.target, input_def)?;
                if c.max_mismatches > c.min_length {
                    bail!("Max mismatches must be <= min length");
                }
                Ok(())
            }
            Transformation::TrimPolyTail(c) => verify_target(c.target, input_def),
            Transformation::TrimQualityStart(c) => verify_target(c.target, input_def),
            Transformation::TrimQualityEnd(c) => verify_target(c.target, input_def),
            Transformation::FilterEmpty(c) => verify_target(c.target, input_def),
            Transformation::FilterMinLen(c) => verify_target(c.target, input_def),
            Transformation::FilterMaxLen(c) => verify_target(c.target, input_def),
            Transformation::FilterMeanQuality(c) => verify_target(c.target, input_def),
            Transformation::FilterQualifiedBases(c) => verify_target(c.target, input_def),
            Transformation::FilterTooManyN(c) => verify_target(c.target, input_def),
            Transformation::SwapR1AndR2 => {
                if input_def.read2.is_none() {
                    bail!("Read2 is not defined in the input section, but used by transformation SwapR1AndR2");
                }
                Ok(())
            }
            _ => Ok(()),
        };
    }

    pub fn transform(
        &mut self,
        mut block: io::FastQBlocksCombined,
        block_no: usize,
    ) -> (io::FastQBlocksCombined, bool) {
        match self {
            Transformation::Head(config) => {
                let remaining = config.n - config.so_far;
                if remaining == 0 {
                    return (block.empty(), false);
                } else {
                    block.resize(remaining.min(block.len()));
                    let do_continue = remaining > block.len();
                    config.so_far += block.len();
                    (block, do_continue)
                }
            }

            Transformation::Skip(config) => {
                let remaining = config.n - config.so_far;
                if remaining == 0 {
                    (block, true)
                } else {
                    if remaining >= block.len() {
                        config.so_far += block.len();
                        (block.empty(), true)
                    } else {
                        let here = remaining.min(block.len());
                        config.so_far += here;
                        block.block_read1.entries.drain(0..here);
                        if let Some(ref mut read2) = block.block_read2 {
                            read2.entries.drain(0..here);
                            assert_eq!(read2.len(), block.block_read1.len());
                        }
                        if let Some(ref mut index1) = block.block_index1 {
                            index1.entries.drain(0..here);
                        }
                        if let Some(ref mut index2) = block.block_index2 {
                            index2.entries.drain(0..here);
                        }
                        (block, true)
                    }
                }
            }

            Transformation::CutStart(config) => {
                apply_in_place(config.target, |read| read.cut_start(config.n), &mut block);
                (block, true)
            }

            Transformation::CutEnd(config) => {
                apply_in_place(config.target, |read| read.cut_end(config.n), &mut block);
                (block, true)
            }

            Transformation::MaxLen(config) => {
                apply_in_place(config.target, |read| read.max_len(config.n), &mut block);
                (block, true)
            }

            Transformation::PreFix(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.prefix(&config.seq, &config.qual),
                    &mut block,
                );
                (block, true)
            }

            Transformation::PostFix(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.postfix(&config.seq, &config.qual),
                    &mut block,
                );
                (block, true)
            }

            Transformation::Reverse(config) => {
                apply_in_place_wrapped(config.target, |read| read.reverse(), &mut block);
                (block, true)
            }

            Transformation::ConvertPhred64To33 => {
                block.apply_mut(|read1, read2, index1, index2| {
                    let qual = read1.qual();
                    let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                    if new_qual.iter().any(|x| *x < 33) {
                        panic!("Phred 64-33 conversion yielded values below 33 -> wasn't Phred 64 to begin with");
                    }
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
                (block, true)
            }
            Transformation::ValidateSeq(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| {
                        if read.seq().iter().any(|x| !config.allowed.contains(x)) {
                            panic!(
                                "Invalid base found in sequence: {:?} {:?}",
                                std::str::from_utf8(read.name()),
                                std::str::from_utf8(read.seq())
                            );
                        }
                    },
                    &mut block,
                );

                (block, true)
            }
            Transformation::ValidatePhred(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| {
                        if read.qual().iter().any(|x| *x < 33 || *x > 74) {
                            panic!(
                                "Invalid phred quality found. Expected 33..=74 (!..J) : {:?} {:?}",
                                std::str::from_utf8(read.name()),
                                std::str::from_utf8(read.qual())
                            );
                        }
                    },
                    &mut block,
                );

                (block, true)
            }
            Transformation::ExtractToName(config) => {
                block.apply_mut(|read1, read2, index1, index2| {
                    let source = match config.source {
                        Target::Read1 => &read1,
                        Target::Read2 => &read2.as_ref().expect("Input def and target mismatch"),
                        Target::Index1 => &index1.as_ref().expect("Input def and target mismatch"),
                        Target::Index2 => &index2.as_ref().expect("Input def and target mismatch"),
                    };
                    let extracted: Vec<u8> = source
                        .seq()
                        .iter()
                        .skip(config.start)
                        .take(config.length)
                        .cloned()
                        .collect();

                    let name = read1.name();
                    let mut split_pos = None;
                    for letter in config.readname_end_chars.iter() {
                        if let Some(pos) = name.iter().position(|&x| x == *letter) {
                            split_pos = Some(pos);
                            break;
                        }
                    }
                    let new_name = match split_pos {
                        None => {
                            let mut new_name: Vec<u8> = name.into();
                            new_name.extend(config.separator.iter());
                            new_name.extend(extracted.iter());
                            new_name
                        }
                        Some(split_pos) => {
                            let mut new_name = Vec::with_capacity(
                                name.len() + config.separator.len() + extracted.len(),
                            );
                            new_name.extend(name.iter().take(split_pos));
                            new_name.extend(config.separator.iter());
                            new_name.extend(extracted.iter());
                            new_name.extend(name.iter().skip(split_pos));
                            new_name
                        }
                    };
                    read1.replace_name(new_name);
                });
                (block, true)
            }

            Transformation::TrimAdapterMismatchTail(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| {
                        read.trim_adapter_mismatch_tail(
                            &config.query,
                            config.min_length,
                            config.max_mismatches,
                        )
                    },
                    &mut block,
                );
                (block, true)
            }
            Transformation::TrimPolyTail(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| {
                        read.trim_poly_base(
                            config.min_length,
                            config.max_mismatch_rate,
                            config.max_consecutive_mismatches,
                            config.base,
                        )
                    },
                    &mut block,
                );
                (block, true)
            }

            Transformation::TrimQualityStart(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.trim_quality_start(config.min),
                    &mut block,
                );
                (block, true)
            }
            Transformation::TrimQualityEnd(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.trim_quality_end(config.min),
                    &mut block,
                );
                (block, true)
            }
            Transformation::FilterEmpty(config) => {
                apply_filter(config.target, &mut block, |read| read.seq().len() > 0);
                (block, true)
            }
            Transformation::FilterMinLen(config) => {
                apply_filter(config.target, &mut block, |read| {
                    read.seq().len() >= config.n
                });
                (block, true)
            }

            Transformation::FilterMaxLen(config) => {
                apply_filter(config.target, &mut block, |read| {
                    read.seq().len() <= config.n
                });
                (block, true)
            }

            Transformation::FilterMeanQuality(config) => {
                apply_filter(config.target, &mut block, |read| {
                    let qual = read.qual();
                    let sum: usize = qual.iter().map(|x| *x as usize).sum();
                    let avg_qual = sum as f32 / qual.len() as f32;
                    avg_qual >= config.min
                });
                (block, true)
            }

            Transformation::FilterQualifiedBases(config) => {
                apply_filter(config.target, &mut block, |read| {
                    let qual = read.qual();
                    let sum: usize = qual
                        .iter()
                        .map(|x| (*x >= config.min_quality) as usize)
                        .sum();
                    let pct = sum as f32 / qual.len() as f32;
                    pct >= config.min_percentage
                });
                (block, true)
            }
            Transformation::FilterTooManyN(config) => {
                apply_filter(config.target, &mut block, |read| {
                    let seq = read.seq();
                    let sum: usize = seq.iter().map(|x| (*x == b'N') as usize).sum();
                    sum <= config.n
                });
                (block, true)
            }
            Transformation::FilterSample(config) => {
                let extended_seed = extend_seed(config.seed);

                //todo: I think we should singlecore this, and have just one rng in total,
                //not reinitalizie it over and over
                let mut rng = rand_chacha::ChaChaRng::from_seed(extended_seed);
                apply_filter(Target::Read1, &mut block, |_| rng.gen_bool(config.p as f64));
                (block, true)
            }

            Transformation::Progress(config) => {
                if let None = config.start_time {
                    config.start_time = Some(std::time::Instant::now());
                }
                let (counter, next) = {
                    let mut counter = config.total_count.lock().unwrap();
                    //    println!("Thread {:?}", thread::current().id());
                    let val = *counter;
                    let next = *counter + block.len();
                    *counter = next;
                    drop(counter);
                    (val, next)
                };
                //now for any multiple of n that's in the range, we print a message
                let offset = counter % config.n;
                for ii in ((counter + offset)..next).step_by(config.n) {
                    let elapsed = config.start_time.unwrap().elapsed().as_secs_f64();
                    let rate_total = ii as f64 / elapsed;
                    if elapsed > 1.0 {
                        println!(
                            "Processed Total: {} ({:.2} molecules/s), Elapsed: {}s",
                            ii,
                            rate_total,
                            config.start_time.unwrap().elapsed().as_secs()
                        );
                    } else {
                        println!(
                            "Processed Total: {}, Elapsed: {}s",
                            ii,
                            config.start_time.unwrap().elapsed().as_secs()
                        );
                    }
                }
                (block, true)
            }

            Transformation::InternalDelay(config) => {
                if let None = config.rng {
                    let seed = block_no; //needs to be reproducible, but different for each block
                    let seed_bytes = seed.to_le_bytes();

                    // Extend the seed_bytes to 32 bytes
                    let mut extended_seed = [0u8; 32];
                    extended_seed[..8].copy_from_slice(&seed_bytes);
                    let rng = rand_chacha::ChaCha20Rng::from_seed(extended_seed);
                    config.rng = Some(rng);
                }
                //todo: I think we should singlecore this, and have just one rng in total,
                //not reinitalizie it over and over

                let rng = config.rng.as_mut().unwrap();
                let delay = rng.gen_range(0..10);
                thread::sleep(std::time::Duration::from_millis(delay));
                (block, true)
            }

            Transformation::Report(config) => {
                fn update_from_read(target: &mut ReportPart, read: &io::WrappedFastQRead) {
                    {
                        //this is terribly slow right now.
                        //I need to multicore and aggregate this.
                        let read_len = read.len();
                        if target.length_distribution.len() <= read_len {
                            target.length_distribution.resize(read_len + 1, 0);
                            target.per_position_counts.a.resize(read_len, 0);
                            target.per_position_counts.g.resize(read_len, 0);
                            target.per_position_counts.c.resize(read_len, 0);
                            target.per_position_counts.t.resize(read_len, 0);
                            target.per_position_counts.n.resize(read_len, 0);
                            target
                                .expected_errors_from_quality_curve
                                .resize(read_len, 0.0);
                        }
                        target.length_distribution[read_len] += 1;

                        //
                        //this takes about 3s on data/large/ERR12828869_1.fq
                        let q20_bases = 0;
                        let q30_bases = 0;
                        /* for (ii, phred_qual) in read.qual().iter().enumerate() {
                        } */

                        for (ii, base) in read.qual().iter().enumerate() {
                            if *base >= 20 + PHRED33OFFSET {
                                target.q20_bases += 1;
                                if *base >= 30 + PHRED33OFFSET {
                                    target.q30_bases += 1;
                                }
                            }
                            // averaging phred with the arithetic mean is a bad idea.
                            // https://www.drive5.com/usearch/manual/avgq.html
                            // I think what we should be reporting is the
                            // this (powf) is very slow, so we use a lookup table
                            // let q = base.saturating_sub(PHRED33OFFSET) as f64;
                            // let e = 10f64.powf(q / -10.0);
                            // % expected value at each position.
                            let e = Q_LOOKUP[*base as usize];
                            target.expected_errors_from_quality_curve[ii] += e;
                        }
                        target.q20_bases += q20_bases;
                        target.q30_bases += q30_bases;

                        //this takes about 12s on data/large/ERR12828869_1.fq
                        let seq = read.seq();
                        for (ii, base) in seq.iter().enumerate() {
                            match base {
                                b'A' => target.per_position_counts.a[ii] += 1,
                                b'C' => target.per_position_counts.c[ii] += 1,
                                b'G' => target.per_position_counts.g[ii] += 1,
                                b'T' => target.per_position_counts.t[ii] += 1,
                                _ => target.per_position_counts.n[ii] += 1,
                            }
                        }

                        //this takes about 1s
                        //
                        //this takes another 11s.
                        if target.duplication_filter.as_ref().unwrap().contains(seq) {
                            target.duplicate_count += 1;
                        } else {
                            target.duplication_filter.as_mut().unwrap().insert(seq);
                        }

                        //todo: AGTCN per position (just sum, floats come later)
                        //qual curve (needs floats & avg? or just sum and divide by read count,
                        //but short reads will mess that up...)
                        //kmer count?
                        //duplication rate (how is that done in fastp)
                        //overrepresented_sequencs (how is that done in fastp)
                        //min, maximum read length?
                    }
                }
                config.data.read_count += block.len();
                let (initial_capacity, false_positive_probability) = if config.debug_reproducibility
                {
                    (100, 0.1)
                } else {
                    (1_000_000, 0.01)
                };
                if config.data.read1.is_none() {
                    config.data.read1 = Some(Default::default());
                    config.data.read1.as_mut().unwrap().duplication_filter = Some(
                        reproducible_cuckoofilter(42, initial_capacity, false_positive_probability),
                    );
                }
                let mut iter = block.block_read1.get_pseudo_iter();
                while let Some(read) = iter.next() {
                    update_from_read(config.data.read1.as_mut().unwrap(), &read);
                }

                if block.block_read2.is_some() && config.data.read2.is_none() {
                    config.data.read2 = Some(Default::default());
                    config.data.read2.as_mut().unwrap().duplication_filter = Some(
                        reproducible_cuckoofilter(42, initial_capacity, false_positive_probability),
                    );
                }
                if let Some(block_read2) = &mut block.block_read2 {
                    let mut iter = block_read2.get_pseudo_iter();
                    while let Some(read) = iter.next() {
                        update_from_read(config.data.read2.as_mut().unwrap(), &read);
                    }
                }
                if block.block_index1.is_some() && config.data.index1.is_none() {
                    config.data.index1 = Some(Default::default());
                    config.data.index1.as_mut().unwrap().duplication_filter = Some(
                        reproducible_cuckoofilter(42, initial_capacity, false_positive_probability),
                    );
                }
                if let Some(block_index1) = &mut block.block_index1 {
                    let mut iter = block_index1.get_pseudo_iter();
                    while let Some(read) = iter.next() {
                        update_from_read(config.data.read2.as_mut().unwrap(), &read);
                    }
                }

                if block.block_index2.is_some() && config.data.index2.is_none() {
                    config.data.index2 = Some(Default::default());
                    config.data.index2.as_mut().unwrap().duplication_filter = Some(
                        reproducible_cuckoofilter(42, initial_capacity, false_positive_probability),
                    );
                }
                if let Some(block_index2) = &mut block.block_index2 {
                    let mut iter = block_index2.get_pseudo_iter();
                    while let Some(read) = iter.next() {
                        update_from_read(config.data.read2.as_mut().unwrap(), &read);
                    }
                }
                (block, true)
            }

            Transformation::Inspect(config) => {
                let collector = &mut config.collector;
                let source = match config.target {
                    Target::Read1 => &block.block_read1,
                    Target::Read2 => block.block_read2.as_ref().unwrap(),
                    Target::Index1 => block.block_index1.as_ref().unwrap(),
                    Target::Index2 => block.block_index2.as_ref().unwrap(),
                };
                while collector.len() < config.n {
                    let mut iter = source.get_pseudo_iter();
                    while let Some(read) = iter.next() {
                        collector.push((
                            read.name().to_vec(),
                            read.seq().to_vec(),
                            read.qual().to_vec(),
                        ))
                    }
                }
                (block, true)
            }
            Transformation::QuantifyRegion(config) => {
                let collector = &mut config.collector;
                let source = match config.target {
                    Target::Read1 => &block.block_read1,
                    Target::Read2 => block.block_read2.as_ref().unwrap(),
                    Target::Index1 => block.block_index1.as_ref().unwrap(),
                    Target::Index2 => block.block_index2.as_ref().unwrap(),
                };
                let mut iter = source.get_pseudo_iter();
                while let Some(read) = iter.next() {
                    let seq = read.seq();
                    let region = seq
                        .iter()
                        .skip(config.start)
                        .take(config.length)
                        .cloned()
                        .collect();
                    *collector.entry(region).or_insert(0) += 1;
                }
                (block, true)
            }

            Transformation::FilterDuplicates(config) => {
                use rand::SeedableRng;
                if let None = config.filter {
                    config.filter = Some(reproducible_cuckoofilter(
                        config.seed,
                        1_000_000,
                        config.false_positive_rate,
                    ));
                }
                let filter = config.filter.as_mut().unwrap();
                if let Ok(target) = config.target.try_into() {
                    apply_filter(target, &mut block, |read| {
                        if filter.contains(read.seq()) {
                            config.invert
                        } else {
                            filter.insert(read.seq());
                            !config.invert
                        }
                    });
                } else {
                    apply_filter_all(&mut block, |read1, read2, index1, index2| {
                        //wish I could feed tehse into the filter without creating the vec
                        let mut seq: Vec<_> = Vec::new();
                        seq.extend(read1.seq().iter());
                        if let Some(read2) = read2 {
                            seq.extend(read2.seq().iter());
                        }
                        if let Some(index1) = index1 {
                            seq.extend(index1.seq().iter());
                        }
                        if let Some(index2) = index2 {
                            seq.extend(index2.seq().iter());
                        }

                        if filter.contains(&seq) {
                            config.invert
                        } else {
                            filter.insert(&seq);
                            !config.invert
                        }
                    });
                }
                (block, true)
            }

            Transformation::FilterLowComplexity(config) => {
                apply_filter(config.target, &mut block, |read| {
                    //how many transitions are there in read.seq()
                    let mut transitions = 0;
                    let seq = read.seq();
                    for ii in 0..seq.len() - 1 {
                        if seq[ii] != seq[ii + 1] {
                            transitions += 1;
                        }
                    }
                    let ratio = transitions as f32 / (read.len() - 1) as f32;
                    ratio >= config.threshold
                });
                (block, true)
            }

            Transformation::SwapR1AndR2 => {
                let read1 = block.block_read1;
                let read2 = block.block_read2.take().unwrap();
                block.block_read1 = read2;
                block.block_read2 = Some(read1);
                (block, true)
            }
        }
    }

    pub fn finalize(&mut self, output_prefix: &str, output_directory: &Path) -> Result<()> {
        //happens on the same thread as the processing.
        fn fill_in(part: &mut ReportPart) {
            let mut reads_with_at_least_this_length = vec![0; part.length_distribution.len()];
            let mut running = 0;
            for (ii, count) in part.length_distribution.iter().enumerate().rev() {
                running += count;
                reads_with_at_least_this_length[ii] = running;
            }
            for ii in 0..part.expected_errors_from_quality_curve.len() {
                part.expected_errors_from_quality_curve[ii] /=
                    reads_with_at_least_this_length[ii] as f64;
            }
            part.duplication_filter.take();
            let c_bases: usize = part.per_position_counts.c.iter().sum();

            let g_bases: usize = part.per_position_counts.g.iter().sum();
            part.gc_bases = g_bases + c_bases;
            part.total_bases = part.per_position_counts.a.iter().sum::<usize>()
                + c_bases
                + g_bases
                + part.per_position_counts.t.iter().sum::<usize>()
                + part.per_position_counts.n.iter().sum::<usize>();
        }
        match self {
            Transformation::Report(config) => {
                let data = if config.json || config.html {
                    for p in [
                        &mut config.data.read1,
                        &mut config.data.read2,
                        &mut config.data.index1,
                        &mut config.data.index2,
                    ] {
                        if let Some(p) = p.as_mut() {
                            fill_in(p);
                        }
                    }
                    &config.data
                } else {
                    return Ok(());
                };
                if config.json {
                    let report_file = std::fs::File::create(
                        output_directory.join(format!("{}_{}.json", output_prefix, config.infix)),
                    )?;
                    let mut bufwriter = BufWriter::new(report_file);
                    serde_json::to_writer_pretty(&mut bufwriter, &data)?;
                }
                if config.html {
                    let report_file = std::fs::File::create(
                        output_directory.join(format!("{}_{}.html", output_prefix, config.infix)),
                    )?;
                    let mut bufwriter = BufWriter::new(report_file);
                    let template = include_str!("../html/template.html");
                    let chartjs = include_str!("../html/chart/chart.umd.min.js");
                    let json = serde_json::to_string_pretty(&data).unwrap();
                    let html = template
                        .replace("%TITLE%", &config.infix)
                        .replace("\"%DATA%\"", &json)
                        .replace("/*%CHART%*/", chartjs);
                    bufwriter.write_all(html.as_bytes())?;
                }
                Ok(())
            }
            Transformation::Inspect(config) => {
                use std::io::Write;
                let report_file = std::fs::File::create(
                    output_directory.join(format!("{}_{}.fq", output_prefix, config.infix)),
                )?;
                let mut bufwriter = BufWriter::new(report_file);
                for (name, seq, qual) in config.collector.iter() {
                    bufwriter.write_all(b"@")?;
                    bufwriter.write_all(name)?;
                    bufwriter.write_all(b"\n")?;
                    bufwriter.write_all(seq)?;
                    bufwriter.write_all(b"\n+\n")?;
                    bufwriter.write_all(qual)?;
                    bufwriter.write_all(b"\n")?;
                }
                Ok(())
            }
            Transformation::QuantifyRegion(config) => {
                use std::io::Write;
                let report_file = std::fs::File::create(
                    output_directory.join(format!("{}_{}.qr.json", output_prefix, config.infix)),
                )?;
                let mut bufwriter = BufWriter::new(report_file);
                let str_collector: HashMap<String, usize> = config
                    .collector
                    .iter()
                    .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
                    .collect();
                let json = serde_json::to_string_pretty(&str_collector)?;
                bufwriter.write_all(json.as_bytes())?;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}

/// for the cases where the actual data is irrelevant.
fn apply_in_place(
    target: Target,
    f: impl Fn(&mut io::FastQRead),
    block: &mut io::FastQBlocksCombined,
) {
    match target {
        Target::Read1 => {
            for read in block.block_read1.entries.iter_mut() {
                f(read);
            }
        }
        Target::Read2 => {
            for read in block.block_read2.as_mut().unwrap().entries.iter_mut() {
                f(read);
            }
        }
        Target::Index1 => {
            for read in block.block_index1.as_mut().unwrap().entries.iter_mut() {
                f(read);
            }
        }
        Target::Index2 => {
            for read in block.block_index2.as_mut().unwrap().entries.iter_mut() {
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
        Target::Read1 => block.block_read1.apply_mut(f),
        Target::Read2 => block
            .block_read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index1 => block
            .block_index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index2 => block
            .block_index2
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
        Target::Read1 => &block.block_read1,
        Target::Read2 => &block.block_read2.as_ref().unwrap(),
        Target::Index1 => &block.block_index1.as_ref().unwrap(),
        Target::Index2 => &block.block_index2.as_ref().unwrap(),
    };
    let keep: Vec<_> = target.apply(f);
    let mut iter = keep.iter();
    block.block_read1.entries.retain(|_| *iter.next().unwrap());
    if let Some(ref mut read2) = block.block_read2 {
        let mut iter = keep.iter();
        read2.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index1) = block.block_index1 {
        let mut iter = keep.iter();
        index1.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index2) = block.block_index2 {
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
    while let Some(molecule) = block_iter.next() {
        keep.push(f(
            &molecule.0,
            molecule.1.as_ref(),
            molecule.2.as_ref(),
            molecule.3.as_ref(),
        ))
    }

    let mut iter = keep.iter();
    block.block_read1.entries.retain(|_| *iter.next().unwrap());
    if let Some(ref mut read2) = block.block_read2 {
        let mut iter = keep.iter();
        read2.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index1) = block.block_index1 {
        let mut iter = keep.iter();
        index1.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index2) = block.block_index2 {
        let mut iter = keep.iter();
        index2.entries.retain(|_| *iter.next().unwrap());
    }
}
