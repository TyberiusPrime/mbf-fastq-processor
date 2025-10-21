use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::sync::{Mutex as AsyncMutex, mpsc};
use tokio::task::JoinHandle;

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
    raw_tx: mpsc::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
    input_options: crate::config::InputOptions,
) -> Result<()> {
    let mut parser = ChainedParser::new(readers, block_size, buffer_size, input_options);
    loop {
        let (out_block, was_final) = parser.parse()?;
        if !out_block.entries.is_empty() || !was_final {
            if raw_tx.blocking_send(out_block).is_err() {
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
    combiner_output_tx: mpsc::Sender<(usize, io::FastQBlocksCombined)>,
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
            if combiner_output_tx.blocking_send(out).is_err() {
                break;
            }
        }

        if was_final {
            break;
        }
    }
    Ok(())
}

type Block = (usize, io::FastQBlocksCombined);
type BlockSender = mpsc::Sender<Block>;
type BlockReceiver = mpsc::Receiver<Block>;
type RawReceiver = mpsc::Receiver<io::FastQBlock>;

async fn combine_segment_blocks(
    mut raw_receivers: Vec<RawReceiver>,
    combiner_output_tx: BlockSender,
    error_collector: Arc<Mutex<Vec<String>>>,
) {
    let mut block_no = 1usize;
    loop {
        let mut blocks = Vec::with_capacity(raw_receivers.len());
        for (idx, receiver) in raw_receivers.iter_mut().enumerate() {
            match receiver.recv().await {
                Some(block) => blocks.push(block),
                None => {
                    if blocks.is_empty() {
                        if idx == 0 {
                            // First segment ended; ensure others are finished as well
                            for other_receiver in raw_receivers.iter_mut().skip(1) {
                                if other_receiver.recv().await.is_some() {
                                    error_collector.lock().unwrap().push(
                                        "Unequal number of reads in the segment inputs (first < later). Check your fastqs for identical read counts".to_string(),
                                    );
                                    return;
                                }
                            }
                            return;
                        }
                        error_collector.lock().unwrap().push(
                            "Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts".to_string(),
                        );
                        return;
                    }
                    error_collector.lock().unwrap().push(
                        "Unequal number of reads in the segment inputs (first > later). Check your fastqs for identical read counts".to_string(),
                    );
                    return;
                }
            }
        }

        if blocks.is_empty() {
            break;
        }
        let first_len = blocks[0].len();
        if !blocks.iter().all(|b| b.len() == first_len) {
            error_collector.lock().unwrap().push(
                "Unequal block sizes in input segments. This suggests your fastqs have different numbers of reads.".to_string(),
            );
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
        if combiner_output_tx.send(out).await.is_err() {
            break;
        }
    }
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

        let (input_threads, combiner_thread, combiner_output_rx) =
            match parsed.input.structured.as_ref().unwrap() {
                StructuredInput::Interleaved { segment_order, .. } => {
                    let error_collector = error_collector.clone();
                    let segment_order_len = segment_order.len();
                    let (combiner_output_tx, combiner_output_rx) = mpsc::channel(channel_size);
                    let options = input_options.clone();
                    let readers = input_files.segments.pop().unwrap();
                    let combiner_thread = tokio::task::spawn_blocking(move || {
                        if let Err(e) = parse_interleaved_and_send(
                            readers,
                            combiner_output_tx,
                            segment_order_len,
                            buffer_size,
                            block_size,
                            options,
                        ) {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error in interleaved parsing task: {e:?}"));
                        }
                    });
                    (Vec::new(), combiner_thread, combiner_output_rx)
                }
                StructuredInput::Segmented { segment_order, .. } => {
                    // we spawn one reading task per input segment for reading & decompressing
                    // and another task that collects the blocks into combined blocks
                    let mut threads = Vec::new();
                    let mut raw_rx_readers = Vec::new();
                    for (segment_name, this_segments_input_files) in
                        segment_order.iter().zip(input_files.segments.into_iter())
                    {
                        let segment_name = segment_name.clone();
                        let error_collector = error_collector.clone();
                        let options = input_options.clone();
                        let (raw_tx_read, raw_rx_read) = mpsc::channel(channel_size);
                        let read_thread = tokio::task::spawn_blocking(move || {
                            if let Err(e) = parse_and_send(
                                this_segments_input_files,
                                raw_tx_read,
                                buffer_size,
                                block_size,
                                options,
                            ) {
                                error_collector.lock().unwrap().push(format!(
                                    "Error in reading task for segment {segment_name}: {e:?}"
                                ));
                            }
                        });
                        threads.push(read_thread);
                        raw_rx_readers.push(raw_rx_read);
                    }
                    let (combiner_output_tx, combiner_output_rx) = mpsc::channel(channel_size);
                    let error_collector = error_collector.clone();
                    let combiner_thread = tokio::spawn(async move {
                        combine_segment_blocks(raw_rx_readers, combiner_output_tx, error_collector)
                            .await;
                    });
                    (threads, combiner_thread, combiner_output_rx)
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

    input_threads: Vec<JoinHandle<()>>,
    combiner_thread: JoinHandle<()>,
    combiner_output_rx: BlockReceiver,

    error_collector: Arc<Mutex<Vec<String>>>,
    allow_overwrite: bool,
}
impl RunStage2 {
    #[allow(clippy::too_many_lines)]
    pub fn create_stage_threads(self, parsed: &Config) -> RunStage3 {
        let stages = &parsed.transform;
        let channel_size = 50;

        let mut receivers: Vec<Option<BlockReceiver>> = Vec::with_capacity(stages.len() + 1);
        receivers.push(Some(self.combiner_output_rx));

        let mut senders: Vec<BlockSender> = Vec::with_capacity(stages.len());
        for _ in 0..stages.len() {
            let (tx, rx) = mpsc::channel(channel_size);
            senders.push(tx);
            receivers.push(Some(rx));
        }

        let thread_count = parsed.options.thread_count;
        let output_prefix = Arc::new(self.output_prefix);
        let report_collector = Arc::new(Mutex::new(Vec::<FinalizeReportResult>::new()));
        let mut stage_tasks = Vec::new();

        for (stage_no, stage) in stages.iter().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_thread_count = if needs_serial { 1 } else { thread_count };
            let input_rx = receivers[stage_no]
                .take()
                .expect("stage receiver must be available");
            let output_tx = senders[stage_no].clone();

            if needs_serial {
                let mut stage = stage.clone();
                let input_info = self.input_info.clone();
                let output_prefix = output_prefix.clone();
                let output_directory = self.output_directory.clone();
                let demultiplex_info = self.demultiplex_info.clone();
                let report_collector = report_collector.clone();
                let error_collector = self.error_collector.clone();
                let demultiplex_start = self.demultiplex_start;
                stage_tasks.push(tokio::spawn(async move {
                    let mut last_block_outputted = 0usize;
                    let mut buffer: Vec<Block> = Vec::new();
                    let mut input_rx = input_rx;
                    'outer: while let Some((block_no, block)) = input_rx.recv().await {
                        buffer.push((block_no, block));
                        loop {
                            let next_index = buffer
                                .iter()
                                .position(|(bn, _)| *bn == last_block_outputted + 1);
                            let Some(idx) = next_index else {
                                break;
                            };
                            let current = buffer.remove(idx);
                            last_block_outputted += 1;
                            match handle_stage_async(
                                current,
                                &input_info,
                                &output_tx,
                                stage_no,
                                &mut stage,
                                &demultiplex_info,
                                demultiplex_start,
                            )
                            .await
                            {
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
                        }
                    }

                    match stage.finalize(
                        &input_info,
                        &output_prefix,
                        &output_directory,
                        if stage_no >= demultiplex_start {
                            &demultiplex_info
                        } else {
                            &Demultiplexed::No
                        },
                    ) {
                        Ok(Some(report)) => {
                            report_collector.lock().unwrap().push(report);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error_collector
                                .lock()
                                .unwrap()
                                .push(format!("Error finalizing stage {stage_no}: {e:?}"));
                        }
                    }
                }));
            } else {
                let input_info = self.input_info.clone();
                let demultiplex_info = self.demultiplex_info.clone();
                let error_collector = self.error_collector.clone();
                let demultiplex_start = self.demultiplex_start;
                let shared_receiver = Arc::new(AsyncMutex::new(input_rx));
                for _ in 0..local_thread_count {
                    let mut stage = stage.clone();
                    let input_info = input_info.clone();
                    let demultiplex_info = demultiplex_info.clone();
                    let error_collector = error_collector.clone();
                    let output_tx = output_tx.clone();
                    let shared_receiver = shared_receiver.clone();
                    stage_tasks.push(tokio::spawn(async move {
                        loop {
                            let received = {
                                let mut guard = shared_receiver.lock().await;
                                guard.recv().await
                            };
                            let Some(block) = received else {
                                break;
                            };
                            if let Err(e) = handle_stage_async(
                                block,
                                &input_info,
                                &output_tx,
                                stage_no,
                                &mut stage,
                                &demultiplex_info,
                                demultiplex_start,
                            )
                            .await
                            {
                                error_collector
                                    .lock()
                                    .unwrap()
                                    .push(format!("Error in stage {stage_no} processing: {e:?}"));
                                break;
                            }
                        }
                    }));
                }
            }
        }

        let stage_to_output_channel = receivers
            .last_mut()
            .and_then(|rx| rx.take())
            .expect("final stage receiver missing");

        RunStage3 {
            output_directory: self.output_directory,
            demultiplex_info: self.demultiplex_info,
            report_html: self.report_html,
            report_json: self.report_json,
            allow_overwrite: self.allow_overwrite,
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            stage_threads: stage_tasks,
            stage_to_output_channel,
            report_collector,
            error_collector: self.error_collector,
        }
    }
}

pub struct RunStage3 {
    output_directory: PathBuf,
    demultiplex_info: Demultiplexed,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,

    input_threads: Vec<JoinHandle<()>>,
    combiner_thread: JoinHandle<()>,
    stage_threads: Vec<JoinHandle<()>>,
    stage_to_output_channel: BlockReceiver,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
    error_collector: Arc<Mutex<Vec<String>>>,
}

impl RunStage3 {
    #[allow(clippy::too_many_lines)]
    pub fn create_output_threads(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<RunStage4> {
        let mut output_files = open_output_files(
            parsed,
            &self.output_directory,
            &self.demultiplex_info,
            self.report_html,
            self.report_json,
            self.allow_overwrite,
        )?;

        let output_buffer_size = parsed.options.output_buffer_size;
        let cloned_input_config = parsed.input.clone();
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

        let mut stage_threads = self.stage_threads;
        let mut input_channel = self.stage_to_output_channel;
        let output_directory = self.output_directory;
        let demultiplex_info = self.demultiplex_info;
        let report_collector = self.report_collector.clone();
        let error_collector = self.error_collector.clone();

        let output_thread = tokio::spawn(async move {
            let mut last_block_outputted = 0usize;
            let mut buffer: Vec<Block> = Vec::new();
            while let Some((block_no, block)) = input_channel.recv().await {
                buffer.push((block_no, block));
                loop {
                    let next_index = buffer
                        .iter()
                        .position(|(bn, _)| *bn == last_block_outputted + 1);
                    let Some(idx) = next_index else {
                        break;
                    };
                    let (_, block) = buffer.remove(idx);
                    last_block_outputted += 1;
                    if let Err(e) = output_block(
                        &block,
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
                }
            }

            for task in stage_threads.drain(..) {
                if let Err(e) = task.await {
                    error_collector
                        .lock()
                        .unwrap()
                        .push(format!("Failure in stage task: {e}"));
                }
            }

            for set_of_output_files in &mut output_files.output_segments {
                if let Err(e) = set_of_output_files.lock().unwrap().finish() {
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
        });

        Ok(RunStage4 {
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            output_thread,
            error_collector: self.error_collector,
        })
    }
}

pub struct RunStage4 {
    error_collector: Arc<Mutex<Vec<String>>>,
    input_threads: Vec<JoinHandle<()>>,
    combiner_thread: JoinHandle<()>,
    output_thread: JoinHandle<()>,
}

impl RunStage4 {
    pub async fn join_threads(self) -> RunStage5 {
        let mut errors = Vec::new();

        errors.extend(self.error_collector.lock().unwrap().drain(..));

        for handle in self.input_threads {
            if let Err(e) = handle.await {
                errors.push(format!("Failure in input task: {e}"));
            }
        }

        if let Err(e) = self.combiner_thread.await {
            errors.push(format!("Failure in read-combination task: {e}"));
        }

        if let Err(e) = self.output_thread.await {
            errors.push(format!("Failure in output task: {e}"));
        }

        errors.extend(self.error_collector.lock().unwrap().drain(..));

        RunStage5 { errors }
    }
}

pub struct RunStage5 {
    pub errors: Vec<String>,
}

async fn handle_stage_async(
    block: Block,
    input_info: &transformations::InputInfo,
    output_tx: &BlockSender,
    stage_no: usize,
    stage: &mut Transformation,
    demultiplex_info: &Demultiplexed,
    demultiplex_start: usize,
) -> anyhow::Result<bool> {
    let (block_no, block_payload) = block;
    let (out_block, stage_continue) = stage.apply(
        block_payload,
        input_info,
        block_no,
        if stage_no >= demultiplex_start {
            demultiplex_info
        } else {
            &Demultiplexed::No
        },
    )?;

    if output_tx.send((block_no, out_block)).await.is_err() {
        return Ok(false);
    }

    if !stage_continue {
        assert!(
            stage.needs_serial(),
            "Non serial stages must not return do_continue = false"
        );
        return Ok(false);
    }
    Ok(true)
}
