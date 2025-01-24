use super::{
    default_name_separator, extract_regions, reproducible_cuckoofilter, OurCuckCooFilter, Target,
};
use crate::config::deser::u8_from_string;
use crate::{demultiplex::Demultiplexed, io};
use anyhow::Result;
use once_cell::sync::OnceCell;
use serde_valid::Validate;
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const PHRED33OFFSET: u8 = 33;

// phred score (33 sanger encoding) to probability of error
// python: ([1.0] * 32 + [10**(q/-10) for q in range(0,256)])[:256]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::excessive_precision)]
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
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformProgress {
    #[serde(skip)]
    pub total_count: Arc<Mutex<usize>>,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    pub n: usize,
    pub output_infix: Option<String>,
    #[serde(skip)]
    pub filename: Option<PathBuf>,
}

impl ConfigTransformProgress {
    pub fn output(&self, msg: &str) {
        if let Some(filename) = self.filename.as_ref() {
            let mut report_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)
                .expect("failed to open progress file");
            writeln!(report_file, "{msg}").expect("failed to write to progress file");
        } else {
            println!("{msg}");
        }
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn transform_progress(
    config: &mut ConfigTransformProgress,
    block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    if config.start_time.is_none() {
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
        let msg: String = if elapsed > 1.0 {
            format!(
                "Processed Total: {} ({:.2} molecules/s), Elapsed: {}s",
                ii,
                rate_total,
                config.start_time.unwrap().elapsed().as_secs()
            )
        } else {
            format!(
                "Processed Total: {}, Elapsed: {}s",
                ii,
                config.start_time.unwrap().elapsed().as_secs()
            )
        };
        config.output(&msg);
    }
    (block, true)
}

const BASE_TO_INDEX: [u8; 256] = {
    let mut out = [4; 256]; //everything else is an N
    out[b'A' as usize] = 0;
    out[b'C' as usize] = 1;
    out[b'G' as usize] = 2;
    out[b'T' as usize] = 3;
    out[b'a' as usize] = 0;
    out[b'c' as usize] = 1;
    out[b'g' as usize] = 2;
    out[b't' as usize] = 3;
    out
};

type PositionCount = [usize; 5];

#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct PositionCountOut {
    a: Vec<usize>,
    c: Vec<usize>,
    g: Vec<usize>,
    t: Vec<usize>,
    n: Vec<usize>,
}

#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct ReportPart1 {
    total_bases: usize,
    q20_bases: usize,
    q30_bases: usize,
    gc_bases: usize,
    per_position_counts: Vec<PositionCount>,
    length_distribution: Vec<usize>,
    expected_errors_from_quality_curve: Vec<f64>,
}
#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct ReportOutput {
    total_bases: usize,
    q20_bases: usize,
    q30_bases: usize,
    gc_bases: usize,
    per_position_counts: PositionCountOut,
    length_distribution: Vec<usize>,
    expected_errors_from_quality_curve: Vec<f64>,
    duplicate_count: usize, // technically a part2 value, but we output only this struct at the end
}

impl ReportOutput {
    fn assemble(part1: &ReportPart1, part2: &ReportPart2) -> Self {
        Self {
            total_bases: part1.total_bases,
            q20_bases: part1.q20_bases,
            q30_bases: part1.q30_bases,
            gc_bases: part1.gc_bases,
            per_position_counts: PositionCountOut {
                a: part1
                    .per_position_counts
                    .iter()
                    .map(|x| x[BASE_TO_INDEX[b'A' as usize] as usize])
                    .collect(),
                c: part1
                    .per_position_counts
                    .iter()
                    .map(|x| x[BASE_TO_INDEX[b'C' as usize] as usize])
                    .collect(),
                g: part1
                    .per_position_counts
                    .iter()
                    .map(|x| x[BASE_TO_INDEX[b'G' as usize] as usize])
                    .collect(),
                t: part1
                    .per_position_counts
                    .iter()
                    .map(|x| x[BASE_TO_INDEX[b'T' as usize] as usize])
                    .collect(),
                n: part1
                    .per_position_counts
                    .iter()
                    .map(|x| x[BASE_TO_INDEX[b'N' as usize] as usize])
                    .collect(),
            },
            length_distribution: part1.length_distribution.clone(),
            expected_errors_from_quality_curve: part1.expected_errors_from_quality_curve.clone(),
            duplicate_count: part2.duplicate_count,
        }
    }
}

impl ReportPart1 {
    fn fill_in(&mut self) {
        let mut reads_with_at_least_this_length = vec![0; self.length_distribution.len()];
        let mut running = 0;
        for (ii, count) in self.length_distribution.iter().enumerate().rev() {
            running += count;
            reads_with_at_least_this_length[ii] = running;
        }
        for (ii, item) in self
            .expected_errors_from_quality_curve
            .iter_mut()
            .enumerate()
        {
            *item /= reads_with_at_least_this_length[ii] as f64;
        }
        let a_bases: usize = self
            .per_position_counts
            .iter()
            .map(|x| x[BASE_TO_INDEX[b'A' as usize] as usize])
            .sum();
        let c_bases: usize = self
            .per_position_counts
            .iter()
            .map(|x| x[BASE_TO_INDEX[b'C' as usize] as usize])
            .sum();
        let g_bases: usize = self
            .per_position_counts
            .iter()
            .map(|x| x[BASE_TO_INDEX[b'G' as usize] as usize])
            .sum();
        let t_bases: usize = self
            .per_position_counts
            .iter()
            .map(|x| x[BASE_TO_INDEX[b'T' as usize] as usize])
            .sum();
        let n_bases: usize = self
            .per_position_counts
            .iter()
            .map(|x| x[BASE_TO_INDEX[b'N' as usize] as usize])
            .sum();
        self.gc_bases = g_bases + c_bases;
        self.total_bases = a_bases + c_bases + g_bases + t_bases + n_bases
    }
}

impl ReportPart2 {
    fn fill_in(&mut self) {
        self.duplication_filter.take();
    }
}
#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct ReportPart2 {
    duplicate_count: usize,
    #[serde(skip)]
    duplication_filter: Option<OurCuckCooFilter>,
}

unsafe impl Send for ReportPart2 {} //fine as long as duplication_filter is None

#[derive(serde::Serialize, Debug, Clone)]
pub struct ReportData<T> {
    program_version: String,
    read_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    read1: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read2: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index1: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index2: Option<T>,
}

impl<T> Default for ReportData<T> {
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
#[serde(deny_unknown_fields)]
pub struct ConfigTransformReport {
    pub infix: String,
    pub json: bool,
    pub html: bool,
    #[serde(default)]
    pub debug_reproducibility: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ConfigTransformReportPart1 {
    //#[serde(skip)]
    pub data: Vec<ReportData<ReportPart1>>,
    pub to_part2: Arc<Mutex<OnceCell<Vec<ReportData<ReportPart1>>>>>,
}
#[derive(Debug, Default, Clone)]

pub struct ConfigTransformReportPart2 {
    //#[serde(skip)]
    pub data: Vec<ReportData<ReportPart2>>,
    pub config: ConfigTransformReport,
    pub from_part1: Arc<Mutex<OnceCell<Vec<ReportData<ReportPart1>>>>>,
}

pub fn init_report_part1(
    config: &mut ConfigTransformReportPart1,
    demultiplex_info: &Demultiplexed,
) {
    //if there's a demultiplex step *before* this report,
    match demultiplex_info {
        Demultiplexed::No => {
            config.data.push(ReportData::default());
        }
        Demultiplexed::Yes(demultiplex_info) => {
            let mut report_data = Vec::new();
            for _ in 0..demultiplex_info.len_outputs() {
                //yeah, we include no-barcode anyway.
                // It's fairly cheap
                report_data.push(ReportData::default());
            }
            config.data = report_data;
        }
    }
}

pub fn transform_report_part1(
    config: &mut Box<ConfigTransformReportPart1>,
    block: crate::io::FastQBlocksCombined,
    demultiplex_info: &Demultiplexed,
) -> (crate::io::FastQBlocksCombined, bool) {
    fn update_from_read_part1(target: &mut ReportPart1, read: &io::WrappedFastQRead) {
        //this is terribly slow right now.
        //I need to multicore and aggregate this.
        let read_len = read.len();
        if target.length_distribution.len() <= read_len {
            //println!("Had to resize report buffer, {read_len}");
            target.length_distribution.resize(read_len + 1, 0);
            target
                .per_position_counts
                .resize(read_len, PositionCount::default());
            target
                .expected_errors_from_quality_curve
                .resize(read_len, 0.0);
        }
        target.length_distribution[read_len] += 1;

        //
        //this takes about 3s on data/large/ERR12828869_1.fq
        let q20_bases = 0;
        let q30_bases = 0;

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
        let seq: &[u8] = read.seq();
        for (ii, base) in seq.iter().enumerate() {
            // using the lookup table is *much* faster than a match
            // and only very slightly slower than using base & 0x7 as index
            // into an array of size 8. And unlike the 0x7 bit trick
            // it is not wrongly mapping non bases to agct
            let idx = BASE_TO_INDEX[*base as usize];
            target.per_position_counts[ii][idx as usize] += 1;
        }

        //todo:
        //kmer count?
        //overrepresented_sequencs (how is that done in fastp)
        //min, maximum read length? - that's something for the finalization though.
    }
    for tag in demultiplex_info.iter_tags() {
        // no need to capture no-barcode if we're
        // not outputing it
        let output = &mut config.data[tag as usize];
        for (storage, read_block) in [
            (&mut output.read1, Some(&block.read1)),
            (&mut output.read2, block.read2.as_ref()),
            (&mut output.index1, block.index1.as_ref()),
            (&mut output.index2, block.index2.as_ref()),
        ] {
            if read_block.is_some() {
                if storage.is_none() {
                    *storage = Some(ReportPart1::default());
                }
                let mut iter = match &block.output_tags {
                    Some(output_tags) => read_block
                        .as_ref()
                        .unwrap()
                        .get_pseudo_iter_filtered_to_tag(tag, output_tags),
                    None => read_block.as_ref().unwrap().get_pseudo_iter(),
                };
                while let Some(read) = iter.pseudo_next() {
                    update_from_read_part1(storage.as_mut().unwrap(), &read);
                }
            }
        }
    }
    (block, true)
}

pub fn finalize_report_part1(
    config: &mut ConfigTransformReportPart1,
    demultiplex_info: &Demultiplexed,
) {
    for tag in demultiplex_info.iter_tags() {
        let report_data = &mut config.data[tag as usize];
        for p in [
            &mut report_data.read1,
            &mut report_data.read2,
            &mut report_data.index1,
            &mut report_data.index2,
        ] {
            if let Some(p) = p.as_mut() {
                p.fill_in();
            }
        }
        config.data[tag as usize].read_count = report_data
            .read1
            .as_ref()
            .unwrap()
            .length_distribution
            .iter()
            .sum();
    }
    config
        .to_part2
        .lock()
        .expect("Failed to retrieve report data lock?")
        .set(config.data.clone())
        .expect("failed to retrieve report data lock?");
}

pub fn init_report_part2(
    config: &mut ConfigTransformReportPart2,
    demultiplex_info: &Demultiplexed,
) {
    //if there's a demultiplex step *before* this report,
    match demultiplex_info {
        Demultiplexed::No => {
            config.data.push(ReportData::default());
        }
        Demultiplexed::Yes(demultiplex_info) => {
            let mut report_data = Vec::new();
            for _ in 0..demultiplex_info.len_outputs() {
                //yeah, we include no-barcode anyway.
                // It's fairly cheap
                report_data.push(ReportData::default());
            }
            config.data = report_data;
        }
    }
}
pub fn transform_report_part2(
    config: &mut Box<ConfigTransformReportPart2>,
    block: crate::io::FastQBlocksCombined,
    demultiplex_info: &Demultiplexed,
) -> (crate::io::FastQBlocksCombined, bool) {
    fn update_from_read_part2(target: &mut ReportPart2, read: &io::WrappedFastQRead) {
        let seq = read.seq();

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
    let (initial_capacity, false_positive_probability) = if config.config.debug_reproducibility {
        (100, 0.1)
    } else {
        (1_000_000, 0.01)
    };

    for tag in demultiplex_info.iter_tags() {
        // no need to capture no-barcode if we're
        // not outputing it
        let output = &mut config.data[tag as usize];
        for (storage, read_block) in [
            (&mut output.read1, Some(&block.read1)),
            (&mut output.read2, block.read2.as_ref()),
            (&mut output.index1, block.index1.as_ref()),
            (&mut output.index2, block.index2.as_ref()),
        ] {
            if read_block.is_some() {
                if storage.is_none() {
                    *storage = Some(ReportPart2::default());
                    storage.as_mut().unwrap().duplication_filter = Some(reproducible_cuckoofilter(
                        42,
                        initial_capacity,
                        false_positive_probability,
                    ));
                }
                let mut iter = match &block.output_tags {
                    Some(output_tags) => read_block
                        .as_ref()
                        .unwrap()
                        .get_pseudo_iter_filtered_to_tag(tag, output_tags),
                    None => read_block.as_ref().unwrap().get_pseudo_iter(),
                };
                while let Some(read) = iter.pseudo_next() {
                    update_from_read_part2(storage.as_mut().unwrap(), &read);
                }
            }
        }
    }
    (block, true)
}

pub fn finalize_report_part2(
    config: &mut Box<ConfigTransformReportPart2>,
    output_prefix: &str,
    output_directory: &Path,
    demultiplex_info: &Demultiplexed,
) -> Result<()> {
    if !(config.config.json || config.config.html) {
        unreachable!()
    }
    dbg!("retrieved data");
    let mut from_part1 = config
        .from_part1
        .lock()
        .expect("failed to retrieve report data lock?");
    //ake sure part 1 has depositied it's thing
    from_part1.wait();
    let mut part1 = from_part1.take().unwrap();
    for tag in demultiplex_info.iter_tags() {
        let part1_data = &mut part1[tag as usize];
        let part2_data = &mut config.data[tag as usize];
        let mut out: ReportData<ReportOutput> = ReportData {
            read_count: part1_data.read_count,
            ..Default::default()
        };
        for (p1, p2, o1) in [
            (&part1_data.read1, &mut part2_data.read1, &mut out.read1),
            (&part1_data.read2, &mut part2_data.read2, &mut out.read2),
            (&part1_data.index1, &mut part2_data.index1, &mut out.index1),
            (&part1_data.index2, &mut part2_data.index2, &mut out.index2),
        ] {
            if let Some(p2) = p2.as_mut() {
                p2.fill_in();
                *o1 = Some(ReportOutput::assemble(p1.as_ref().unwrap(), p2));
            }
        }
        let data = out;

        let barcode_name = demultiplex_info.get_name(tag);
        let barcode_infix = match barcode_name {
            Some(x) => format!("_{x}"),
            None => String::new(),
        };

        let prefix = format!("{}_{}{}", output_prefix, config.config.infix, barcode_infix);

        if config.config.json {
            let report_file =
                std::fs::File::create(output_directory.join(format!("{prefix}.json")))?;
            let mut bufwriter = BufWriter::new(report_file);
            serde_json::to_writer_pretty(&mut bufwriter, &data)?;
        }
        if config.config.html {
            let report_file =
                std::fs::File::create(output_directory.join(format!("{prefix}.html")))?;
            let mut bufwriter = BufWriter::new(report_file);
            let template = include_str!("../../html/template.html");
            let chartjs = include_str!("../../html/chart/chart.umd.min.js");
            let json = serde_json::to_string_pretty(&data).unwrap();
            let html = template
                .replace("%TITLE%", &config.config.infix)
                .replace("\"%DATA%\"", &json)
                .replace("/*%CHART%*/", chartjs);
            bufwriter.write_all(html.as_bytes())?;
        }
    }
    Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformInspect {
    pub n: usize,
    pub target: Target,
    pub infix: String,
    #[serde(skip)]
    pub collector: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
}

pub fn transform_inspect(
    config: &mut ConfigTransformInspect,
    block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let collector = &mut config.collector;
    let source = match config.target {
        Target::Read1 => &block.read1,
        Target::Read2 => block.read2.as_ref().unwrap(),
        Target::Index1 => block.index1.as_ref().unwrap(),
        Target::Index2 => block.index2.as_ref().unwrap(),
    };
    if collector.len() < config.n {
        let mut iter = source.get_pseudo_iter();
        while let Some(read) = iter.pseudo_next() {
            if collector.len() >= config.n {
                break;
            }
            collector.push((
                read.name().to_vec(),
                read.seq().to_vec(),
                read.qual().to_vec(),
            ));
        }
    }
    (block, true)
}

pub fn finalize_inspect(
    config: &mut ConfigTransformInspect,
    output_prefix: &str,
    output_directory: &Path,
    _demultiplex_info: &Demultiplexed,
) -> Result<()> {
    use std::io::Write;
    let target = match config.target {
        Target::Read1 => "1",
        Target::Read2 => "2",
        Target::Index1 => "i1",
        Target::Index2 => "i2",
    };
    let report_file = std::fs::File::create(
        output_directory.join(format!("{}_{}_{}.fq", output_prefix, config.infix, target)),
    )?;
    let mut bufwriter = BufWriter::new(report_file);
    for (name, seq, qual) in &config.collector {
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

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformQuantifyRegions {
    pub infix: String,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_separator"
    )]
    pub separator: Vec<u8>,
    #[validate(min_items = 1)]
    pub regions: Vec<crate::config::RegionDefinition>,

    #[serde(skip)]
    pub collector: HashMap<Vec<u8>, usize>,
}

pub fn transform_quantify_regions(
    config: &mut ConfigTransformQuantifyRegions,
    block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let collector = &mut config.collector;
    for ii in 0..block.read1.len() {
        let key = extract_regions(ii, &block, &config.regions, &config.separator);
        *collector.entry(key).or_insert(0) += 1;
    }
    (block, true)
}
