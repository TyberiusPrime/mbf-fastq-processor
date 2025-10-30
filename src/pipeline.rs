use anyhow::{Context, Result};
use crossbeam::channel::bounded;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    config::{Config, StructuredInput},
    demultiplex::Demultiplexed,
    io::{self, parsers::ChainedParser, parsers::Parser},
    output::{open_output_files, output_block, output_html_report, output_json_report},
    transformations::{self, FinalizeReportResult, Step, Transformation},
};

#[allow(clippy::collapsible_if)]
fn parse_and_send(
    readers: Vec<io::InputFile>,
    raw_tx: &crossbeam::channel::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
    loop {
        let (out_block, was_final) = parser.parse()?;
        if !out_block.entries.is_empty() || !was_final {
            if raw_tx.send(out_block).is_err() {
                break;
            }
        }
        if was_final {
            break;
        }
    }
    Ok(())
}

fn parse_interleaved_and_send(
    readers: Vec<io::InputFile>,
    combiner_output_tx: &crossbeam::channel::Sender<(usize, io::FastQBlocksCombined)>,
    segment_count: usize,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
    let mut block_no = 1;
    loop {
        let (out_block, was_final) = parser.parse()?;
        if !out_block.entries.is_empty() || !was_final {
            let out_blocks = out_block.split_interleaved(segment_count);
            let out = (
                block_no,
                io::FastQBlocksCombined {
                    segments: out_blocks,
                    output_tags: None,
                    tags: None,
                },
            );
            block_no += 1;
            if combiner_output_tx.send(out).is_err() {
                break;
            }
        }

        if was_final {
            break;
        }
    }
    Ok(())
}

pub struct RunStage0 {
    report_html: bool,
    report_json: bool,
}

impl RunStage0 {
    pub fn new(parsed: &Config) -> Self {
        RunStage0 {
            report_html: parsed.output.as_ref().is_some_and(|o| o.report_html),
            report_json: parsed.output.as_ref().is_some_and(|o| o.report_json),
        }
    }

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
        let output_ix_separator = parsed
            .output
            .as_ref()
            .map_or_else(crate::config::default_ix_separator, |x| {
                x.ix_separator.clone()
            });

        let mut demultiplex_info = Demultiplexed::No;
        let mut demultiplex_start = 0;
        let input_info = transformations::InputInfo {
            segment_order: parsed.input.get_segment_order().clone(),
        };
        for (index, transform) in (parsed.transform).iter_mut().enumerate() {
            transform.configure_output_separator(&output_ix_separator);
            let new_demultiplex_info = transform
                .init(
                    &input_info,
                    &output_prefix,
                    output_directory,
                    &demultiplex_info,
                    allow_overwrite,
                )
                .context("Transform initialize failed")?;
            if let Some(new_demultiplex_info) = new_demultiplex_info {
                assert!(
                    matches!(demultiplex_info, Demultiplexed::No),
                    "Demultiplexed info already set, but new demultiplex info returned. More than one demultiplex transform not supported"
                );
                demultiplex_info = Demultiplexed::Yes(new_demultiplex_info);
                demultiplex_start = index;
            }
        }
        RunStage0::distribute_progress(&mut parsed.transform);
        Ok(RunStage1 {
            input_info,
            report_html: self.report_html,
            report_json: self.report_json,
            output_directory: output_directory.to_owned(),
            output_prefix,
            demultiplex_info,
            demultiplex_start,
            allow_overwrite,
        })
    }
    fn distribute_progress(transforms: &mut Vec<Transformation>) {
        let progress_output = transforms
            .iter()
            .filter_map(|x| {
                if let Transformation::Progress(inner) = x {
                    Some(inner.clone())
                } else {
                    None
                }
            })
            .next_back();
        if let Some(progress_output) = progress_output {
            for step in transforms {
                step.store_progress_output(&progress_output);
            }
        }
    }
}

pub struct RunStage1 {
    input_info: transformations::InputInfo,
    output_prefix: String,
    output_directory: PathBuf,
    demultiplex_info: Demultiplexed,
    demultiplex_start: usize,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,
}

impl RunStage1 {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    pub fn create_input_threads(self, parsed: &Config) -> Result<RunStage2> {
        let input_config = &parsed.input;
        let mut input_files =
            io::open_input_files(input_config).context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 2;
        let error_collector = Arc::new(Mutex::new(Vec::<String>::new()));
        let input_options = parsed.input.options.clone();

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
                //I need to receive the blocks (from all segment input threads)
                //and then, match them up into something that's the same length!
                let mut block_no = 1; // for the sorting later on.
                loop {
                    let mut blocks = Vec::new();
                    for receiver in &raw_rx_readers {
                        if let Ok(block) = receiver.recv() {
                            blocks.push(block);
                        } else if blocks.is_empty() {
                                //The first segment reader is done.
                                //that's the expected behaviour when we're running out of reads.
                                //now every other reader should also be returning an error.
                                //because otherwise the others have more remaining reads
                                for other_receiver in &raw_rx_readers[1..] {
                                    if let Ok(_block) = other_receiver.recv() {
                                        error_collector.lock().unwrap().push("Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts".to_string());
                                    }
                                }
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
                            tags: None,
                        },
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
            output_prefix: self.output_prefix,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_info: self.demultiplex_info.clone(),
            demultiplex_start: self.demultiplex_start,
            input_threads,
            combiner_thread,
            combiner_output_rx,
            error_collector,
            allow_overwrite: self.allow_overwrite,
        })
    }
}

pub struct RunStage2 {
    input_info: transformations::InputInfo,
    output_prefix: String,
    output_directory: PathBuf,
    report_html: bool,
    report_json: bool,
    demultiplex_info: Demultiplexed,
    demultiplex_start: usize,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    combiner_output_rx: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,

    error_collector: Arc<Mutex<Vec<String>>>,
    allow_overwrite: bool,
}
impl RunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_stage_threads(self, parsed: &mut Config) -> RunStage3 {
        let stages = &mut parsed.transform;
        let channel_size = 50;

        let mut channels: Vec<_> = (0..=stages.len())
            .map(|_| {
                let (tx, rx) = bounded::<(usize, io::FastQBlocksCombined)>(channel_size);
                (tx, rx)
            })
            .collect();
        channels[0].1 = self.combiner_output_rx;

        let thread_count = parsed.options.thread_count;
        let output_prefix = Arc::new(self.output_prefix);
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));
        let mut threads = Vec::new();

        for (stage_no, stage) in stages.iter_mut().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_thread_count = if needs_serial { 1 } else { thread_count };
            for _ in 0..local_thread_count {
                let mut stage = stage.move_inited();
                let input_rx2 = channels[stage_no].1.clone();
                let output_tx2 = channels[stage_no + 1].0.clone();
                let output_prefix = output_prefix.clone();
                let output_directory = self.output_directory.clone();
                let demultiplex_info2 = self.demultiplex_info.clone();
                let report_collector = report_collector.clone();
                let error_collector = self.error_collector.clone();
                if needs_serial {
                    //I suppose we could RC this, but it's only a few dozend bytes, typicallly.
                    //we used to have it on the SegmentIndex, but that's a lot of duplication
                    //and only used by a couple of transforms
                    let input_info: transformations::InputInfo = (self.input_info).clone();
                    threads.push(
                        thread::Builder::new()
                            .name(format!("Serial_stage {stage_no}"))
                            .spawn(move || {
                                //we need to ensure the blocks are passed on in order
                                let mut last_block_outputted = 0;
                                let mut buffer = Vec::new();
                                'outer: while let Ok((block_no, block)) = input_rx2.recv() {
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
                                            {
                                                let result = handle_stage(
                                                    to_output,
                                                    &input_info,
                                                    &output_tx2,
                                                    stage_no,
                                                    &mut stage,
                                                    &demultiplex_info2,
                                                    self.demultiplex_start,
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
                                        &input_info,
                                        &output_prefix,
                                        &output_directory,
                                        if stage_no >= self.demultiplex_start {
                                            &demultiplex_info2
                                        } else {
                                            &Demultiplexed::No
                                        },
                                    )
                                    .unwrap();
                                if let Some(report) = report {
                                    report_collector.lock().unwrap().push(report);
                                }
                            })
                            .unwrap(),
                    );
                } else {
                    let input_info: transformations::InputInfo = (self.input_info).clone();

                    threads.push(
                        thread::Builder::new()
                            .name(format!("Stage {stage_no}"))
                            .spawn(move || {
                                loop {
                                    match input_rx2.recv() {
                                        Ok(block) => {
                                            if let Err(e) = handle_stage(
                                                block,
                                                &input_info,
                                                &output_tx2,
                                                stage_no,
                                                &mut stage,
                                                &demultiplex_info2,
                                                self.demultiplex_start,
                                            ) {
                                                // For now, panic - will be improved in Phase 4
                                                error_collector.lock().unwrap().push(format!(
                                                    "Error in stage {stage_no} processing: {e:?}"
                                                ));
                                                return;
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
        RunStage3 {
            //input_info: self.input_info,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_info: self.demultiplex_info.clone(),
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            stage_threads: threads,
            stage_to_output_channel: channels[channels.len() - 1].1.clone(),
            report_collector,
            error_collector: self.error_collector,
            allow_overwrite: self.allow_overwrite,
        }
    }
}

pub struct RunStage3 {
    output_directory: PathBuf,
    demultiplex_info: Demultiplexed,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    stage_threads: Vec<thread::JoinHandle<()>>,
    stage_to_output_channel: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
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

        let mut output_files = open_output_files(
            parsed,
            &self.output_directory,
            &self.demultiplex_info,
            self.report_html,
            self.report_json,
            self.allow_overwrite,
        )?;

        let output_directory = self.output_directory;
        let demultiplex_info = self.demultiplex_info;
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
                        if let Err(e) = set_of_output_files.lock().unwrap().finish() {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error finishing output files: {e:?}"));
                            return;
                        }
                    }
                    //todo: wait for all reports to have been sent...
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
    block: (usize, io::FastQBlocksCombined),
    input_info: &transformations::InputInfo,
    output_tx2: &crossbeam::channel::Sender<(usize, io::FastQBlocksCombined)>,
    stage_no: usize,
    stage: &mut Transformation,
    demultiplex_info: &Demultiplexed,
    demultiplex_start: usize,
) -> anyhow::Result<bool> {
    let mut out_block = block.1;
    let mut do_continue = true;
    let stage_continue;

    (out_block, stage_continue) = stage.apply(
        out_block,
        input_info,
        block.0,
        if stage_no >= demultiplex_start {
            demultiplex_info
        } else {
            &Demultiplexed::No
        },
    )?;
    do_continue = do_continue && stage_continue;

    match output_tx2.send((block.0, out_block)) {
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
