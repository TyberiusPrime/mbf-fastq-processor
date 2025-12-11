//! Async pipeline implementation using tokio.
//!
//! This is an alternative implementation of the processing pipeline that uses
//! tokio's async runtime instead of crossbeam channels and std threads.
//! It maintains the same order invariants as the sync pipeline.

use anyhow::{Context, Result, bail};
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

use crate::{
    config::{Config, StructuredInput},
    demultiplex::{DemultiplexBarcodes, DemultiplexInfo, OptDemultiplex},
    io::{
        self,
        parsers::{ChainedParser, ThreadCount},
    },
    output::{open_output_files, output_block, output_html_report, output_json_report},
    transformations::{self, FinalizeReportResult, Step, Transformation},
};

/// Message type sent through channels
type BlockMessage = (usize, io::FastQBlocksCombined, Option<usize>);

/// Parse input files and send blocks through the channel
fn parse_and_send_blocking(
    readers: Vec<io::InputFile>,
    raw_tx: &std::sync::mpsc::SyncSender<(io::FastQBlock, Option<usize>)>,
    buffer_size: usize,
    block_size: usize,
    input_thread_count: ThreadCount,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(
        readers,
        block_size,
        buffer_size,
        input_thread_count,
        input_options,
    );
    loop {
        let res = parser.parse()?;
        if !res.fastq_block.entries.is_empty() || !res.was_final {
            if raw_tx
                .send((res.fastq_block, res.expected_read_count))
                .is_err()
            {
                break;
            }
        }
        if res.was_final {
            break;
        }
    }
    Ok(())
}

fn parse_interleaved_and_send_blocking(
    readers: Vec<io::InputFile>,
    combiner_output_tx: &std::sync::mpsc::SyncSender<BlockMessage>,
    segment_count: usize,
    buffer_size: usize,
    input_thread_count: ThreadCount,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(
        readers,
        block_size,
        buffer_size,
        input_thread_count,
        input_options,
    );
    let mut block_no = 1;
    let mut expected_read_count = None;
    loop {
        let res = parser.parse()?;
        if expected_read_count.is_none() && res.expected_read_count.is_some() {
            expected_read_count = res.expected_read_count;
        }
        if !res.fastq_block.entries.is_empty() || !res.was_final {
            let out_blocks = res.fastq_block.split_interleaved(segment_count);
            let out = (
                block_no,
                io::FastQBlocksCombined {
                    segments: out_blocks,
                    output_tags: None,
                    tags: HashMap::default(),
                    is_final: false,
                },
                expected_read_count,
            );
            block_no += 1;
            if combiner_output_tx.send(out).is_err() {
                break;
            }
        }

        if res.was_final {
            let final_block = io::FastQBlocksCombined {
                segments: vec![io::FastQBlock::empty()],
                output_tags: None,
                tags: HashMap::default(),
                is_final: true,
            };
            let _ = combiner_output_tx.send((block_no, final_block, expected_read_count));
            break;
        }
    }
    Ok(())
}

fn run_combiner_thread_blocking(
    raw_rx_readers: Vec<std::sync::mpsc::Receiver<(io::FastQBlock, Option<usize>)>>,
    combiner_output_tx: std::sync::mpsc::SyncSender<BlockMessage>,
    largest_segment_idx: usize,
    error_collector: Arc<Mutex<Vec<String>>>,
) {
    let mut block_no = 1;
    let mut expected_read_count = None;
    loop {
        let mut blocks = Vec::new();
        for receiver in &raw_rx_readers {
            if let Ok((block, block_expected_read_count)) = receiver.recv() {
                if block_no == 1 && blocks.len() == largest_segment_idx {
                    expected_read_count = block_expected_read_count;
                }
                blocks.push(block);
            } else if blocks.is_empty() {
                for other_receiver in &raw_rx_readers[1..] {
                    if let Ok((_block, _block_expected_read_count)) = other_receiver.recv() {
                        error_collector.lock().expect("mutex lock should not be poisoned").push("Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts".to_string());
                    }
                }
                let empty_segments: Vec<io::FastQBlock> = raw_rx_readers
                    .iter()
                    .map(|_| io::FastQBlock::empty())
                    .collect();
                let final_block = io::FastQBlocksCombined {
                    segments: empty_segments,
                    output_tags: None,
                    tags: Default::default(),
                    is_final: true,
                };
                let _ = combiner_output_tx.send((block_no, final_block, expected_read_count));
                return;
            } else {
                error_collector.lock().expect("mutex lock should not be poisoned").push("Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts".to_string());
                return;
            }
        }
        let first_len = blocks[0].len();
        if !blocks.iter().all(|b| b.len() == first_len) {
            error_collector.lock().expect("mutex lock should not be poisoned").push("Unequal block sizes in input segments. This suggests your fastqs have different numbers of reads.".to_string());
            return;
        }
        let out = (
            block_no,
            io::FastQBlocksCombined {
                segments: blocks,
                output_tags: None,
                tags: Default::default(),
                is_final: false,
            },
            expected_read_count,
        );
        block_no += 1;
        if combiner_output_tx.send(out).is_err() {
            break;
        }
    }
}

fn run_benchmark_combiner_thread_blocking(
    first_block: io::FastQBlocksCombined,
    combiner_output_tx: std::sync::mpsc::SyncSender<BlockMessage>,
    molecule_count: usize,
) {
    let mut block_no = 1;
    let mut molecules_sent = 0;
    let block_molecule_count = first_block
        .segments
        .iter()
        .map(|s| s.len())
        .min()
        .unwrap_or(0);

    if block_molecule_count == 0 {
        unreachable!("Empty first block in benchmark. Should have been validated before?");
    }

    while molecules_sent < molecule_count {
        let mut cloned_block = first_block.clone();
        cloned_block.is_final = false;

        let current_block_size = cloned_block
            .segments
            .iter()
            .map(|s| s.len())
            .min()
            .unwrap_or(0);
        molecules_sent += current_block_size;

        if combiner_output_tx
            .send((block_no, cloned_block, Some(molecule_count)))
            .is_err()
        {
            break;
        }

        block_no += 1;

        if molecules_sent >= molecule_count {
            break;
        }
    }

    let final_block = io::FastQBlocksCombined {
        segments: first_block
            .segments
            .iter()
            .map(|_| io::FastQBlock::empty())
            .collect(),
        output_tags: None,
        tags: Default::default(),
        is_final: true,
    };
    let _ = combiner_output_tx.send((block_no, final_block, Some(molecule_count)));
}

fn run_benchmark_interleaved_thread_blocking(
    first_block: io::FastQBlock,
    combiner_output_tx: std::sync::mpsc::SyncSender<BlockMessage>,
    segment_count: usize,
    molecule_count: usize,
) {
    let mut block_no = 1;
    let mut molecules_sent = 0;
    let block_molecule_count = first_block.len();

    if block_molecule_count == 0 {
        panic!("Empty first block in benchmark. Should have been validated before?");
    }

    let out_blocks = first_block.split_interleaved(segment_count);

    while molecules_sent < molecule_count {
        molecules_sent += block_molecule_count;
        let out_blocks = out_blocks.clone();

        let out = (
            block_no,
            io::FastQBlocksCombined {
                segments: out_blocks,
                output_tags: None,
                tags: Default::default(),
                is_final: false,
            },
            Some(molecule_count),
        );

        if combiner_output_tx.send(out).is_err() {
            break;
        }

        block_no += 1;

        if molecules_sent >= molecule_count {
            break;
        }
    }

    let final_block = io::FastQBlocksCombined {
        segments: vec![io::FastQBlock::empty()],
        output_tags: None,
        tags: Default::default(),
        is_final: true,
    };
    let _ = combiner_output_tx.send((block_no, final_block, Some(molecule_count)));
}

pub struct AsyncRunStage0 {
    report_html: bool,
    report_json: bool,
    report_timing: bool,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn checked_f64_to_u16(value: f64) -> Option<u16> {
    if value.is_finite() && value >= 0.0 && value <= f64::from(u16::MAX) {
        Some(value as u16)
    } else {
        None
    }
}

impl AsyncRunStage0 {
    pub fn new(parsed: &Config) -> Self {
        AsyncRunStage0 {
            report_html: parsed.output.as_ref().is_some_and(|o| o.report_html),
            report_json: parsed.output.as_ref().is_some_and(|o| o.report_json),
            report_timing: parsed.output.as_ref().is_some_and(|o| o.report_timing),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn configure_demultiplex_and_init_stages(
        self,
        parsed: &mut Config,
        output_directory: &Path,
        allow_overwrite: bool,
    ) -> Result<AsyncRunStage1> {
        let output_prefix = parsed
            .output
            .as_ref()
            .map_or("mbf_fastq_preprocessor_output", |x| &x.prefix)
            .to_string();
        let output_ix_separator = parsed.get_ix_separator();

        let input_info = transformations::InputInfo {
            segment_order: parsed.input.get_segment_order().clone(),
            barcodes_data: parsed.barcodes.clone(),
            comment_insert_char: parsed.input.options.read_comment_character,
            initial_filter_capacity: None,
        };
        let mut demultiplex_infos: Vec<(usize, OptDemultiplex)> = Vec::new();
        let progress_output = {
            let mut res = None;
            for step in &mut parsed.transform {
                if let Transformation::Progress(inner) = step {
                    inner.init(
                        &input_info,
                        &output_prefix,
                        output_directory,
                        &output_ix_separator,
                        &OptDemultiplex::No,
                        allow_overwrite,
                    )?;
                    res = Some(inner.clone());
                }
            }
            res
        };

        let mut current_bit_start = 0;

        for (index, transform) in (parsed.transform).iter_mut().enumerate() {
            if !matches!(transform, Transformation::Progress(_)) {
                if let Some(progress_output) = &progress_output {
                    transform.store_progress_output(progress_output);
                }
                let new_demultiplex_barcodes: Option<DemultiplexBarcodes> = {
                    let last_demultiplex_info = demultiplex_infos
                        .iter()
                        .last()
                        .map_or(&OptDemultiplex::No, |x| &x.1);
                    transform
                        .init(
                            &input_info,
                            &output_prefix,
                            output_directory,
                            &output_ix_separator,
                            last_demultiplex_info,
                            allow_overwrite,
                        )
                        .context("Transform initialize failed")?
                };
                #[allow(clippy::cast_precision_loss)]
                if let Some(new_demultiplex_barcodes) = new_demultiplex_barcodes {
                    let barcode_count = new_demultiplex_barcodes.barcode_to_name.len()
                        + usize::from(new_demultiplex_barcodes.include_no_barcode);
                    let bits_needed = checked_f64_to_u16((barcode_count as f64).log2().ceil())
                        .expect("Barcodes would not fit into a u16");
                    let mut tag_to_name = BTreeMap::new();
                    if new_demultiplex_barcodes.include_no_barcode {
                        tag_to_name.insert(0, Some("no-barcode".to_string()));
                    } else {
                        tag_to_name.insert(0, None);
                    }

                    let unique_names = new_demultiplex_barcodes
                        .barcode_to_name
                        .values()
                        .collect::<std::collections::BTreeSet<_>>();
                    let unique_names = unique_names.into_iter().cloned().collect::<Vec<_>>();
                    let mut local_name_to_tag = BTreeMap::new();
                    let mut tag_value: crate::demultiplex::Tag = 1;
                    for name in unique_names {
                        let bitpattern = tag_value << current_bit_start;
                        tag_to_name.insert(bitpattern, Some(name.clone()));
                        local_name_to_tag.insert(name, bitpattern);
                        tag_value += 1;
                    }
                    let local_barcode_to_tag = new_demultiplex_barcodes
                        .barcode_to_name
                        .into_iter()
                        .map(|(k, v)| {
                            let tag = local_name_to_tag
                                .get(&v)
                                .expect("tag must exist in local_name_to_tag map");
                            (k, *tag)
                        })
                        .collect();

                    if demultiplex_infos.is_empty() {
                        demultiplex_infos.push((
                            index,
                            OptDemultiplex::Yes(DemultiplexInfo::new(
                                tag_to_name,
                                local_barcode_to_tag,
                            )),
                        ));
                    } else {
                        let mut next = BTreeMap::new();
                        {
                            let last_demultiplex_info = demultiplex_infos
                                .iter()
                                .last()
                                .map_or(&OptDemultiplex::No, |x| &x.1);

                            for (old_tag, old_name) in &last_demultiplex_info.expect("last_demultiplex_info must be Some when iterating over tag_to_name").tag_to_name {
                                for (new_tag, new_name) in &tag_to_name {
                                    let combined_tag = old_tag | new_tag;
                                    let out_name: Option<String> = {
                                        if let Some(old_name) = old_name {
                                            new_name.as_ref().map(|new_name| {
                                                format!(
                                                    "{}{}{}",
                                                    old_name, &output_ix_separator, new_name
                                                )
                                            })
                                        } else {
                                            None
                                        }
                                    };
                                    next.insert(combined_tag, out_name);
                                }
                            }
                        }
                        demultiplex_infos.push((
                            index,
                            OptDemultiplex::Yes(DemultiplexInfo::new(next, local_barcode_to_tag)),
                        ));
                    }
                    current_bit_start += bits_needed;
                    if current_bit_start > 64 {
                        bail!("Too many demultiplexed outputs defined - exceeds 64 bits");
                    }
                }
            }
        }

        Ok(AsyncRunStage1 {
            input_info,
            report_html: self.report_html,
            report_json: self.report_json,
            report_timing: self.report_timing,
            output_directory: output_directory.to_owned(),
            demultiplex_infos,
            allow_overwrite,
        })
    }
}

pub struct AsyncRunStage1 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    allow_overwrite: bool,
}

impl AsyncRunStage1 {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn create_input_threads(self, parsed: &Config) -> Result<AsyncRunStage2> {
        let input_config = &parsed.input;
        let thread_count = parsed.options.thread_count;
        let input_thread_count = ThreadCount(
            thread_count
                .saturating_sub(2)
                .saturating_div(parsed.input.parser_count())
                .max(1),
        );
        let mut input_files =
            io::open_input_files(input_config).context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 50;
        let error_collector = Arc::new(Mutex::new(Vec::<String>::new()));
        let timing_collector = Arc::new(Mutex::new(Vec::<crate::timing::StepTiming>::new()));
        let input_options = parsed.input.options.clone();

        let largest_segment_idx = input_files.largest_segment_idx;

        // Use std::sync::mpsc for the blocking reader threads, then bridge to tokio
        let (bridge_tx, bridge_rx) = std::sync::mpsc::sync_channel::<BlockMessage>(channel_size);

        let input_threads = if let Some(benchmark) = &parsed.benchmark {
            if benchmark.enable {
                let molecule_count = benchmark.molecule_count;

                match parsed
                    .input
                    .structured
                    .as_ref()
                    .expect("structured input must be Some after config validation")
                {
                    StructuredInput::Interleaved { segment_order, .. } => {
                        let segment_order_len = segment_order.len();
                        let combiner_output_tx = bridge_tx;

                        let mut parser = ChainedParser::new(
                            input_files.segment_files.segments.pop().expect(
                                "segments must contain at least one element for interleaved input",
                            ),
                            block_size,
                            buffer_size,
                            input_thread_count,
                            input_options,
                        );

                        let first_block = parser
                            .parse()
                            .context("Failed to read first block for benchmark")?;
                        if first_block.fastq_block.entries.is_empty() {
                            bail!(
                                "Benchmark error: Input is empty - cannot benchmark with empty input"
                            );
                        }

                        let handle = std::thread::Builder::new()
                            .name("AsyncBenchmarkInterleavedReader".into())
                            .spawn(move || {
                                run_benchmark_interleaved_thread_blocking(
                                    first_block.fastq_block,
                                    combiner_output_tx,
                                    segment_order_len,
                                    molecule_count,
                                );
                            })
                            .expect("thread spawn should not fail");

                        vec![handle]
                    }
                    StructuredInput::Segmented { .. } => {
                        let combiner_output_tx = bridge_tx;

                        let mut first_blocks = Vec::new();
                        for this_segments_input_files in
                            input_files.segment_files.segments.into_iter()
                        {
                            let mut parser = ChainedParser::new(
                                this_segments_input_files,
                                block_size,
                                buffer_size,
                                input_thread_count,
                                input_options.clone(),
                            );

                            let first_block = parser
                                .parse()
                                .context("Failed to read first block for benchmark")?;
                            if first_block.fastq_block.entries.is_empty() {
                                bail!(
                                    "Benchmark error: Input segment is empty - cannot benchmark with empty input"
                                );
                            }
                            first_blocks.push(first_block.fastq_block);
                        }

                        let first_len = first_blocks[0].len();
                        if !first_blocks.iter().all(|b| b.len() == first_len) {
                            bail!(
                                "Benchmark error: First blocks of different segments have different sizes. Cannot proceed with benchmark."
                            );
                        }

                        let first_combined = io::FastQBlocksCombined {
                            segments: first_blocks,
                            output_tags: None,
                            tags: Default::default(),
                            is_final: false,
                        };

                        let handle = std::thread::Builder::new()
                            .name("AsyncBenchmarkCombiner".into())
                            .spawn(move || {
                                run_benchmark_combiner_thread_blocking(
                                    first_combined,
                                    combiner_output_tx,
                                    molecule_count,
                                );
                            })
                            .expect("thread spawn should not fail");

                        vec![handle]
                    }
                }
            } else {
                bail!("Benchmark is configured but not enabled");
            }
        } else {
            match parsed
                .input
                .structured
                .as_ref()
                .expect("structured input must be Some after config validation")
            {
                StructuredInput::Interleaved { segment_order, .. } => {
                    let error_collector = error_collector.clone();
                    let segment_order_len = segment_order.len();
                    let combiner_output_tx = bridge_tx;
                    let options = input_options.clone();
                    let handle = std::thread::Builder::new()
                        .name("AsyncInterleavedReader".into())
                        .spawn(move || {
                            if let Err(e) = parse_interleaved_and_send_blocking(
                                input_files.segment_files.segments.pop().expect(
                                    "segments must contain at least one element for interleaved input",
                                ),
                                &combiner_output_tx,
                                segment_order_len,
                                buffer_size,
                                input_thread_count,
                                block_size,
                                options,
                            ) {
                                error_collector
                                    .lock()
                                    .expect("mutex lock should not be poisoned")
                                    .push(format!("Error in interleaved parsing thread: {e:?}"));
                            }
                        })
                        .expect("thread spawn should not fail");

                    vec![handle]
                }
                StructuredInput::Segmented { segment_order, .. } => {
                    let mut threads = Vec::new();
                    let mut raw_rx_readers = Vec::new();
                    for (segment_name, this_segments_input_files) in segment_order
                        .iter()
                        .zip(input_files.segment_files.segments.into_iter())
                    {
                        let segment_name = segment_name.clone();
                        let error_collector = error_collector.clone();
                        let options = input_options.clone();
                        let (raw_tx_read, raw_rx_read) =
                            std::sync::mpsc::sync_channel(channel_size);
                        let read_thread = std::thread::Builder::new()
                            .name(format!("AsyncReader_{segment_name}"))
                            .spawn(move || {
                                if let Err(e) = parse_and_send_blocking(
                                    this_segments_input_files,
                                    &raw_tx_read,
                                    buffer_size,
                                    block_size,
                                    input_thread_count,
                                    options,
                                ) {
                                    error_collector
                                        .lock()
                                        .expect("mutex lock should not be poisoned")
                                        .push(format!(
                                            "Error in reading thread for segment {segment_name}: {e:?}"
                                        ));
                                }
                            })
                            .expect("thread spawn should not fail");
                        threads.push(read_thread);
                        raw_rx_readers.push(raw_rx_read);
                    }

                    {
                        let error_collector = error_collector.clone();
                        let combiner = std::thread::Builder::new()
                            .name("AsyncCombiner".into())
                            .spawn(move || {
                                run_combiner_thread_blocking(
                                    raw_rx_readers,
                                    bridge_tx,
                                    largest_segment_idx,
                                    error_collector,
                                );
                            })
                            .expect("thread spawn should not fail");
                        threads.push(combiner);
                    }
                    threads
                }
            }
        };

        Ok(AsyncRunStage2 {
            input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            report_timing: self.report_timing,
            demultiplex_infos: self.demultiplex_infos,
            input_threads,
            bridge_rx,
            error_collector,
            timing_collector,
            allow_overwrite: self.allow_overwrite,
        })
    }
}

pub struct AsyncRunStage2 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,

    input_threads: Vec<std::thread::JoinHandle<()>>,
    bridge_rx: std::sync::mpsc::Receiver<BlockMessage>,

    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    allow_overwrite: bool,
}

impl AsyncRunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_stage_channels(self, parsed: &mut Config) -> AsyncRunStage3 {
        let stages = std::mem::take(&mut parsed.transform);
        let channel_size = 50;

        // Create tokio mpsc channels for stages
        let mut channels: Vec<_> = (0..=stages.len())
            .map(|_| {
                let (tx, rx) = mpsc::channel::<BlockMessage>(channel_size);
                (Some(tx), Some(rx))
            })
            .collect();

        let thread_count = parsed.options.thread_count;
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));

        let mut stage_configs = Vec::new();

        for (stage_no, stage) in stages.into_iter().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_thread_count = if needs_serial { 1 } else { thread_count };

            let mut demultiplex_info_for_stage = OptDemultiplex::No;
            for (idx, demultiplex_info) in self.demultiplex_infos.iter().rev() {
                if *idx <= stage_no {
                    demultiplex_info_for_stage = demultiplex_info.clone();
                    break;
                }
            }

            let input_rx = channels[stage_no]
                .1
                .take()
                .expect("channel receiver must exist at stage position");
            let output_tx = channels[stage_no + 1]
                .0
                .take()
                .expect("channel sender must exist at next stage position");

            stage_configs.push(StageConfig {
                stage_no,
                stage,
                needs_serial,
                transmits_premature_termination,
                local_thread_count,
                demultiplex_info: demultiplex_info_for_stage,
                input_rx,
                output_tx,
            });
        }

        let last_channel = channels.len() - 1;
        let final_channel = channels[last_channel]
            .1
            .take()
            .expect("final channel receiver must exist");

        // First channel sender for bridging
        let first_tx = channels[0]
            .0
            .take()
            .expect("first channel sender must exist");

        AsyncRunStage3 {
            input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            report_timing: self.report_timing,
            demultiplex_infos: self.demultiplex_infos,
            input_threads: self.input_threads,
            bridge_rx: self.bridge_rx,
            first_tx,
            stage_configs,
            stage_to_output_channel: final_channel,
            report_collector,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
            allow_overwrite: self.allow_overwrite,
        }
    }
}

struct StageConfig {
    stage_no: usize,
    stage: Transformation,
    needs_serial: bool,
    transmits_premature_termination: bool,
    local_thread_count: usize,
    demultiplex_info: OptDemultiplex,
    input_rx: mpsc::Receiver<BlockMessage>,
    output_tx: mpsc::Sender<BlockMessage>,
}

pub struct AsyncRunStage3 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    allow_overwrite: bool,

    input_threads: Vec<std::thread::JoinHandle<()>>,
    bridge_rx: std::sync::mpsc::Receiver<BlockMessage>,
    first_tx: mpsc::Sender<BlockMessage>,
    stage_configs: Vec<StageConfig>,
    stage_to_output_channel: mpsc::Receiver<BlockMessage>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
}

fn handle_stage(
    block: BlockMessage,
    input_info: &transformations::InputInfo,
    stage: &mut Transformation,
    demultiplex_info: &OptDemultiplex,
    timing_collector: &Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    step_no: usize,
    step_type: &str,
) -> anyhow::Result<(BlockMessage, bool)> {
    let block_no = block.0;
    let mut out_block = block.1;
    let expected_read_count = block.2;
    let stage_continue;
    let mut input_info = input_info.clone();
    input_info.initial_filter_capacity = block.2;

    let (wall_start, cpu_start) = crate::timing::StepTiming::start();
    (out_block, stage_continue) =
        stage.apply(out_block, &input_info, block_no, demultiplex_info)?;
    let timing = crate::timing::StepTiming::from_start(
        step_no,
        step_type.to_string(),
        wall_start,
        cpu_start,
    );

    timing_collector
        .lock()
        .expect("mutex lock should not be poisoned")
        .push(timing);

    let do_continue = stage_continue;
    if !do_continue {
        out_block.is_final = true;
    }

    Ok(((block_no, out_block, expected_read_count), do_continue))
}

impl AsyncRunStage3 {
    #[allow(clippy::too_many_lines)]
    pub async fn run_pipeline(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<AsyncRunStage4> {
        let cloned_input_config = parsed.input.clone();
        let output_buffer_size = parsed.options.output_buffer_size;

        let demultiplex_info = self
            .demultiplex_infos
            .iter()
            .last()
            .map_or(OptDemultiplex::No, |x| x.1.clone());

        let mut output_files = open_output_files(
            parsed,
            &self.output_directory,
            &demultiplex_info,
            self.report_html,
            self.report_json,
            self.report_timing,
            self.allow_overwrite,
        )?;

        let output_directory = self.output_directory.clone();
        let report_collector = self.report_collector.clone();
        let error_collector = self.error_collector.clone();
        let timing_collector = self.timing_collector.clone();

        let mut interleave_order = Vec::new();
        if let Some(output) = &parsed.output {
            if let Some(interleave) = &output.interleave {
                for name in interleave {
                    let idx = parsed
                        .input
                        .get_segment_order()
                        .iter()
                        .position(|x| x == name)
                        .expect("segment name must exist in segment_order");
                    interleave_order.push(idx);
                }
            }
        }

        // Bridge from std::sync::mpsc to tokio::mpsc
        let bridge_rx = self.bridge_rx;
        let first_tx = self.first_tx;

        let bridge_task = tokio::task::spawn_blocking(move || {
            while let Ok(msg) = bridge_rx.recv() {
                if first_tx.blocking_send(msg).is_err() {
                    break;
                }
            }
            drop(first_tx);
        });

        // Spawn stage tasks
        let mut stage_handles = Vec::new();

        for config in self.stage_configs {
            let input_info = self.input_info.clone();
            let error_collector = error_collector.clone();
            let timing_collector = timing_collector.clone();
            let report_collector = report_collector.clone();

            let handle = tokio::spawn(async move {
                run_stage_task(
                    config.stage_no,
                    config.stage,
                    config.needs_serial,
                    config.transmits_premature_termination,
                    config.local_thread_count,
                    config.demultiplex_info,
                    config.input_rx,
                    config.output_tx,
                    input_info,
                    error_collector,
                    timing_collector,
                    report_collector,
                )
                .await
            });

            stage_handles.push(handle);
        }

        // Output task
        let mut stage_to_output_channel = self.stage_to_output_channel;
        let output_error_collector = error_collector.clone();

        let output_handle = tokio::spawn(async move {
            let mut last_block_outputted = 0;
            let mut buffer = Vec::new();

            while let Some((block_no, block, _expected_read_count)) =
                stage_to_output_channel.recv().await
            {
                buffer.push((block_no, block));
                loop {
                    let mut send = None;
                    for (ii, (block_no, _block)) in buffer.iter().enumerate() {
                        if block_no - 1 == last_block_outputted {
                            last_block_outputted += 1;
                            send = Some(ii);
                            break;
                        }
                    }
                    if let Some(send_idx) = send {
                        let to_output = buffer.remove(send_idx);
                        if let Err(e) = output_block(
                            &to_output.1,
                            &mut output_files.output_segments,
                            &interleave_order,
                            &demultiplex_info,
                            output_buffer_size,
                        ) {
                            output_error_collector
                                .lock()
                                .expect("mutex lock should not be poisoned")
                                .push(format!("Error in output thread: {e:?}"));
                            return;
                        }
                    } else {
                        break;
                    }
                }
            }

            // Finish output files
            for set_of_output_files in &mut output_files.output_segments {
                if let Err(e) = set_of_output_files
                    .1
                    .lock()
                    .expect("mutex lock should not be poisoned")
                    .finish()
                {
                    output_error_collector
                        .lock()
                        .expect("mutex lock should not be poisoned")
                        .push(format!("Error finishing output files: {e:?}"));
                    return;
                }
            }

            // Generate reports
            let json_report = {
                let need_json = output_files.output_reports.json.is_some()
                    | output_files.output_reports.html.is_some();
                if need_json {
                    match output_json_report(
                        output_files.output_reports.json.as_mut(),
                        &report_collector,
                        &report_labels,
                        &output_directory.to_string_lossy(),
                        &cloned_input_config,
                        &raw_config,
                    ) {
                        Ok(res) => Some(res),
                        Err(e) => {
                            output_error_collector
                                .lock()
                                .expect("mutex lock should not be poisoned")
                                .push(format!("Error writing json report: {e:?}"));
                            return;
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(output_html) = output_files.output_reports.html.as_mut() {
                if let Err(e) = output_html_report(
                    output_html,
                    &json_report.expect("json_report must be Some when html output is enabled"),
                ) {
                    output_error_collector
                        .lock()
                        .expect("mutex lock should not be poisoned")
                        .push(format!("Error writing html report: {e:?}"));
                }
            }

            // Write timing report
            let timings = timing_collector
                .lock()
                .expect("mutex lock should not be poisoned")
                .clone();

            if let Some(timing_file) = output_files.output_reports.timing.as_mut() {
                let stats = crate::timing::aggregate_timings(timings);
                if let Err(e) = write!(
                    timing_file,
                    "{}",
                    serde_json::to_string_pretty(&stats)
                        .expect("JSON serialization to string should not fail"),
                ) {
                    output_error_collector
                        .lock()
                        .expect("mutex lock should not be poisoned")
                        .push(format!("Failed to write timing report: {}", e));
                }
            }
        });

        // Wait for all tasks
        let _ = bridge_task.await;

        for handle in stage_handles {
            let _ = handle.await;
        }

        let _ = output_handle.await;

        Ok(AsyncRunStage4 {
            error_collector,
            input_threads: self.input_threads,
        })
    }
}

async fn run_stage_task(
    stage_no: usize,
    mut stage: Transformation,
    needs_serial: bool,
    transmits_premature_termination: bool,
    _local_thread_count: usize,
    demultiplex_info: OptDemultiplex,
    mut input_rx: mpsc::Receiver<BlockMessage>,
    output_tx: mpsc::Sender<BlockMessage>,
    input_info: transformations::InputInfo,
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
) {
    let step_type = stage.to_string();

    if needs_serial {
        // Serial processing - maintain order
        let mut last_block_outputted = 0;
        let mut buffer: Vec<BlockMessage> = Vec::new();

        while let Some(block) = input_rx.recv().await {
            buffer.push(block);

            loop {
                let mut process_idx = None;
                for (ii, (block_no, _, _)) in buffer.iter().enumerate() {
                    if block_no - 1 == last_block_outputted {
                        process_idx = Some(ii);
                        break;
                    }
                }

                if let Some(idx) = process_idx {
                    let to_process = buffer.remove(idx);
                    last_block_outputted += 1;

                    match handle_stage(
                        to_process,
                        &input_info,
                        &mut stage,
                        &demultiplex_info,
                        &timing_collector,
                        stage_no,
                        &step_type,
                    ) {
                        Ok((out_block, do_continue)) => {
                            if output_tx.send(out_block).await.is_err() {
                                return;
                            }
                            if !do_continue && transmits_premature_termination {
                                break;
                            }
                        }
                        Err(e) => {
                            error_collector
                                .lock()
                                .expect("mutex lock should not be poisoned")
                                .push(format!("Error in stage {stage_no} processing: {e:?}"));
                            return;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        // Finalize
        let report = stage.finalize(&demultiplex_info);
        match report {
            Ok(Some(report)) => {
                report_collector
                    .lock()
                    .expect("mutex lock should not be poisoned")
                    .push(report);
            }
            Ok(None) => {}
            Err(err) => {
                error_collector
                    .lock()
                    .expect("mutex lock should not be poisoned")
                    .push(format!("Error in stage {stage_no} finalization: {err:?}"));
            }
        }
    } else {
        // Parallel processing - blocks can be processed concurrently but order maintained at output
        while let Some(block) = input_rx.recv().await {
            match handle_stage(
                block,
                &input_info,
                &mut stage,
                &demultiplex_info,
                &timing_collector,
                stage_no,
                &step_type,
            ) {
                Ok((out_block, do_continue)) => {
                    if output_tx.send(out_block).await.is_err() {
                        return;
                    }
                    if !do_continue && transmits_premature_termination {
                        break;
                    }
                }
                Err(e) => {
                    error_collector
                        .lock()
                        .expect("mutex lock should not be poisoned")
                        .push(format!("Error in stage {stage_no} processing: {e:?}"));
                    return;
                }
            }
        }
    }
}

pub struct AsyncRunStage4 {
    error_collector: Arc<Mutex<Vec<String>>>,
    input_threads: Vec<std::thread::JoinHandle<()>>,
}

fn collect_thread_failures(
    threads: Vec<std::thread::JoinHandle<()>>,
    msg: &str,
    error_collector: &Arc<Mutex<Vec<String>>>,
) -> Vec<String> {
    let mut stage_errors = Vec::new();
    for s in error_collector
        .lock()
        .expect("mutex lock should not be poisoned")
        .drain(..)
    {
        stage_errors.push(s);
    }
    for p in threads {
        if let Err(e) = p.join() {
            let err_msg = if let Some(e) = e.downcast_ref::<String>() {
                e.to_string()
            } else if let Some(e) = e.downcast_ref::<&str>() {
                (*e).to_string()
            } else {
                format!(
                    "Unknown error: {:?} {:?}",
                    e,
                    std::any::type_name_of_val(&e)
                )
            };
            stage_errors.push(format!("{msg}: {err_msg}"));
        }
    }
    stage_errors
}

impl AsyncRunStage4 {
    pub fn join_threads(self) -> AsyncRunStage5 {
        let mut errors = Vec::new();
        errors.extend(collect_thread_failures(
            self.input_threads,
            "Failure in input thread",
            &self.error_collector,
        ));

        AsyncRunStage5 { errors }
    }
}

pub struct AsyncRunStage5 {
    pub errors: Vec<String>,
}
