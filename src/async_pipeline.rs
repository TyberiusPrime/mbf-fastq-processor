use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;

use crate::{
    config::{Config, StructuredInput},
    demultiplex::{DemultiplexBarcodes, DemultiplexInfo, OptDemultiplex},
    io::{
        self,
        parsers::{ChainedParser, Parser},
    },
    output::{open_output_files, output_block, output_html_report, output_json_report},
    transformations::{self, FinalizeReportResult, Step, Transformation},
};

// Async version of parse_and_send using tokio channels
async fn parse_and_send_async(
    readers: Vec<io::InputFile>,
    tx: mpsc::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
    error_collector: Arc<Mutex<Vec<String>>>,
) {
    let result = tokio::task::spawn_blocking(move || {
        let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
        let rt = tokio::runtime::Handle::current();
        loop {
            let (out_block, was_final) = match parser.parse() {
                Ok(result) => result,
                Err(e) => {
                    return Err(e);
                }
            };
            if !out_block.entries.is_empty() || !was_final {
                if rt.block_on(tx.send(out_block)).is_err() {
                    break;
                }
            }
            if was_final {
                break;
            }
        }
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error_collector
                .lock()
                .unwrap()
                .push(format!("Error in async parsing: {e:?}"));
        }
        Err(e) => {
            error_collector
                .lock()
                .unwrap()
                .push(format!("Error in async parsing task: {e:?}"));
        }
    }
}

// Async version of parse_interleaved_and_send
async fn parse_interleaved_and_send_async(
    readers: Vec<io::InputFile>,
    tx: mpsc::Sender<(usize, io::FastQBlocksCombined)>,
    segment_count: usize,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
    error_collector: Arc<Mutex<Vec<String>>>,
) {
    let result = tokio::task::spawn_blocking(move || {
        let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
        let mut block_no = 1;
        let rt = tokio::runtime::Handle::current();
        loop {
            let (out_block, was_final) = match parser.parse() {
                Ok(result) => result,
                Err(e) => {
                    return Err(e);
                }
            };
            if !out_block.entries.is_empty() || !was_final {
                let out_blocks = out_block.split_interleaved(segment_count);
                let out = (
                    block_no,
                    io::FastQBlocksCombined {
                        segments: out_blocks,
                        output_tags: None,
                        tags: Default::default(),
                        is_final: false,
                    },
                );
                block_no += 1;
                if rt.block_on(tx.send(out)).is_err() {
                    break;
                }
            }

            if was_final {
                // Send final empty block
                let final_block = io::FastQBlocksCombined {
                    segments: vec![io::FastQBlock::empty()],
                    output_tags: None,
                    tags: Default::default(),
                    is_final: true,
                };
                let _ = rt.block_on(tx.send((block_no, final_block)));
                break;
            }
        }
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error_collector
                .lock()
                .unwrap()
                .push(format!("Error in async interleaved parsing: {e:?}"));
        }
        Err(e) => {
            error_collector
                .lock()
                .unwrap()
                .push(format!("Error in async interleaved parsing task: {e:?}"));
        }
    }
}

// Similar structure to the sync version
pub struct AsyncRunStage0 {
    report_html: bool,
    report_json: bool,
}

impl AsyncRunStage0 {
    pub fn new(parsed: &Config) -> Self {
        AsyncRunStage0 {
            report_html: parsed.output.as_ref().is_some_and(|o| o.report_html),
            report_json: parsed.output.as_ref().is_some_and(|o| o.report_json),
        }
    }

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

        Ok(AsyncRunStage1 {
            input_info,
            report_html: self.report_html,
            report_json: self.report_json,
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
    allow_overwrite: bool,
}

impl AsyncRunStage1 {
    pub async fn create_input_tasks(self, parsed: &Config) -> Result<AsyncRunStage2> {
        let input_config = &parsed.input;
        let mut input_files =
            io::open_input_files(input_config).context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 2;
        let error_collector = Arc::new(Mutex::new(Vec::<String>::new()));
        let timing_collector = Arc::new(Mutex::new(Vec::<crate::timing::StepTiming>::new()));
        let input_options = parsed.input.options.clone();

        let combiner_output_rx = match parsed.input.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. } => {
                let error_collector = error_collector.clone();
                let segment_order_len = segment_order.len();
                let (combiner_output_tx, combiner_output_rx) =
                    mpsc::channel::<(usize, io::FastQBlocksCombined)>(channel_size);
                let options = input_options.clone();

                tokio::spawn(parse_interleaved_and_send_async(
                    input_files.segments.pop().unwrap(),
                    combiner_output_tx,
                    segment_order_len,
                    buffer_size,
                    block_size,
                    options,
                    error_collector,
                ));

                combiner_output_rx
            }
            StructuredInput::Segmented { segment_order, .. } => {
                // Spawn one reading task per input segment
                let mut raw_rx_readers = Vec::new();
                for (_segment_name, this_segments_input_files) in
                    segment_order.iter().zip(input_files.segments.into_iter())
                {
                    let error_collector = error_collector.clone();
                    let options = input_options.clone();
                    let (raw_tx_read, raw_rx_read) = mpsc::channel(channel_size);

                    tokio::spawn(async move {
                        parse_and_send_async(
                            this_segments_input_files,
                            raw_tx_read,
                            buffer_size,
                            block_size,
                            options,
                            error_collector.clone(),
                        )
                        .await;
                    });

                    raw_rx_readers.push(raw_rx_read);
                }
                let (combiner_output_tx, combiner_output_rx) =
                    mpsc::channel::<(usize, io::FastQBlocksCombined)>(channel_size);

                {
                    let error_collector = error_collector.clone();
                    tokio::spawn(async move {
                        let mut block_no = 1;
                        loop {
                            let mut blocks = Vec::new();
                            for receiver in &mut raw_rx_readers.iter_mut() {
                                if let Some(block) = receiver.recv().await {
                                    blocks.push(block);
                                } else if blocks.is_empty() {
                                    // First segment reader is done
                                    for other_receiver in &mut raw_rx_readers[1..].iter_mut() {
                                        if other_receiver.recv().await.is_some() {
                                            error_collector.lock().unwrap().push(
                                                "Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                    // Send final empty block
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
                                    let _ = combiner_output_tx.send((block_no, final_block)).await;
                                    return;
                                } else {
                                    error_collector.lock().unwrap().push(
                                        "Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts"
                                            .to_string(),
                                    );
                                    return;
                                }
                            }
                            // Make sure they all have the same length
                            let first_len = blocks[0].len();
                            if !blocks.iter().all(|b| b.len() == first_len) {
                                error_collector.lock().unwrap().push(
                                    "Unequal block sizes in input segments. This suggests your fastqs have different numbers of reads."
                                        .to_string(),
                                );
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
                            if combiner_output_tx.send(out).await.is_err() {
                                break;
                            }
                        }
                    });
                }

                combiner_output_rx
            }
        };

        Ok(AsyncRunStage2 {
            input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_infos: self.demultiplex_infos,
            combiner_output_rx,
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
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,

    combiner_output_rx: mpsc::Receiver<(usize, io::FastQBlocksCombined)>,

    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    allow_overwrite: bool,
}

impl AsyncRunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_stage_tasks(self, parsed: &mut Config) -> AsyncRunStage3 {
        let stages = std::mem::replace(&mut parsed.transform, Vec::new());
        let channel_size = 50;

        let mut channels: Vec<_> = (0..=stages.len())
            .map(|_| {
                let (tx, rx) = mpsc::channel::<(usize, io::FastQBlocksCombined)>(channel_size);
                (tx, Some(rx))
            })
            .collect();

        // Set the first channel's receiver to be the combiner output
        channels[0].1 = Some(self.combiner_output_rx);

        let thread_count = parsed.options.thread_count;
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));
        let mut task_handles = Vec::new();

        for (stage_no, mut stage) in stages.into_iter().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_task_count = if needs_serial { 1 } else { thread_count };

            if needs_serial {
                // Serial stage - spawn single task that enforces ordering
                let mut input_rx = channels[stage_no].1.take().unwrap();
                let output_tx2 = channels[stage_no + 1].0.clone();
                let error_collector = self.error_collector.clone();
                let timing_collector = self.timing_collector.clone();
                let input_info = self.input_info.clone();
                let step_type = stage.to_string();
                let mut demultiplex_info_for_stage = OptDemultiplex::No;
                for (idx, demultiplex_info) in self.demultiplex_infos.iter().rev() {
                    if *idx <= stage_no {
                        demultiplex_info_for_stage = demultiplex_info.clone();
                        break;
                    }
                }

                let report_collector = report_collector.clone();
                task_handles.push(tokio::spawn(async move {
                    let mut last_block_outputted = 0;
                    let mut buffer = Vec::new();
                    'outer: while let Some((block_no, block)) = input_rx.recv().await {
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
                                let result = handle_stage_async(
                                    to_output,
                                    &input_info,
                                    &output_tx2,
                                    &mut stage,
                                    &demultiplex_info_for_stage,
                                    &timing_collector,
                                    stage_no,
                                    &step_type,
                                )
                                .await;
                                match result {
                                    Ok(do_continue) => {
                                        if !do_continue && transmits_premature_termination {
                                            break 'outer;
                                        }
                                    }
                                    Err(e) => {
                                        error_collector.lock().unwrap().push(format!(
                                            "Error in stage {stage_no} processing: {e:?}"
                                        ));
                                        break 'outer;
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    let report = stage.finalize(&demultiplex_info_for_stage);
                    match report {
                        Ok(Some(report)) => report_collector.lock().unwrap().push(report),
                        Ok(None) => {}
                        Err(err) => {
                            error_collector.lock().unwrap().push(format!(
                                "Error in stage {stage_no} finalization: {err:?}"
                            ));
                        }
                    }
                }));
            } else {
                // Parallel stage - wrap receiver in Arc<Mutex> for sharing
                let input_rx_shared = Arc::new(tokio::sync::Mutex::new(
                    channels[stage_no].1.take().unwrap()
                ));
                let output_tx2 = channels[stage_no + 1].0.clone();
                let step_type = stage.to_string();

                for _ in 0..local_task_count {
                    let input_info = self.input_info.clone();
                    let input_rx = input_rx_shared.clone();
                    let output_tx = output_tx2.clone();
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

                    task_handles.push(tokio::spawn(async move {
                        loop {
                            let block = {
                                let mut rx = input_rx.lock().await;
                                rx.recv().await
                            };

                            match block {
                                Some(block) => {
                                    if let Err(e) = handle_stage_async(
                                        block,
                                        &input_info,
                                        &output_tx,
                                        &mut stage,
                                        &demultiplex_info_for_stage,
                                        &timing_collector,
                                        stage_no,
                                        &step_type,
                                    )
                                    .await
                                    {
                                        error_collector.lock().unwrap().push(format!(
                                            "Error in stage {stage_no} processing: {e:?}"
                                        ));
                                        return;
                                    }
                                }
                                None => return,
                            }
                        }
                    }));
                }
            }
        }

        let last_channel_idx = channels.len() - 1;
        let stage_to_output_channel = channels[last_channel_idx].1.take().unwrap();

        AsyncRunStage3 {
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_infos: self.demultiplex_infos,
            stage_tasks: task_handles,
            stage_to_output_channel,
            report_collector,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
            allow_overwrite: self.allow_overwrite,
        }
    }
}

pub struct AsyncRunStage3 {
    output_directory: PathBuf,
    demultiplex_infos: Vec<(usize, OptDemultiplex)>,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,

    stage_tasks: Vec<tokio::task::JoinHandle<()>>,
    stage_to_output_channel: mpsc::Receiver<(usize, io::FastQBlocksCombined)>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
}

impl AsyncRunStage3 {
    #[allow(clippy::too_many_lines)]
    pub fn create_output_task(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<AsyncRunStage4> {
        let mut input_channel = self.stage_to_output_channel;
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

        let error_collector = self.error_collector.clone();
        let stage_tasks = self.stage_tasks;

        let output_task = tokio::spawn(async move {
            let mut last_block_outputted = 0;
            let mut buffer = Vec::new();
            while let Some((block_no, block)) = input_channel.recv().await {
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
                                .push(format!("Error in output task: {e:?}"));
                            return;
                        }
                    } else {
                        break;
                    }
                }
            }

            // Join all stage tasks
            for task in stage_tasks {
                if let Err(e) = task.await {
                    error_collector
                        .lock()
                        .unwrap()
                        .push(format!("Error joining stage task: {e:?}"));
                }
            }

            // Finalize output files
            for set_of_output_files in &mut output_files.output_segments {
                if let Err(e) = set_of_output_files.1.lock().unwrap().finish() {
                    error_collector
                        .lock()
                        .unwrap()
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
        });

        Ok(AsyncRunStage4 {
            output_task,
            error_collector: self.error_collector,
            timing_collector: self.timing_collector,
        })
    }
}

pub struct AsyncRunStage4 {
    error_collector: Arc<Mutex<Vec<String>>>,
    timing_collector: Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    output_task: tokio::task::JoinHandle<()>,
}

impl AsyncRunStage4 {
    pub async fn join_tasks(self) -> AsyncRunStage5 {
        let mut errors = Vec::new();

        if let Err(e) = self.output_task.await {
            errors.push(format!("Failure in output task: {e:?}"));
        }

        // Extract any accumulated errors
        errors.extend(self.error_collector.lock().unwrap().drain(..));

        // Extract timing data
        let timings = self.timing_collector.lock().unwrap().clone();

        AsyncRunStage5 { errors, timings }
    }
}

pub struct AsyncRunStage5 {
    pub errors: Vec<String>,
    pub timings: Vec<crate::timing::StepTiming>,
}

async fn handle_stage_async(
    block: (usize, io::FastQBlocksCombined),
    input_info: &transformations::InputInfo,
    output_tx: &mpsc::Sender<(usize, io::FastQBlocksCombined)>,
    stage: &mut Transformation,
    demultiplex_info: &OptDemultiplex,
    timing_collector: &Arc<Mutex<Vec<crate::timing::StepTiming>>>,
    step_no: usize,
    step_type: &str,
) -> anyhow::Result<bool> {
    let mut out_block = block.1;
    let mut do_continue = true;
    let stage_continue;

    // Record timing for this step (both wall and CPU time)
    let (wall_start, cpu_start) = crate::timing::StepTiming::start();
    (out_block, stage_continue) = stage.apply(out_block, input_info, block.0, demultiplex_info)?;
    let timing =
        crate::timing::StepTiming::from_start(step_no, step_type.to_string(), wall_start, cpu_start);

    // Push timing data to collector
    timing_collector.lock().unwrap().push(timing);

    do_continue = do_continue && stage_continue;

    if output_tx.send((block.0, out_block)).await.is_err() {
        // downstream has hung up
        return Ok(false);
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
