use anyhow::{Context, Result, bail};
use crossbeam::channel::bounded;
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    config::{Config, StructuredInput},
    demultiplex::{DemultiplexBarcodes, DemultiplexInfo, OptDemultiplex},
    io::{self, parsers::ChainedParser},
    output::{open_output_files, output_block, output_html_report, output_json_report},
    transformations::{self, FinalizeReportResult, Step, Transformation},
};

#[allow(clippy::collapsible_if)]
fn parse_and_send(
    readers: Vec<io::InputFile>,
    raw_tx: &crossbeam::channel::Sender<(io::FastQBlock, Option<usize>)>,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
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

fn parse_interleaved_and_send(
    readers: Vec<io::InputFile>,
    combiner_output_tx: &crossbeam::channel::Sender<(
        usize,
        io::FastQBlocksCombined,
        Option<usize>,
    )>,
    segment_count: usize,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
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
            } else {
            }
        }

        if res.was_final {
            // Send final empty block
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

pub struct RunStage0 {
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

impl RunStage0 {
    pub fn new(parsed: &Config) -> Self {
        RunStage0 {
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
    ) -> Result<RunStage1> {
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
            initial_filter_capacity: None, // Filled after the first block.
        };
        let mut demultiplex_infos: Vec<(usize, OptDemultiplex)> = Vec::new();
        // we need to initialize the progress_output first
        // so we can store it on each stage before the stages' init
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

        // we combinatorially combine demultiplex stages
        // and at each stage, it get's to see the tag->output names up to the latest defined
        // demultiplexing step
        // We then have two
        let mut current_bit_start = 0;

        for (index, transform) in (parsed.transform).iter_mut().enumerate() {
            if !matches!(transform, Transformation::Progress(_)) {
                //progress was initialized before hand
                //
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
                            let tag = local_name_to_tag.get(&v).unwrap();
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

                            for (old_tag, old_name) in &last_demultiplex_info.unwrap().tag_to_name {
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

        Ok(RunStage1 {
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

pub struct RunStage1 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    //demultiplex_info: Demultiplex,
    //demultiplex_start: usize,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    allow_overwrite: bool,
}

impl RunStage1 {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn create_input_threads(self, parsed: &Config) -> Result<RunStage2> {
        let input_config = &parsed.input;
        let thread_count = parsed.options.thread_count;
        let mut input_files = io::open_input_files(input_config, thread_count)
            .context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 2;
        let error_collector = Arc::new(Mutex::new(Vec::<String>::new()));
        let timing_collector = Arc::new(Mutex::new(Vec::<crate::timing::StepTiming>::new()));
        let input_options = parsed.input.options.clone();

        let largest_segment_idx = input_files.largest_segment_idx;

        let (input_threads, combiner_thread, combiner_output_rx) = match parsed
            .input
            .structured
            .as_ref()
            .unwrap()
        {
            StructuredInput::Interleaved { segment_order, .. } => {
                let error_collector = error_collector.clone();
                let segment_order_len = segment_order.len();
                let input_threads = Vec::new();
                let (combiner_output_tx, combiner_output_rx) =
                    bounded::<(usize, io::FastQBlocksCombined, Option<usize>)>(channel_size);
                let options = input_options.clone();
                let combiner_thread = thread::Builder::new()
                    .name("InterleavedReader".into())
                    .spawn(move || {
                        if let Err(e) = parse_interleaved_and_send(
                            input_files.segment_files.segments.pop().unwrap(),
                            &combiner_output_tx,
                            segment_order_len,
                            buffer_size,
                            block_size,
                            options,
                        ) {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error in interleaved parsing thread: {e:?}"));
                        }
                    })
                    .unwrap();

                /* vec![(
                    "interleaved".to_string(),
                    input_files.segments.pop().unwrap(),
                )]; */
                (input_threads, combiner_thread, combiner_output_rx)
            }
            StructuredInput::Segmented { segment_order, .. } => {
                // we spawn one reading thread per input segment for reading & decompressing.
                // and another thread that collects the blocks into combined blocks
                let mut threads = Vec::new();
                let mut raw_rx_readers = Vec::new();
                for (segment_name, this_segments_input_files) in segment_order
                    .iter()
                    .zip(input_files.segment_files.segments.into_iter())
                {
                    let segment_name = segment_name.clone();
                    let error_collector = error_collector.clone();
                    let options = input_options.clone();
                    let (raw_tx_read, raw_rx_read) = bounded(channel_size);
                    let read_thread = thread::Builder::new()
                        .name(format!("Reader_{segment_name}"))
                        .spawn(move || {
                            if let Err(e) = parse_and_send(
                                this_segments_input_files,
                                &raw_tx_read,
                                buffer_size,
                                block_size,
                                options,
                            ) {
                                error_collector.lock().unwrap().push(format!(
                                    "Error in reading thread for segment {segment_name}: {e:?}"
                                ));
                            }
                        })
                        .unwrap();
                    threads.push(read_thread);
                    raw_rx_readers.push(raw_rx_read);
                }
                let (combiner_output_tx, combiner_output_rx) =
                    bounded::<(usize, io::FastQBlocksCombined, Option<usize>)>(channel_size);

                {
                    let error_collector = error_collector.clone();
                    let combiner = thread::Builder::new()
            .name("Combiner".into())
            .spawn(move || {
                //I need to receive the blocks (from all segment input threads)
                //and then, match them up into something that's the same length!
                let mut block_no = 1; // for the sorting later on.
                let mut expected_read_count = None;
                loop {
                    let mut blocks = Vec::new();
                    for receiver in &raw_rx_readers {
                        //since we read the channels in order,
                        //the resulting blocks will also be in order.
                        if let Ok((block, block_expected_read_count)) = receiver.recv() {
                            if block_no == 1 && blocks.len() == largest_segment_idx
                            {
                                //println!("Received expected read count for largest segment: {:?}", block_expected_read_count);
                                expected_read_count = block_expected_read_count;
                            }
                            blocks.push(block);
                        } else if blocks.is_empty() {
                                //The first segment reader is done.
                                //that's the expected behaviour when we're running out of reads.
                                //now every other reader should also be returning an error.
                                //because otherwise the others have more remaining reads
                                for other_receiver in &raw_rx_readers[1..] {
                                    if let Ok((_block, _block_expected_read_count)) = other_receiver.recv() {
                                        error_collector.lock().unwrap().push("Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts".to_string());
                                    }
                                }
                                // Send final empty block
                                let empty_segments: Vec::<io::FastQBlock> =
                                    raw_rx_readers.iter().map(|_| io::FastQBlock::empty()).collect();
                                let final_block = io::FastQBlocksCombined {
                                    segments: empty_segments,
                                    output_tags: None,
                                    tags: Default::default(),
                                    is_final: true,
                                };
                                let _ = combiner_output_tx.send((block_no, final_block, expected_read_count));
                                return;
                        } else {
                            error_collector.lock().unwrap().push("Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts".to_string());

                                return;
                            }
                    }
                    // make sure they all have the same length
                    let first_len = blocks[0].len();
                    if !blocks.iter().all(|b| b.len() == first_len) {
                        error_collector.lock().unwrap().push("Unequal block sizes in input segments. This suggests your fastqs have different numbers of reads.".to_string());
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
                        expected_read_count
                    );
                    block_no += 1;
                    match combiner_output_tx.send(out) {
                        Ok(()) => {}
                        Err(_) => {
                            //downstream hung up
                            break;
                        }
                    }
                }
            })
            .unwrap();
                    (threads, combiner, combiner_output_rx)
                }
            }
        };

        Ok(RunStage2 {
            input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            report_timing: self.report_timing,
            demultiplex_infos: self.demultiplex_infos,
            input_threads,
            combiner_thread,
            combiner_output_rx,
            error_collector,
            timing_collector,
            allow_overwrite: self.allow_overwrite,
        })
    }
}

pub struct RunStage2 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    combiner_output_rx:
        crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>,

    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    allow_overwrite: bool,
}
impl RunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_stage_threads(self, parsed: &mut Config) -> RunStage3 {
        //take the stages out of parsed now
        let stages = std::mem::take(&mut parsed.transform);
        let channel_size = 50;

        let mut channels: Vec<_> = (0..=stages.len())
            .map(|_| {
                let (tx, rx) =
                    bounded::<(usize, io::FastQBlocksCombined, Option<usize>)>(channel_size);
                (Some(tx), Some(rx))
            })
            .collect();
        channels[0].1 = Some(self.combiner_output_rx);

        let thread_count = parsed.options.thread_count;
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));
        let mut threads = Vec::new();

        //now: needs_serial stages are never cloned.
        //So we can make them panic on clone.
        //while the parallel stages must implement a valid clone.

        for (stage_no, mut stage) in stages.into_iter().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_thread_count = if needs_serial { 1 } else { thread_count };
            /* let mut stage = if needs_serial {
                stage.move_inited()
            } else {
                stage.clone()
            }; */
            //let demultiplex_infos2 = self.demultiplex_infos.clone();
            let report_collector = report_collector.clone();
            let input_rx = channels[stage_no].1.take().unwrap();
            let output_tx = channels[stage_no + 1].0.take().unwrap();

            if needs_serial {
                //I suppose we could RC this, but it's only a few dozen bytes, typically.
                //we used to have it on the SegmentIndex, but that's a lot of duplication
                //and only used by a couple of transforms

                let input_rx2 = input_rx;
                let output_tx2 = output_tx;
                let error_collector = self.error_collector.clone();
                let timing_collector = self.timing_collector.clone();
                let input_info: transformations::InputInfo = (self.input_info).clone();
                let step_type = stage.to_string();
                let mut demultiplex_info_for_stage = OptDemultiplex::No;
                for (idx, demultiplex_info) in self.demultiplex_infos.iter().rev() {
                    if *idx <= stage_no {
                        demultiplex_info_for_stage = demultiplex_info.clone();
                        break;
                    }
                }
                threads.push(
                        thread::Builder::new()
                            .name(format!("Serial_stage {stage_no}"))
                            .spawn(move || {
                                //we need to ensure the blocks are passed on in order
                                let mut last_block_outputted = 0;
                                let mut buffer = Vec::new();
                                'outer: while let Ok((block_no, block, expected_read_count)) = input_rx2.recv() {
                                    buffer.push((block_no, block, expected_read_count));
                                    loop {
                                        let mut send = None;
                                        for (ii, (block_no, _block, _expected_output)) in buffer.iter().enumerate() {
                                            if block_no - 1 == last_block_outputted {
                                                last_block_outputted += 1;
                                                send = Some(ii);
                                                break;
                                            }
                                        }
                                        if let Some(send_idx) = send {
                                            let to_output = buffer.remove(send_idx);
                                            {
                                                let result = handle_stage(
                                                    to_output,
                                                    &input_info,
                                                    &output_tx2,
                                                    &mut stage,
                                                    &demultiplex_info_for_stage,
                                                    &timing_collector,
                                                    stage_no,
                                                    &step_type,
                                                );
                                                match result {
                                                    Ok(do_continue) => {
                                                        if !do_continue && transmits_premature_termination {
                                                            break 'outer;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // Send error to main thread and break
                                                        error_collector.lock().unwrap().push(format!("Error in stage {stage_no} processing: {e:?}"));
                                                            break 'outer;
                                                    }
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                let report = stage
                                    .finalize(
                                        &demultiplex_info_for_stage
                                    );
                                match report {
                                    Ok(Some(report)) => {
                                        report_collector.lock().unwrap().push(report);
                                    },
                                    Ok(None) => {},
                                    Err(err) => {
                                        error_collector.lock().unwrap().push(
                                            format!(
                                                "Error in stage {stage_no} finalization: {err:?}"
                                    ));
                                    }
                                }
                            })
                            .unwrap(),
                    );
            } else {
                let step_type = stage.to_string();
                for _ in 0..local_thread_count {
                    let input_info: transformations::InputInfo = (self.input_info).clone();
                    let input_rx2 = input_rx.clone();
                    let output_tx2 = output_tx.clone();
                    let error_collector = self.error_collector.clone();
                    let timing_collector = self.timing_collector.clone();
                    let step_type = step_type.clone();
                    let mut stage = stage.clone();

                    let mut demultiplex_info_for_stage = OptDemultiplex::No;
                    for (idx, demultiplex_info) in self.demultiplex_infos.iter().rev() {
                        if *idx <= stage_no {
                            demultiplex_info_for_stage = demultiplex_info.clone();
                            break;
                        }
                    }
                    threads.push(
                        thread::Builder::new()
                            .name(format!("Stage {stage_no}"))
                            .spawn(move || {
                                loop {
                                    match input_rx2.recv() {
                                        Ok(block) => {
                                            match handle_stage(
                                                block,
                                                &input_info,
                                                &output_tx2,
                                                &mut stage,
                                                &demultiplex_info_for_stage,
                                                &timing_collector,
                                                stage_no,
                                                &step_type,
                                            ) {
                                                Ok(true) => {}
                                                Ok(false) => {
                                                    if transmits_premature_termination {
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    // For now, panic - will be improved in Phase 4
                                                    error_collector.lock().unwrap().push(format!(
                                                    "Error in stage {stage_no} processing: {e:?}"
                                                ));
                                                    return;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            return;
                                        }
                                    }
                                    //no finalize for parallel stages at this point.
                                }
                            })
                            .unwrap(),
                    );
                }
            }
        }
        let last_channel = channels.len() - 1;
        let final_channel = channels[last_channel].1.take().unwrap();
        RunStage3 {
            //input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            report_timing: self.report_timing,
            demultiplex_infos: self.demultiplex_infos,
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            stage_threads: threads,
            stage_to_output_channel: final_channel,
            report_collector,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
            allow_overwrite: self.allow_overwrite,
        }
    }
}

pub struct RunStage3 {
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    allow_overwrite: bool,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    stage_threads: Vec<thread::JoinHandle<()>>,
    stage_to_output_channel:
        crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined, Option<usize>)>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
}

fn collect_thread_failures(
    threads: Vec<thread::JoinHandle<()>>,
    msg: &str,
    error_collector: &Arc<Mutex<Vec<String>>>,
) -> Vec<String> {
    let mut stage_errors = Vec::new();
    for s in error_collector.lock().unwrap().drain(..) {
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

impl RunStage3 {
    #[allow(clippy::too_many_lines)]
    pub fn create_output_threads(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<RunStage4> {
        let input_channel = self.stage_to_output_channel;
        let output_buffer_size = parsed.options.output_buffer_size;
        let cloned_input_config = parsed.input.clone();

        let demultiplex_info = self
            .demultiplex_infos
            .iter()
            .last()
            .map_or(OptDemultiplex::No, |x| x.1.clone()); // we pass int onto the thread later on.

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

        let mut interleave_order = Vec::new();
        if let Some(output) = &parsed.output {
            if let Some(interleave) = &output.interleave {
                for name in interleave {
                    let idx = parsed
                        .input
                        .get_segment_order()
                        .iter()
                        .position(|x| x == name)
                        .unwrap();
                    interleave_order.push(idx);
                }
            }
        }

        let output = {
            let error_collector = self.error_collector.clone();
            thread::Builder::new()
                .name("output".into())
                .spawn(move || {
                    let mut last_block_outputted = 0;
                    let mut buffer = Vec::new();
                    while let Ok((block_no, block, _expected_read_count)) = input_channel.recv() {
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
                                    error_collector
                                        .lock()
                                        .unwrap()
                                        .push(format!("Error in output thread: {e:?}"));
                                    return;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    //all blocks are done, the stage output channel has been closed.
                    //but that doesn't mean the threads are done and have pushed the reports.
                    //so we join em here
                    /* let stage_errors =
                    collect_thread_failures(self.stage_threads, "stage error", &error_collector); */
                    for thread in self.stage_threads {
                        thread.join().expect("thread join failure");
                    }
                    /* assert!(
                        stage_errors.is_empty(),
                        "Error in stage threads occured: {stage_errors:?}"
                    ); */

                    for set_of_output_files in &mut output_files.output_segments {
                        if let Err(e) = set_of_output_files.1.lock().unwrap().finish() {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error finishing output files: {e:?}"));
                            return;
                        }
                    }
                    let json_report = {
                        let need_json = output_files.output_reports.json.is_some()
                            | output_files.output_reports.html.is_some();
                        if need_json {
                            match output_json_report(
                                output_files.output_reports.json.as_mut(), // None if no .json file
                                // generated
                                &report_collector,
                                &report_labels,
                                &output_directory.to_string_lossy(),
                                &cloned_input_config,
                                &raw_config,
                            ) {
                                Ok(res) => Some(res),
                                Err(e) => {
                                    error_collector
                                        .lock()
                                        .unwrap()
                                        .push(format!("Error writing json report: {e:?}"));
                                    return;
                                }
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(output_html) = output_files.output_reports.html.as_mut() {
                        if let Err(e) = output_html_report(output_html, &json_report.unwrap()) {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error writing html report: {e:?}"));
                        }
                    }

                    // Extract timing data
                    let timings = self.timing_collector.lock().unwrap().clone();

                    // Write timing report if enabled
                    if let Some(timeing_file) = output_files.output_reports.timing.as_mut() {
                        let stats = crate::timing::aggregate_timings(timings);
                        if let Err(e) = write!(
                            timeing_file,
                            "{}",
                            serde_json::to_string_pretty(&stats).unwrap(),
                        ) {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Failed to write timing report: {}", e));
                        }
                    }
                })
                .unwrap()
        };

        Ok(RunStage4 {
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            output_thread: output,
            error_collector: self.error_collector,
        })
    }
}

pub struct RunStage4 {
    error_collector: Arc<Mutex<Vec<String>>>,
    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    output_thread: thread::JoinHandle<()>,
}

impl RunStage4 {
    pub fn join_threads(self) -> RunStage5 {
        let mut errors = Vec::new();
        for (threads, msg) in [
            (vec![self.output_thread], "Failure in output thread"),
            (
                vec![self.combiner_thread],
                "Failure in read-combination-thread thread",
            ),
            //            (self.stage_threads, "Failure in stage processor thread"),
            (self.input_threads, "Failure in input thread"),
        ] {
            errors.extend(collect_thread_failures(threads, msg, &self.error_collector));
        }

        RunStage5 { errors }
    }
}

pub struct RunStage5 {
    pub errors: Vec<String>,
}

fn handle_stage(
    block: (usize, io::FastQBlocksCombined, Option<usize>),
    input_info: &transformations::InputInfo,
    output_tx2: &crossbeam::channel::Sender<(usize, io::FastQBlocksCombined, Option<usize>)>,
    stage: &mut Transformation,
    demultiplex_info: &OptDemultiplex,
    timing_collector: &Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    step_no: usize,
    step_type: &str,
) -> anyhow::Result<bool> {
    let mut out_block = block.1;
    let expected_read_count = block.2;
    let stage_continue;
    let mut input_info = input_info.clone();
    input_info.initial_filter_capacity = block.2;

    // Record timing for this step (both wall and CPU time)
    let (wall_start, cpu_start) = crate::timing::StepTiming::start();
    (out_block, stage_continue) = stage.apply(out_block, &input_info, block.0, demultiplex_info)?;
    let timing = crate::timing::StepTiming::from_start(
        step_no,
        step_type.to_string(),
        wall_start,
        cpu_start,
    );

    // Push timing data to collector
    timing_collector.lock().unwrap().push(timing);

    let do_continue = stage_continue;
    if !do_continue {
        out_block.is_final = true;
    }

    match output_tx2.send((block.0, out_block, expected_read_count)) {
        Ok(()) => {}
        Err(_) => {
            // downstream has hung up
            return Ok(false);
        }
    }
    if !do_continue {
        assert!(
            stage.needs_serial(),
            "Non serial stages must not return do_continue = false"
        );
        return Ok(false);
    }
    Ok(true)
}
