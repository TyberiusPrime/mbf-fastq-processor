#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

use anyhow::{Context, Result};
use crossbeam::channel::bounded;
use ex::Wrapper;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use transformations::{FinalizeReportResult, Step, Transformation};

pub mod config;
pub mod demultiplex;
mod dna;
pub mod io;
mod output;
mod transformations;

pub use config::{Config, FileFormat};
pub use io::FastQRead;
pub use io::{open_input_files, InputFiles, InputSet};

use crate::demultiplex::Demultiplexed;

enum OutputWriter<'a> {
    File(output::HashedAndCompressedWriter<'a, std::fs::File>),
    Stdout(output::HashedAndCompressedWriter<'a, std::io::Stdout>),
}

impl OutputWriter<'_> {
    fn finish(mut self) -> (Option<String>, Option<String>) {
        self.flush().expect("Flushing file failed");
        match self {
            OutputWriter::File(inner) => inner.finish(),
            OutputWriter::Stdout(inner) => inner.finish(),
        }
    }
}

impl std::io::Write for OutputWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputWriter::File(inner) => inner.write(buf),
            OutputWriter::Stdout(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputWriter::File(inner) => inner.flush(),
            OutputWriter::Stdout(inner) => inner.flush(),
        }
    }
}

struct OutputFile<'a> {
    filename: PathBuf,
    writer: OutputWriter<'a>,
}

impl OutputFile<'_> {
    fn new_file(
        filename: impl AsRef<Path>,
        format: FileFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
    ) -> Result<Self> {
        let filename = filename.as_ref().to_owned();
        let file_handle = ex::fs::File::create(&filename)
            .with_context(|| format!("Could not open output file: {}", filename.display()))?;
        Ok(OutputFile {
            filename: filename.clone(),
            writer: OutputWriter::File(output::HashedAndCompressedWriter::new(
                file_handle.into_inner(),
                format,
                do_uncompressed_hash,
                do_compressed_hash,
                compression_level,
            )?),
        })
    }
    fn new_stdout(
        format: FileFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
    ) -> Result<Self> {
        let filename = "stdout".into();
        let file_handle = std::io::stdout();
        Ok(OutputFile {
            filename,
            writer: OutputWriter::Stdout(output::HashedAndCompressedWriter::new(
                file_handle,
                format,
                do_uncompressed_hash,
                do_compressed_hash,
                compression_level,
            )?),
        })
    }

    fn finish(self) -> Result<()> {
        // First flush the writer to complete any compression
        let (uncompressed_hash, compressed_hash) = self.writer.finish();

        if let Some(hash) = uncompressed_hash {
            Self::write_hash_file_static(&self.filename, &hash, ".uncompressed.sha256")?;
        }
        if let Some(hash) = compressed_hash {
            Self::write_hash_file_static(&self.filename, &hash, ".compressed.sha256")?;
        }

        Ok(())
    }

    fn write_hash_file_static(filename: &Path, hash: &str, suffix: &str) -> Result<()> {
        let hash_filename = filename.with_file_name(format!(
            "{}{}",
            filename.file_name().unwrap_or_default().to_string_lossy(),
            suffix
        ));

        let mut fh = ex::fs::File::create(hash_filename)
            .with_context(|| format!("Could not open file for hashing: {}", filename.display()))?;
        fh.write_all(hash.as_bytes())?;
        fh.flush()?;
        Ok(())
    }
}

#[derive(Default)]
struct OutputFastqs<'a> {
    read1: Option<OutputFile<'a>>,
    read2: Option<OutputFile<'a>>,
    index1: Option<OutputFile<'a>>,
    index2: Option<OutputFile<'a>>,
}

impl OutputFastqs<'_> {
    fn finish(&mut self) -> Result<()> {
        if let Some(inner) = self.read1.take() {
            inner.finish()?;
        }

        if let Some(inner) = self.read2.take() {
            inner.finish()?;
        }
        if let Some(inner) = self.index1.take() {
            inner.finish()?;
        }
        if let Some(inner) = self.index2.take() {
            inner.finish()?;
        }
        Ok(())
    }
}

struct OutputReports {
    html: Option<BufWriter<ex::fs::File>>,
    json: Option<BufWriter<ex::fs::File>>,
}

impl OutputReports {
    fn new(
        output_directory: &Path,
        prefix: &String,
        report_html: bool,
        report_json: bool,
    ) -> OutputReports {
        OutputReports {
            html: if report_html {
                Some(BufWriter::new(
                    ex::fs::File::create(output_directory.join(format!("{prefix}.html"))).unwrap(),
                ))
            } else {
                None
            },
            json: if report_json {
                Some(BufWriter::new(
                    ex::fs::File::create(output_directory.join(format!("{prefix}.json"))).unwrap(),
                ))
            } else {
                None
            },
        }
    }
}

#[allow(clippy::too_many_lines)]
fn open_one_set_of_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
    infix: &str,
) -> Result<OutputFastqs<'a>> {
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let suffix = output_config.get_suffix();
            let include_uncompressed_hashes = output_config.output_hash_uncompressed;
            let include_compressed_hashes = output_config.output_hash_compressed;
            let (read1, read2, index1, index2) = match output_config.format {
                FileFormat::None => (None, None, None, None),
                _ => {
                    let (read1, read2) = {
                        if output_config.stdout {
                            //interleaving is handled by outputing both to the read1 output
                            (
                                Some(OutputFile::new_stdout(
                                    output_config.format,
                                    false,
                                    false,
                                    output_config.compression_level,
                                )?),
                                None,
                            )
                        } else if output_config.interleave.is_some() {
                            //interleaving is handled by outputing both to the read1 output
                            ////interleaving requires read2 to be set, checked in validation
                            let interleave = Some(OutputFile::new_file(
                                output_directory.join(format!(
                                    "{}{}_interleaved.{}",
                                    output_config.prefix, infix, suffix
                                )),
                                output_config.format,
                                include_uncompressed_hashes,
                                include_compressed_hashes,
                                output_config.compression_level,
                            )?);
                            (interleave, None)
                        } else {
                            let read1 = if output_config.output_read1 {
                                Some(OutputFile::new_file(
                                    output_directory.join(format!(
                                        "{}{}_1.{}",
                                        output_config.prefix, infix, suffix
                                    )),
                                    output_config.format,
                                    include_uncompressed_hashes,
                                    include_compressed_hashes,
                                    output_config.compression_level,
                                )?)
                            } else {
                                None
                            };
                            let read2 =
                                if (parsed_config.input.get_segment_files("read2").is_some()
                                    && output_config.output_read2)
                                    || false
                                //todo || parsed_config.input.interleaved
                                {
                                    Some(OutputFile::new_file(
                                        output_directory.join(format!(
                                            "{}{}_2.{}",
                                            output_config.prefix, infix, suffix
                                        )),
                                        output_config.format,
                                        include_uncompressed_hashes,
                                        include_compressed_hashes,
                                        output_config.compression_level,
                                    )?)
                                } else {
                                    None
                                };
                            (read1, read2)
                        }
                    };

                    let (index1, index2) = (
                        if output_config.output_index1
                            && parsed_config.input.get_segment_files("index1").is_some()
                        {
                            Some(OutputFile::new_file(
                                output_directory.join(format!(
                                    "{}{}_i1.{}",
                                    output_config.prefix, infix, suffix
                                )),
                                output_config.format,
                                include_uncompressed_hashes,
                                include_compressed_hashes,
                                output_config.compression_level,
                            )?)
                        } else {
                            None
                        },
                        if output_config.output_index2
                            && parsed_config.input.get_segment_files("index2").is_some()
                        {
                            Some(OutputFile::new_file(
                                output_directory.join(format!(
                                    "{}{}_i2.{}",
                                    output_config.prefix, infix, suffix
                                )),
                                output_config.format,
                                include_uncompressed_hashes,
                                include_compressed_hashes,
                                output_config.compression_level,
                            )?)
                        } else {
                            None
                        },
                    );
                    (read1, read2, index1, index2)
                }
            };

            OutputFastqs {
                read1,
                read2,
                index1,
                index2,
            }
        }
        None => OutputFastqs::default(),
    })
}

struct OutputFiles<'a> {
    output_fastq: Vec<Arc<Mutex<OutputFastqs<'a>>>>,
    output_reports: OutputReports,
}

fn open_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
    demultiplexed: &Demultiplexed,
    report_html: bool,
    report_json: bool,
) -> Result<OutputFiles<'a>> {
    let output_reports = match &parsed_config.output {
        Some(output_config) => OutputReports::new(
            output_directory,
            &output_config.prefix,
            report_html,
            report_json,
        ),
        None => OutputReports {
            html: None,
            json: None,
        },
    };

    match demultiplexed {
        Demultiplexed::No => {
            let output_files = open_one_set_of_output_files(parsed_config, output_directory, "")?;
            Ok(OutputFiles {
                output_fastq: vec![Arc::new(Mutex::new(output_files))],
                output_reports,
            })
        }
        Demultiplexed::Yes(demultiplex_info) => {
            let mut res = Vec::new();
            let mut seen: HashMap<String, Arc<Mutex<OutputFastqs>>> = HashMap::new();
            for (_tag, output_key) in demultiplex_info.iter_outputs() {
                if seen.contains_key(output_key) {
                    res.push(seen[output_key].clone());
                } else {
                    let output = Arc::new(Mutex::new(open_one_set_of_output_files(
                        parsed_config,
                        output_directory,
                        &format!("_{output_key}"),
                    )?));
                    seen.insert(output_key.to_string(), output.clone());
                    res.push(output);
                }
            }
            Ok(OutputFiles {
                output_fastq: res,
                output_reports,
            })
        }
    }
}

fn parse_and_send(
    readers: Vec<io::NifflerReader>,
    raw_tx: &crossbeam::channel::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
) {
    let mut parser = io::FastQParser::new(readers, block_size, buffer_size);
    loop {
        let (out_block, was_final) = parser.parse().unwrap();
        match raw_tx.send(out_block) {
            Ok(()) => {}
            Err(_) => {
                break;
            }
        }
        if was_final {
            break;
        }
    }
}

fn parse_interleaved_and_send(
    readers: Vec<io::NifflerReader>,
    raw_tx_read1: &crossbeam::channel::Sender<io::FastQBlock>,
    raw_tx_read2: &crossbeam::channel::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
) {
    let mut parser = io::FastQParser::new(readers, block_size, buffer_size);
    loop {
        let (out_block, was_final) = parser.parse().unwrap();
        let (out_block_r1, out_block_r2) = out_block.split_interleaved();

        match raw_tx_read1.send(out_block_r1) {
            Ok(()) => {}
            Err(_) => {
                break;
            }
        }

        match raw_tx_read2.send(out_block_r2) {
            Ok(()) => {}
            Err(_) => {
                break;
            }
        }
        if was_final {
            break;
        }
    }
}

struct RunStage0 {
    report_html: bool,
    report_json: bool,
}

impl RunStage0 {
    fn new(parsed: &Config) -> Self {
        RunStage0 {
            report_html: parsed.output.as_ref().is_some_and(|o| o.report_html),
            report_json: parsed.output.as_ref().is_some_and(|o| o.report_json),
        }
    }

    fn configure_demultiplex_and_init_stages(
        self,
        parsed: &mut Config,
        output_directory: &Path,
    ) -> Result<RunStage1> {
        let output_prefix = parsed
            .output
            .as_ref()
            .map_or("mbf_fastq_preprocessor_output", |x| &x.prefix)
            .to_string();

        let mut demultiplex_info = Demultiplexed::No;
        let mut demultiplex_start = 0;
        let input_info = transformations::InputInfo {
            has_read1: true,
            has_read2: parsed.input.get_segment_files("read2").is_some(),
            has_index1: parsed.input.get_segment_files("index1").is_some(),
            has_index2: parsed.input.get_segment_files("index2").is_some(),
        };
        for (index, transform) in (parsed.transform).iter_mut().enumerate() {
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
        Ok(RunStage1 {
            report_html: self.report_html,
            report_json: self.report_json,
            output_directory: output_directory.to_owned(),
            output_prefix,
            demultiplex_info,
            demultiplex_start,
        })
    }
}

struct RunStage1 {
    output_prefix: String,
    output_directory: PathBuf,
    demultiplex_info: Demultiplexed,
    demultiplex_start: usize,
    report_html: bool,
    report_json: bool,
}

impl RunStage1 {
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    fn create_input_threads(self, parsed: &Config) -> Result<RunStage2> {
        let input_config = &parsed.input;
        let input_files = open_input_files(input_config).context("Error opening input files")?;

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let channel_size = 2;
        let mut threads = Vec::new();

        //we spawn one reading thread per input file for reading & decompressing.
        let (raw_tx_read1, raw_rx_read1) = bounded(channel_size);
        let input_files = input_files.transpose();
        let has_read2 = input_files.read2.is_some(); //todo || parsed.input.interleaved;
        let has_index1 = input_files.index1.is_some();
        let has_index2 = input_files.index2.is_some();
        #[allow(clippy::if_not_else)]
        let (thread_read1, mut raw_rx_read2, thread_read2) = if true {
            //todo !parsed.input.interleaved {
            let thread_read1 = thread::Builder::new()
                .name("Reader_read1".into())
                .spawn(move || {
                    parse_and_send(input_files.read1, &raw_tx_read1, buffer_size, block_size);
                })
                .unwrap();
            let (raw_rx_read2, thread_read2) = match input_files.read2 {
                Some(reader_read2) => {
                    let (raw_tx_read2, raw_rx_read2) = bounded(channel_size);
                    let thread_read2 = thread::Builder::new()
                        .name("Reader_read2".into())
                        .spawn(move || {
                            parse_and_send(reader_read2, &raw_tx_read2, buffer_size, block_size);
                        })
                        .unwrap();
                    (Some(raw_rx_read2), Some(thread_read2))
                }
                None => (None, None),
            };
            (thread_read1, raw_rx_read2, thread_read2)
        } else {
            // if interleaved...
            let (raw_tx_read2, raw_rx_read2) = bounded(channel_size);
            let thread_read_interleaved = thread::Builder::new()
                .name("Reader_interleaved".into())
                .spawn(move || {
                    parse_interleaved_and_send(
                        input_files.read1,
                        &raw_tx_read1,
                        &raw_tx_read2,
                        buffer_size,
                        block_size,
                    );
                })
                .unwrap();

            (thread_read_interleaved, Some(raw_rx_read2), None)
        };

        threads.push(thread_read1);
        if let Some(thread_read2) = thread_read2 {
            threads.push(thread_read2);
        }

        let (mut raw_rx_index1, thread_index1) = match input_files.index1 {
            Some(reader_index1) => {
                let (raw_tx_index1, raw_rx_index1) = bounded(channel_size);
                let thread_index1 = thread::Builder::new()
                    .name("Reader_i1".into())
                    .spawn(move || {
                        parse_and_send(reader_index1, &raw_tx_index1, buffer_size, block_size);
                    })
                    .unwrap();
                (Some(raw_rx_index1), Some(thread_index1))
            }
            None => (None, None),
        };
        if let Some(thread_index1) = thread_index1 {
            threads.push(thread_index1);
        }

        let (mut raw_rx_index2, thread_index2) = match input_files.index2 {
            Some(reader_index2) => {
                let (raw_tx_index2, raw_rx_index2) = bounded(channel_size);
                let thread_index2 = thread::Builder::new()
                    .name("Reader_i2".into())
                    .spawn(move || {
                        parse_and_send(reader_index2, &raw_tx_index2, buffer_size, block_size);
                    })
                    .unwrap();
                (Some(raw_rx_index2), Some(thread_index2))
            }
            None => (None, None),
        };
        if let Some(thread_index2) = thread_index2 {
            threads.push(thread_index2);
        }

        let (combiner_output_tx, combiner_output_rx) =
            bounded::<(usize, io::FastQBlocksCombined)>(channel_size);

        //to.
        let combiner = thread::Builder::new()
            .name("Combiner".into())
            .spawn(move || {
                //I need to receive the blocks (from all four input threads)
                //and then, match them up into something that's the same length!
                let mut block_no = 1; // for the sorting later on.
                loop {
                    let Ok(block_read1) = raw_rx_read1.recv() else {
                        break;
                    };
                    let block_read2 = if has_read2 {
                        let r = match raw_rx_read2.as_mut().unwrap().recv() {
                            Ok(block) => block,
                            Err(e) => panic!("Block for read1 received, but not for read2!: {e:?}"),
                        };
                        assert_eq!(r.entries.len(), block_read1.entries.len());
                        Some(r)
                    } else {
                        None
                    };
                    let block_index1 = if has_index1 {
                        match raw_rx_index1.as_mut().unwrap().recv() {
                            Ok(block) => Some(block),
                            _ => panic!("Block for read1 received, but not for index1!"),
                        }
                    } else {
                        None
                    };

                    let block_index2 = if has_index2 {
                        match raw_rx_index2.as_mut().unwrap().recv() {
                            Ok(block) => Some(block),
                            _ => panic!("Block for read1 received, but not for index2!"),
                        }
                    } else {
                        None
                    };

                    let out = (
                        block_no,
                        io::FastQBlocksCombined {
                            read1: block_read1,
                            read2: block_read2,
                            index1: block_index1,
                            index2: block_index2,
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

        Ok(RunStage2 {
            output_prefix: self.output_prefix,
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_info: self.demultiplex_info.clone(),
            demultiplex_start: self.demultiplex_start,
            input_threads: threads,
            combiner_thread: combiner,
            combiner_output_rx,
        })
    }
}

struct RunStage2 {
    output_prefix: String,
    output_directory: PathBuf,
    report_html: bool,
    report_json: bool,
    demultiplex_info: Demultiplexed,
    demultiplex_start: usize,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    combiner_output_rx: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,
}
impl RunStage2 {
    #[allow(clippy::too_many_lines)]
    fn create_stage_threads(self, parsed: &Config) -> RunStage3 {
        let stages = &parsed.transform;
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

        for (stage_no, stage) in stages.iter().enumerate() {
            let needs_serial = stage.needs_serial();
            let transmits_premature_termination = stage.transmits_premature_termination();
            let local_thread_count = if needs_serial { 1 } else { thread_count };
            for _ in 0..local_thread_count {
                let mut stage = stage.clone();
                let input_rx2 = channels[stage_no].1.clone();
                let output_tx2 = channels[stage_no + 1].0.clone();
                let output_prefix = output_prefix.clone();
                let output_directory = self.output_directory.clone();
                let demultiplex_info2 = self.demultiplex_info.clone();
                let report_collector = report_collector.clone();
                if needs_serial {
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
                                                let do_continue = handle_stage(
                                                    to_output,
                                                    &output_tx2,
                                                    stage_no,
                                                    &mut stage,
                                                    &demultiplex_info2,
                                                    self.demultiplex_start,
                                                );
                                                if !do_continue && transmits_premature_termination {
                                                    break 'outer;
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                let report = stage
                                    .finalize(
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
                    threads.push(
                        thread::Builder::new()
                            .name(format!("Stage {stage_no}"))
                            .spawn(move || {
                                loop {
                                    match input_rx2.recv() {
                                        Ok(block) => {
                                            handle_stage(
                                                block,
                                                &output_tx2,
                                                stage_no,
                                                &mut stage,
                                                &demultiplex_info2,
                                                self.demultiplex_start,
                                            );
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
            output_directory: self.output_directory,
            report_html: self.report_html,
            report_json: self.report_json,
            demultiplex_info: self.demultiplex_info.clone(),
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            stage_threads: threads,
            stage_to_output_channel: channels[channels.len() - 1].1.clone(),
            report_collector,
        }
    }
}

struct RunStage3 {
    output_directory: PathBuf,
    demultiplex_info: Demultiplexed,
    report_html: bool,
    report_json: bool,

    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    stage_threads: Vec<thread::JoinHandle<()>>,
    stage_to_output_channel: crossbeam::channel::Receiver<(usize, io::FastQBlocksCombined)>,
    report_collector: Arc<Mutex<Vec<FinalizeReportResult>>>,
}

fn collect_thread_failures(threads: Vec<thread::JoinHandle<()>>, msg: &str) -> Vec<String> {
    let mut stage_errors = Vec::new();
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
    fn create_output_threads(
        self,
        parsed: &Config,
        report_labels: Vec<String>,
        raw_config: String,
    ) -> Result<RunStage4> {
        let input_channel = self.stage_to_output_channel;
        let interleaved = parsed.output.as_ref().is_some_and(|o| o.interleave.is_some());
        let output_buffer_size = parsed.options.output_buffer_size;
        let cloned_input_config = parsed.input.clone();

        let mut output_files = open_output_files(
            parsed,
            &self.output_directory,
            &self.demultiplex_info,
            self.report_html,
            self.report_json,
        )?;

        let output_directory = self.output_directory;
        let demultiplex_info = self.demultiplex_info;
        let report_collector = self.report_collector.clone();

        let output = thread::Builder::new()
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
                            output_block(
                                &to_output.1,
                                &mut output_files.output_fastq,
                                interleaved,
                                &demultiplex_info,
                                output_buffer_size,
                            );
                        } else {
                            break;
                        }
                    }
                }
                //all blocks are done, the stage output channel has been closed.
                //but that doesn't mean the threads are done and have pushed the reports.
                //so we join em here
                let stage_errors = collect_thread_failures(self.stage_threads, "stage error");
                assert!(
                    stage_errors.is_empty(),
                    "Error in stage threads occured: {stage_errors:?}"
                );

                for set_of_output_files in &mut output_files.output_fastq {
                    set_of_output_files
                        .lock()
                        .unwrap()
                        .finish()
                        .expect("Error finishing output files"); //todo: turn into result?
                }
                //todo: wait for all reports to have been sent...
                let json_report = {
                    let need_json = output_files.output_reports.json.is_some()
                        | output_files.output_reports.html.is_some();
                    if need_json {
                        Some(
                            output_json_report(
                                output_files.output_reports.json.as_mut(), // None if no .json file
                                // generated
                                &report_collector,
                                &report_labels,
                                &output_directory.to_string_lossy(),
                                &cloned_input_config,
                                &raw_config,
                            )
                            .expect("error writing json report"),
                        )
                    } else {
                        None
                    }
                };

                if let Some(output_html) = output_files.output_reports.html.as_mut() {
                    output_html_report(output_html, &json_report.unwrap())
                        .expect("error writing html report");
                }
            })
            .unwrap();

        Ok(RunStage4 {
            input_threads: self.input_threads,
            combiner_thread: self.combiner_thread,
            output_thread: output,
        })
    }
}

struct RunStage4 {
    input_threads: Vec<thread::JoinHandle<()>>,
    combiner_thread: thread::JoinHandle<()>,
    output_thread: thread::JoinHandle<()>,
}

impl RunStage4 {
    fn join_threads(self) -> RunStage5 {
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
            errors.extend(collect_thread_failures(threads, msg));
        }

        RunStage5 { errors }
    }
}

struct RunStage5 {
    errors: Vec<String>,
}

#[allow(clippy::similar_names)] // I like rx/tx nomenclature
#[allow(clippy::too_many_lines)] //todo: this is true.
pub fn run(
    toml_file: &Path,
    output_directory: &Path, //todo: figure out wether this is just an output directory, or a
                             //*working* directory
) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;
    parsed.check().context("Error in configuration")?;
    let (new_transforms, report_labels) = Transformation::expand(parsed.transform);
    parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
    #[allow(clippy::if_not_else)]
    {
        let run = RunStage0::new(&parsed);
        let run = run.configure_demultiplex_and_init_stages(&mut parsed, &output_directory)?;
        let parsed = parsed; //after this, stages are transformed and ready, and config is read only.
        let run = run.create_input_threads(&parsed)?;
        let run = run.create_stage_threads(&parsed);
        let run = run.create_output_threads(&parsed, report_labels, raw_config)?;
        let run = run.join_threads();
        //
        //promote all panics to actual process failures with exit code != 0
        let errors = run.errors;
        if !errors.is_empty() {
            eprintln!("\nErrors occurred during processing:");
            for error in &errors {
                eprintln!("{error}");
            }
            std::process::exit(101);
        }
        //assert!(errors.is_empty(), "Error in threads occured: {errors:?}");

        //ok all this needs is a buffer that makes sure we reorder correctly at the end.
        //and then something block based, not single reads to pass between the threads.
        drop(parsed);
    }

    Ok(())
}

fn handle_stage(
    block: (usize, io::FastQBlocksCombined),
    output_tx2: &crossbeam::channel::Sender<(usize, io::FastQBlocksCombined)>,
    stage_no: usize,
    stage: &mut Transformation,
    demultiplex_info: &Demultiplexed,
    demultiplex_start: usize,
) -> bool {
    let mut out_block = block.1;
    let mut do_continue = true;
    let stage_continue;

    (out_block, stage_continue) = stage.apply(
        out_block,
        block.0,
        if stage_no >= demultiplex_start {
            demultiplex_info
        } else {
            &Demultiplexed::No
        },
    );
    do_continue = do_continue && stage_continue;

    match output_tx2.send((block.0, out_block)) {
        Ok(()) => {}
        Err(_) => {
            // downstream has hung up
            return false;
        }
    }
    if !do_continue {
        assert!(
            stage.needs_serial(),
            "Non serial stages must not return do_continue = false"
        );
        return false;
    }
    true
}

#[allow(clippy::if_not_else)]
fn output_block(
    block: &io::FastQBlocksCombined,
    output_files: &mut [Arc<Mutex<OutputFastqs>>],
    interleaved: bool,
    demultiplexed: &Demultiplexed,
    buffer_size: usize,
) {
    block.sanity_check();
    match demultiplexed {
        Demultiplexed::No => {
            output_block_demultiplex(block, &mut output_files[0], interleaved, None, buffer_size);
        }
        Demultiplexed::Yes(demultiplex_info) => {
            for (file_no, (tag, _output_key)) in demultiplex_info.iter_outputs().enumerate() {
                let output_files = &mut output_files[file_no];
                output_block_demultiplex(block, output_files, interleaved, Some(tag), buffer_size);
            }
        }
    }
}

#[allow(clippy::if_not_else)]
fn output_block_demultiplex(
    block: &io::FastQBlocksCombined,
    output_files: &mut Arc<Mutex<OutputFastqs>>,
    interleaved: bool,
    tag: Option<u16>,
    buffer_size: usize,
) {
    let mut buffer = Vec::with_capacity(buffer_size);
    let mut of = output_files.lock().unwrap();
    if !interleaved {
        output_block_inner(
            of.read1.as_mut(),
            Some(&block.read1),
            &mut buffer,
            buffer_size,
            tag,
            block.output_tags.as_ref(),
        );
        output_block_inner(
            of.read2.as_mut(),
            block.read2.as_ref(),
            &mut buffer,
            buffer_size,
            tag,
            block.output_tags.as_ref(),
        );
    } else {
        output_block_interleaved(
            of.read1.as_mut(),
            &block.read1,
            block.read2.as_ref().unwrap(),
            &mut buffer,
            buffer_size,
            tag,
            block.output_tags.as_ref(),
        );
    }
    output_block_inner(
        of.index1.as_mut(),
        block.index1.as_ref(),
        &mut buffer,
        buffer_size,
        tag,
        block.output_tags.as_ref(),
    );
    output_block_inner(
        of.index2.as_mut(),
        block.index2.as_ref(),
        &mut buffer,
        buffer_size,
        tag,
        block.output_tags.as_ref(),
    );
}

fn output_block_inner(
    output_file: Option<&mut OutputFile<'_>>,
    block: Option<&io::FastQBlock>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) {
    if let Some(of) = output_file {
        let mut pseudo_iter = if let Some(demultiplex_tag) = demultiplex_tag {
            block
                .unwrap()
                .get_pseudo_iter_filtered_to_tag(demultiplex_tag, output_tags.as_ref().unwrap())
        } else {
            block.unwrap().get_pseudo_iter()
        };
        while let Some(read) = pseudo_iter.pseudo_next() {
            read.append_as_fastq(buffer);
            if buffer.len() > buffer_size {
                of.writer.write_all(buffer).unwrap();
                buffer.clear();
            }
        }
        of.writer.write_all(buffer).unwrap();
    }
    buffer.clear();
}

#[allow(clippy::too_many_arguments)]
fn output_block_interleaved(
    output_file: Option<&mut OutputFile<'_>>,
    block_r1: &io::FastQBlock,
    block_r2: &io::FastQBlock,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) {
    if let Some(of) = output_file {
        let mut pseudo_iter = if let Some(demultiplex_tag) = demultiplex_tag {
            block_r1.get_pseudo_iter_filtered_to_tag(demultiplex_tag, output_tags.as_ref().unwrap())
        } else {
            block_r1.get_pseudo_iter()
        };
        let mut pseudo_iter_2 = if let Some(demultiplex_tag) = demultiplex_tag {
            block_r2.get_pseudo_iter_filtered_to_tag(demultiplex_tag, output_tags.as_ref().unwrap())
        } else {
            block_r2.get_pseudo_iter()
        };
        while let Some(read) = pseudo_iter.pseudo_next() {
            let read2 = pseudo_iter_2
                .pseudo_next()
                .expect("Uneven number of r1 and r2 in interleaved output. Bug?");
            read.append_as_fastq(buffer);
            read2.append_as_fastq(buffer);
            if buffer.len() > buffer_size {
                of.writer.write_all(buffer).unwrap();
                buffer.clear();
            }
        }

        of.writer.write_all(buffer).unwrap();
    }
    buffer.clear();
}

fn format_seconds_to_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

fn output_json_report(
    output_file: Option<&mut BufWriter<ex::fs::File>>,
    report_collector: &Arc<Mutex<Vec<FinalizeReportResult>>>,
    report_labels: &[String],
    current_dir: &str,
    input_config: &crate::config::Input,
    raw_config: &str,
) -> Result<String> {
    use json_value_merge::Merge;
    let mut output: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    //store run info such as version in "__"
    output.insert(
        "__".to_string(),
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "cwd": std::env::current_dir().unwrap(),
            "input_files": input_config,
        }),
    );
    let reports = report_collector.lock().unwrap();
    for report in reports.iter() {
        let key = report_labels[report.report_no].clone();
        match output.entry(key) {
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(report.contents.clone());
            }
            serde_json::map::Entry::Occupied(mut entry) => entry.get_mut().merge(&report.contents),
        }
    }
    let mut run_info = serde_json::Map::new();

    run_info.insert(
        "program_version".to_string(),
        serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );

    run_info.insert(
        "input_toml".to_string(),
        serde_json::Value::String(raw_config.to_string()),
    );
    run_info.insert(
        "working_directory".to_string(),
        serde_json::Value::String(current_dir.to_string()),
    );

    output.insert("run_info".to_string(), serde_json::Value::Object(run_info));

    let str_output = serde_json::to_string_pretty(&output)?;
    if let Some(output_file) = output_file {
        output_file.write_all(str_output.as_bytes())?;
    }
    Ok(str_output)
}

fn output_html_report(
    output_file: &mut BufWriter<ex::fs::File>,
    json_report_string: &str,
) -> Result<()> {
    let template = include_str!("../html/template.html");
    let chartjs = include_str!("../html/chart/chart.umd.min.js");
    let html = template
        .replace("%TITLE%", "mbf-fastq-processor-report")
        .replace("\"%DATA%\"", json_report_string)
        .replace("/*%CHART%*/", chartjs);

    output_file.write_all(html.as_bytes())?;
    Ok(())
}
