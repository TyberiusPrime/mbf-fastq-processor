use anyhow::{Context, Result, bail};
use crossbeam::channel::{bounded, Sender};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    config::{Config, StructuredInput},
    demultiplex::{DemultiplexBarcodes, DemultiplexInfo, OptDemultiplex},
    io::{self},
    output::{open_output_files, output_block, output_html_report, output_json_report},
    pipeline::{parse_and_send, parse_interleaved_and_send},
    transformations::{self, FinalizeReportResult, Step, Transformation},
};

// Work item in the single shared queue
struct QueuedBlock {
    stage_no: usize,
    block_no: usize,
    block: io::FastQBlocksCombined,
}

pub struct SimpleCoordRunStage0 {
    report_html: bool,
    report_json: bool,
}

impl SimpleCoordRunStage0 {
    pub fn new(parsed: &Config) -> Self {
        SimpleCoordRunStage0 {
            report_html: parsed.output.as_ref().is_some_and(|o| o.report_html),
            report_json: parsed.output.as_ref().is_some_and(|o| o.report_json),
        }
    }

    pub fn configure_demultiplex_and_init_stages(
        self,
        parsed: &mut Config,
        output_directory: &Path,
        allow_overwrite: bool,
    ) -> Result<SimpleCoordRunStage1> {
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
        };
        let mut demultiplex_infos: Vec<(usize, OptDemultiplex)> = Vec::new();

        // Initialize progress_output first
        let progress_output = {
            let mut res = None;
            for step in parsed.transform.iter_mut() {
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

        // Combine demultiplex stages
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
                        .map(|x| &x.1)
                        .unwrap_or(&OptDemultiplex::No);
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
                if let Some(new_demultiplex_barcodes) = new_demultiplex_barcodes {
                    let barcode_count = new_demultiplex_barcodes.barcode_to_name.len()
                        + if new_demultiplex_barcodes.include_no_barcode {
                            1
                        } else {
                            0
                        };
                    let bits_needed = ((barcode_count as f64).log2().ceil()) as u8;
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
                    for (_ii, name) in unique_names.into_iter().enumerate() {
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
                        ))
                    } else {
                        let mut next = BTreeMap::new();
                        {
                            let last_demultiplex_info = demultiplex_infos
                                .iter()
                                .last()
                                .map(|x| &x.1)
                                .unwrap_or(&OptDemultiplex::No);

                            for (old_tag, old_name) in
                                last_demultiplex_info.unwrap().tag_to_name.iter()
                            {
                                for (new_tag, new_name) in tag_to_name.iter() {
                                    let combined_tag = old_tag | new_tag;
                                    let out_name: Option<String> = {
                                        if let Some(old_name) = old_name {
                                            if let Some(new_name) = new_name {
                                                Some(format!(
                                                    "{}{}{}",
                                                    old_name, &output_ix_separator, new_name
                                                ))
                                            } else {
                                                None
                                            }
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

        Ok(SimpleCoordRunStage1 {
            input_info,
            report_html: self.report_html,
            report_json: self.report_json,
            output_directory: output_directory.to_owned(),
            demultiplex_infos,
            allow_overwrite,
        })
    }
}

pub struct SimpleCoordRunStage1 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,
}

impl SimpleCoordRunStage1 {
    #[allow(clippy::too_many_lines)]
    pub fn create_input_threads(self, parsed: &Config) -> Result<SimpleCoordRunStage2> {
        let input_config = &parsed.input;
        let mut input_files =
            io::open_input_files(input_config).context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 2;
        let error_collector = Arc::new(Mutex::new(Vec::<String>::new()));
        let timing_collector = Arc::new(Mutex::new(Vec::<crate::timing::StepTiming>::new()));
        let input_options = parsed.input.options.clone();

        // Reuse the input thread creation from the sync pipeline
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
                    bounded::<(usize, io::FastQBlocksCombined)>(channel_size);
                let options = input_options.clone();
                let combiner_thread = thread::Builder::new()
                    .name("InterleavedReader".into())
                    .spawn(move || {
                        if let Err(e) = parse_interleaved_and_send(
                            input_files.segments.pop().unwrap(),
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

                (input_threads, combiner_thread, combiner_output_rx)
            }
            StructuredInput::Segmented { segment_order, .. } => {
                let mut threads = Vec::new();
                let mut raw_rx_readers = Vec::new();
                for (segment_name, this_segments_input_files) in
                    segment_order.iter().zip(input_files.segments.into_iter())
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
                    bounded::<(usize, io::FastQBlocksCombined)>(channel_size);

                {
                    let error_collector = error_collector.clone();
                    let combiner = thread::Builder::new()
            .name("Combiner".into())
            .spawn(move || {
                let mut block_no = 1;
                loop {
                    let mut blocks = Vec::new();
                    for receiver in &raw_rx_readers {
                        if let Ok(block) = receiver.recv() {
                            blocks.push(block);
                        } else if blocks.is_empty() {
                                for other_receiver in &raw_rx_readers[1..] {
                                    if other_receiver.recv().is_ok() {
                                        error_collector.lock().unwrap().push("Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts".to_string());
                                    }
                                }
                                let empty_segments: Vec::<io::FastQBlock> =
                                    raw_rx_readers.iter().map(|_| io::FastQBlock::empty()).collect();
                                let final_block = io::FastQBlocksCombined {
                                    segments: empty_segments,
                                    output_tags: None,
                                    tags: Default::default(),
                                    is_final: true,
                                };
                                let _ = combiner_output_tx.send((block_no, final_block));
                                return;
                        } else {
                            error_collector.lock().unwrap().push("Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts".to_string());

                                return;
                            }
                    }
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
                    );
                    block_no += 1;
                    if combiner_output_tx.send(out).is_err() {
                        break;
                    }
                }
            })
            .unwrap();
                    (threads, combiner, combiner_output_rx)
                }
            }
        };

        Ok(SimpleCoordRunStage2 {
            input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
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

pub struct SimpleCoordRunStage2 {
    input_info: transformations::InputInfo,
    output_directory: PathBuf,
    report_html: bool,
    report_json: bool,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    combiner_output_rx: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,

    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    allow_overwrite: bool,
}

impl SimpleCoordRunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_simple_coordinator(self, parsed: &mut Config) -> SimpleCoordRunStage3 {
        let stages = std::mem::replace(&mut parsed.transform, Vec::new());
        let num_stages = stages.len();

        let thread_count = parsed.options.thread_count;
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));

        // Single shared queue for all stages
        let (work_tx, work_rx) = bounded::<QueuedBlock>(100);

        // Output channel
        let (output_tx, output_rx) = bounded::<(usize, io::FastQBlocksCombined)>(50);

        // Build stage metadata
        let stage_metadata: Vec<_> = stages
            .iter()
            .enumerate()
            .map(|(stage_no, stage)| {
                let mut demultiplex_info = OptDemultiplex::No;
                for (idx, dm_info) in self.demultiplex_infos.iter().rev() {
                    if *idx <= stage_no {
                        demultiplex_info = dm_info.clone();
                        break;
                    }
                }
                (stage.needs_serial(), stage.to_string(), demultiplex_info)
            })
            .collect();

        // Channel for serial stage processing
        let (serial_tx, serial_rx) = bounded::<QueuedBlock>(50);

        // Spawn worker threads - they only process parallel stages
        let mut worker_threads = Vec::new();
        for worker_id in 0..thread_count {
            let work_rx = work_rx.clone();
            let work_tx = work_tx.clone();
            let serial_tx = serial_tx.clone();
            let output_tx = output_tx.clone();
            let stages = stages.clone();
            let input_info = self.input_info.clone();
            let stage_metadata = stage_metadata.clone();
            let error_collector = self.error_collector.clone();
            let timing_collector = self.timing_collector.clone();

            let worker = thread::Builder::new()
                .name(format!("SimpleWorker_{worker_id}"))
                .spawn(move || {
                    while let Ok(mut item) = work_rx.recv() {
                        let (needs_serial, step_type, demultiplex_info) = &stage_metadata[item.stage_no];

                        if *needs_serial {
                            // Send to coordinator for serial processing
                            if serial_tx.send(item).is_err() {
                                return;
                            }
                        } else {
                            // Process parallel stage
                            let mut stage = stages[item.stage_no].clone();

                            let (wall_start, cpu_start) = crate::timing::StepTiming::start();
                            let result = stage.apply(
                                item.block,
                                &input_info,
                                item.block_no,
                                demultiplex_info,
                            );
                            let timing = crate::timing::StepTiming::from_start(
                                item.stage_no,
                                step_type.clone(),
                                wall_start,
                                cpu_start,
                            );
                            timing_collector.lock().unwrap().push(timing);

                            match result {
                                Ok((out_block, _continue)) => {
                                    // Move to next stage
                                    item.block = out_block;
                                    item.stage_no += 1;

                                    if item.stage_no >= num_stages {
                                        // Done - send to output
                                        if output_tx.send((item.block_no, item.block)).is_err() {
                                            return;
                                        }
                                    } else {
                                        // Put back in queue for next stage
                                        if work_tx.send(item).is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error_collector.lock().unwrap().push(format!(
                                        "Error in stage {} processing: {e:?}",
                                        item.stage_no
                                    ));
                                    return;
                                }
                            }
                        }
                    }
                })
                .unwrap();
            worker_threads.push(worker);
        }

        // Coordinator thread: handles input feeding and serial stage processing
        let coordinator_thread = {
            let error_collector = self.error_collector.clone();
            let input_rx = self.combiner_output_rx;
            let report_collector = report_collector.clone();
            let timing_collector = self.timing_collector.clone();
            let input_info = self.input_info.clone();

            thread::Builder::new()
                .name("SimpleCoordinator".into())
                .spawn(move || {
                    use crossbeam::channel::select;

                    // Track ordering for serial stages
                    let mut ordering_buffers: BTreeMap<usize, Vec<QueuedBlock>> = BTreeMap::new();
                    let mut last_block_outputted: BTreeMap<usize, usize> = BTreeMap::new();

                    for (stage_no, _stage) in stages.iter().enumerate() {
                        if stage_metadata[stage_no].0 {
                            // needs_serial
                            ordering_buffers.insert(stage_no, Vec::new());
                            last_block_outputted.insert(stage_no, 0);
                        }
                    }

                    let mut input_done = false;

                    // Process both input and serial requests
                    loop {
                        select! {
                            recv(input_rx) -> msg => {
                                match msg {
                                    Ok((block_no, block)) => {
                                        let item = QueuedBlock {
                                            stage_no: 0,
                                            block_no,
                                            block,
                                        };
                                        if work_tx.send(item).is_err() {
                                            return;
                                        }
                                    }
                                    Err(_) => {
                                        input_done = true;
                                    }
                                }
                            }
                            recv(serial_rx) -> msg => {
                                match msg {
                                    Ok(item) => {
                                        let stage_no = item.stage_no;
                                        let (needs_serial, step_type, demultiplex_info) = &stage_metadata[stage_no];

                                        if !needs_serial {
                                            error_collector.lock().unwrap().push(format!(
                                                "Non-serial block sent to serial channel for stage {stage_no}"
                                            ));
                                            return;
                                        }

                                        // Buffer for ordering
                                        let buffer = ordering_buffers.get_mut(&stage_no).unwrap();
                                        buffer.push(item);

                                        // Try to process ordered blocks
                                        loop {
                                            let mut send_idx = None;
                                            let last_out = last_block_outputted.get_mut(&stage_no).unwrap();

                                            for (ii, queued_item) in buffer.iter().enumerate() {
                                                if queued_item.block_no - 1 == *last_out {
                                                    *last_out += 1;
                                                    send_idx = Some(ii);
                                                    break;
                                                }
                                            }

                                            if let Some(idx) = send_idx {
                                                let mut to_process = buffer.remove(idx);
                                                let mut stage = stages[stage_no].clone();

                                                // Process serial stage
                                                let (wall_start, cpu_start) = crate::timing::StepTiming::start();
                                                let result = stage.apply(
                                                    to_process.block,
                                                    &input_info,
                                                    to_process.block_no,
                                                    demultiplex_info,
                                                );
                                                let timing = crate::timing::StepTiming::from_start(
                                                    stage_no,
                                                    step_type.clone(),
                                                    wall_start,
                                                    cpu_start,
                                                );
                                                timing_collector.lock().unwrap().push(timing);

                                                match result {
                                                    Ok((out_block, _continue)) => {
                                                        to_process.block = out_block;
                                                        to_process.stage_no += 1;

                                                        if to_process.stage_no >= num_stages {
                                                            // Done - send to output
                                                            if output_tx.send((to_process.block_no, to_process.block)).is_err() {
                                                                return;
                                                            }
                                                        } else {
                                                            // Put back in work queue for next stage
                                                            if work_tx.send(to_process).is_err() {
                                                                return;
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error_collector.lock().unwrap().push(format!(
                                                            "Error in serial stage {stage_no} processing: {e:?}"
                                                        ));
                                                        return;
                                                    }
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        if input_done {
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if input_done && serial_rx.is_empty() {
                            break;
                        }
                    }

                    // Input done - close work queue
                    drop(work_tx);

                    // Finalize serial stages
                    for (stage_no, mut stage) in stages.into_iter().enumerate() {
                        if stage_metadata[stage_no].0 {
                            // needs_serial
                            let (_needs_serial, _step_type, demultiplex_info) =
                                &stage_metadata[stage_no];
                            let report = stage.finalize(demultiplex_info);
                            match report {
                                Ok(Some(report)) => report_collector.lock().unwrap().push(report),
                                Ok(None) => {}
                                Err(err) => {
                                    error_collector.lock().unwrap().push(format!(
                                        "Error in stage {stage_no} finalization: {err:?}"
                                    ));
                                }
                            }
                        }
                    }
                })
                .unwrap()
        };

        SimpleCoordRunStage3 {
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_infos: self.demultiplex_infos,
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            coordinator_thread,
            worker_threads,
            output_rx,
            report_collector,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
            allow_overwrite: self.allow_overwrite,
        }
    }
}

pub struct SimpleCoordRunStage3 {
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    coordinator_thread: thread::JoinHandle<()>,
    worker_threads: Vec<thread::JoinHandle<()>>,
    output_rx: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
}

impl SimpleCoordRunStage3 {
    #[allow(clippy::too_many_lines)]
    pub fn create_output_thread(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<SimpleCoordRunStage4> {
        let input_channel = self.output_rx;
        let output_buffer_size = parsed.options.output_buffer_size;
        let cloned_input_config = parsed.input.clone();

        let demultiplex_info = self
            .demultiplex_infos
            .iter()
            .last()
            .map(|x| x.1.clone())
            .unwrap_or_else(|| OptDemultiplex::No);

        let mut output_files = open_output_files(
            parsed,
            &self.output_directory,
            &demultiplex_info,
            self.report_html,
            self.report_json,
            self.allow_overwrite,
        )?;

        let output_directory = self.output_directory;
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
                    while let Ok((block_no, block)) = input_channel.recv() {
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
                                output_files.output_reports.json.as_mut(),
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
                })
                .unwrap()
        };

        Ok(SimpleCoordRunStage4 {
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            coordinator_thread: self.coordinator_thread,
            worker_threads: self.worker_threads,
            output_thread: output,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
        })
    }
}

pub struct SimpleCoordRunStage4 {
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    coordinator_thread: thread::JoinHandle<()>,
    worker_threads: Vec<thread::JoinHandle<()>>,
    output_thread: thread::JoinHandle<()>,
}

impl SimpleCoordRunStage4 {
    pub fn join_threads(self) -> SimpleCoordRunStage5 {
        let mut errors = Vec::new();

        // Join coordinator first
        if let Err(e) = self.coordinator_thread.join() {
            errors.push(format!("Failure in coordinator thread: {e:?}"));
        }

        // Join workers
        for (i, worker) in self.worker_threads.into_iter().enumerate() {
            if let Err(e) = worker.join() {
                errors.push(format!("Failure in worker {i}: {e:?}"));
            }
        }

        // Join output
        if let Err(e) = self.output_thread.join() {
            errors.push(format!("Failure in output thread: {e:?}"));
        }

        // Join combiner
        if let Err(e) = self.combiner_thread.join() {
            errors.push(format!("Failure in combiner thread: {e:?}"));
        }

        // Join input threads
        for (i, input) in self.input_threads.into_iter().enumerate() {
            if let Err(e) = input.join() {
                errors.push(format!("Failure in input thread {i}: {e:?}"));
            }
        }

        // Extract accumulated errors
        errors.extend(self.error_collector.lock().unwrap().drain(..));

        // Extract timing data
        let timings = self.timing_collector.lock().unwrap().clone();

        SimpleCoordRunStage5 { errors, timings }
    }
}

pub struct SimpleCoordRunStage5 {
    pub errors: Vec<String>,
    pub timings: Vec<crate::timing::StepTiming>,
}
