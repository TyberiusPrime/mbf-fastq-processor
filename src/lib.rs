
use anyhow::{Context, Result};
use ex::Wrapper;
use flate2::write::GzEncoder;
use sha2::Digest;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::fmt;

pub mod config;
mod fastq_read;
pub mod io;
mod transformations;

use config::{check_config, Config,  FileFormat};
pub use fastq_read::FastQRead;
pub use io::{open_input_files, InputFiles, InputSet};

impl std::fmt::Debug for FastQRead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n    Name: ")?;
        f.write_str(std::str::from_utf8(&self.name).unwrap_or("<invalid utf8>"))?;
        f.write_str("\n    Seq:  ")?;
        f.write_str(std::str::from_utf8(&self.seq).unwrap_or("<invalid utf8>"))?;
        f.write_str("\n    Qual: ")?;
        f.write_str(std::str::from_utf8(&self.qual).unwrap_or("<invalid utf8>"))?;
        f.write_str("\n")?;
        Ok(())
    }
}

pub struct Molecule {
    pub read1: FastQRead,
    pub read2: Option<FastQRead>,
    pub index1: Option<FastQRead>,
    pub index2: Option<FastQRead>,
}

impl Molecule {
    }

impl std::fmt::Debug for Molecule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Molecule:\n")?;
        f.write_str("  Read1: ")?;
        f.write_str(&format!("{:?}", self.read1))?;
        if let Some(read2) = &self.read2 {
            f.write_str("\n  Read2: ")?;
            f.write_str(&format!("{:?}", read2))?;
        }
        if let Some(index1) = &self.index1 {
            f.write_str("\n  Index1: ")?;
            f.write_str(&format!("{:?}", index1))?;
        }
        if let Some(index2) = &self.index2 {
            f.write_str("\n  Index2: ")?;
            f.write_str(&format!("{:?}", index2))?;
        }
        f.write_str(")")?;
        Ok(())
    }
}

enum Writer<'a> {
    Raw(BufWriter<std::fs::File>),
    Gzip(GzEncoder<BufWriter<std::fs::File>>),
    Zstd(zstd::stream::AutoFinishEncoder<'a, BufWriter<std::fs::File>>),
}

impl<'a> Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Raw(inner) => inner.write(buf),
            Writer::Gzip(inner) => inner.write(buf),
            Writer::Zstd(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Raw(inner) => inner.flush(),
            Writer::Gzip(inner) => inner.flush(),
            Writer::Zstd(inner) => inner.flush(),
        }
    }
}

#[derive(Default)]
struct OutputFiles<'a> {
    read1: Option<Writer<'a>>,
    read2: Option<Writer<'a>>,
    index1: Option<Writer<'a>>,
    index2: Option<Writer<'a>>,
    /* reports: Vec<Writer<'a>>,
    inspects: Vec<(
        Option<Writer<'a>>,
        Option<Writer<'a>>,
        Option<Writer<'a>>,
        Option<Writer<'a>>,
     )>, */
     hashers: [
         Option<sha2::Sha256>; 4
     ],
}

fn open_raw_output_file<'a>(path: &PathBuf) -> Result<Writer<'a>> {
    let fh = ex::fs::File::create(path).context("Could not open file.")?;
    let bufwriter = BufWriter::new(fh.into_inner());
    Ok(Writer::Raw(bufwriter))
}

fn open_gzip_output_file<'a>(path: &PathBuf, level: flate2::Compression) -> Result<Writer<'a>> {
    let fh = std::fs::File::create(path).context("Could not open file.")?;
    let buf_writer = BufWriter::new(fh);
    let gz = GzEncoder::new(buf_writer, level);
    Ok(Writer::Gzip(gz))
}
fn open_zstd_output_file<'a>(path: &PathBuf, level: i32) -> Result<Writer<'a>> {
    let fh = std::fs::File::create(path).context("Could not open file.")?;
    let buf_writer = BufWriter::new(fh);
    let encoder = zstd::stream::Encoder::new(buf_writer, level)?;
    let encoder = encoder.auto_finish();
    Ok(Writer::Zstd(encoder))
}
fn open_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
) -> Result<(OutputFiles<'a>, String)> {
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let suffix =
                output_config
                    .suffix
                    .as_deref()
                    .unwrap_or_else(|| match output_config.format {
                        FileFormat::Raw => ".fq",
                        FileFormat::Gzip => ".fq.gz",
                        FileFormat::Zstd => ".fq.zst",
                        FileFormat::None => "",
                    });
            let (read1, read2, index1, index2) = match output_config.format {
                //todo: refactor
                FileFormat::Raw => {
                    let read1 = Some(open_raw_output_file(
                        &output_directory.join(&format!("{}_1{}", output_config.prefix, suffix)),
                    )?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_raw_output_file(
                            &output_directory
                                .join(&format!("{}_2{}", output_config.prefix, suffix)),
                        )?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_raw_output_file(
                                &output_directory
                                    .join(&format!("{}_i1{}", output_config.prefix, suffix)),
                            )?),
                            Some(open_raw_output_file(
                                &output_directory
                                    .join(&format!("{}_i2{}", output_config.prefix, suffix)),
                            )?),
                        )
                    } else {
                        (None, None)
                    };
                    (read1, read2, index1, index2)
                }
                FileFormat::Gzip => {
                    let read1 = Some(open_gzip_output_file(
                        &output_directory.join(&format!("{}_1{}", output_config.prefix, suffix)),
                        flate2::Compression::default(),
                    )?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_gzip_output_file(
                            &output_directory
                                .join(&format!("{}_2{}", output_config.prefix, suffix)),
                            flate2::Compression::default(),
                        )?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_gzip_output_file(
                                &output_directory
                                    .join(&format!("{}_i1{}", output_config.prefix, suffix)),
                                flate2::Compression::default(),
                            )?),
                            Some(open_gzip_output_file(
                                &output_directory
                                    .join(&format!("{}_i2{}", output_config.prefix, suffix)),
                                flate2::Compression::default(),
                            )?),
                        )
                    } else {
                        (None, None)
                    };

                    (read1, read2, index1, index2)
                }
                FileFormat::Zstd => {
                    let read1 = Some(open_zstd_output_file(
                        &output_directory.join(&format!("{}_1{}", output_config.prefix, suffix)),
                        5,
                    )?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_zstd_output_file(
                            &output_directory
                                .join(&format!("{}_2{}", output_config.prefix, suffix)),
                            5,
                        )?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_zstd_output_file(
                                &output_directory
                                    .join(&format!("{}_i1{}", output_config.prefix, suffix)),
                                5,
                            )?),
                            Some(open_zstd_output_file(
                                &output_directory
                                    .join(&format!("{}_i2{}", output_config.prefix, suffix)),
                                5,
                            )?),
                        )
                    } else {
                        (None, None)
                    };

                    (read1, read2, index1, index2)
                }
                FileFormat::None => {
                    (None, None, None, None)
                }
            };
      //      let reports = Vec::new();
       //     let inspects = Vec::new();
            let hashers = if output_config.output_hash {
                [
                    Some(sha2::Sha256::new()),
                    if read2.is_some() {
                        Some(sha2::Sha256::new())
                    } else {
                        None
                    },
                    if index1.is_some() {
                        Some(sha2::Sha256::new())
                    } else {
                        None
                    },
                    if index2.is_some() {
                        Some(sha2::Sha256::new())
                    } else {
                        None
                    },
                ]
            } else {
                [None, None, None, None]
            };
            //todo: open report files.
            (
                OutputFiles {
                    read1,
                    read2,
                    index1,
                    index2,
    //                reports,
     //               inspects,
                    hashers,
                },
                output_config.prefix.to_string(),
            )
        }
        None => (
            OutputFiles::default(),
            "mbf_fastq_preprocessor_output".to_string(),
        ),
    })
}

#[derive(Debug, Clone)]
struct Stage {
    transforms: Vec<transformations::Transformation>,
    needs_serial: bool,
    can_terminate: bool, //can we 'skip' throwing all reads at this stage if a Head happend?
}

/// Split into transforms we can do parallelized
/// and transforms taht
fn split_transforms_into_stages(
    transforms: &[transformations::Transformation],
) -> Vec<Stage> {
    if transforms.is_empty() {
        return Vec::new();
    }
    let mut stages: Vec<Stage> = Vec::new();
    let mut current_stage = Vec::new();
    let mut last = None;
    let mut can_terminate = true;
    for transform in transforms {
        let need_serial = transform.needs_serial();
        if transform.must_run_to_completion() {
            can_terminate = false;
        }
        if Some(need_serial) != last {
            if !current_stage.is_empty() {
                stages.push(Stage{
                    transforms: current_stage,
                    needs_serial: last.take().unwrap(),
                can_terminate
                });
            }
            last = Some(need_serial);
            current_stage = Vec::new();
        }
        current_stage.push(transform.clone());
    }
    stages.push(
        Stage{
            transforms: current_stage, 
            needs_serial: last.take().unwrap(),
            can_terminate
    });
    stages
}

fn parse_and_send(
    readers: Vec<io::NifflerReader>,
    raw_tx_read1: crossbeam::channel::Sender<io::FastQBlock>,
    buffer_size: usize,
    block_size: usize,
    premature_termination_signaled: Arc<AtomicBool>,
) -> () {
    let mut parser = io::FastQParser::new(readers, block_size, buffer_size);
    loop {
        let (out_block, was_final) = parser.parse().unwrap();
        match raw_tx_read1.send(out_block) {
            Ok(_) => {}
            Err(e) => {
                if premature_termination_signaled.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                } else {
                    panic!("Error sending parsed block to next stage: {:?}", e)
                }
            }
        }
        if was_final {
            break;
        }
    }
}
/* ) -> () {
    let mut current = 0;
    let mut last_partial = io::PartialStatus::NoPartial;
    let mut last_partial_read = None;

    while current < reader_read1.len() {
        loop {
            let mut buffer = vec![0; buffer_size];
            let more = reader_read1[current].read(&mut buffer).unwrap();
            println!("read bytes: {more} -buffer size {buffer_size} thread: {:?}", thread::current().id());
            if more == 0 {
                if last_partial != io::PartialStatus::NoPartial {
                    panic!("(Sub) fastq file did not end in a complete read"); //todo: promote this
                                                                               //to an error outside of the thread...
                }
                break;
            }
            buffer.resize(more, 0); //restrict to actually read bytes
            let fq_block =
                io::parse_to_fastq_block(buffer, last_partial, last_partial_read).unwrap();
            last_partial = fq_block.status;
            last_partial_read = fq_block.partial_read;
            dbg!(fq_block.block.entries.len());
            //
            //
            if !fq_block.block.entries.is_empty() {
                match raw_tx_read1.send(fq_block.block) {
                    Ok(_) => {}
                    Err(e) => {
                        if premature_termination_signaled.load(std::sync::atomic::Ordering::Relaxed)
                        {
                            break;
                        } else {
                            panic!("Error sending parsed block to next stage: {:?}", e)
                        }
                    }
                }
            }
        }
        current += 1;
    }
} */

pub fn run(toml_file: &Path, output_directory: &Path) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file).context("Could not read toml file.")?;
    let parsed = toml::from_str::<Config>(&raw_config).context("Could not parse toml file.")?;
    check_config(&parsed)?;
    //let start_time = std::time::Instant::now();
    {
        let input_files =
            open_input_files(parsed.input.clone()).context("error opening input files")?;
        let (mut output_files, output_prefix) = open_output_files(&parsed, &output_directory)?;

        use crossbeam::channel::bounded;
        let channel_size = 50;

        let mut stages = split_transforms_into_stages(&parsed.transform);

        let channels: Vec<_> = (0..stages.len() + 1)
            .into_iter()
            .map(|_| {
                let (tx, rx) = bounded::<(usize, io::FastQBlocksCombined)>(channel_size);
                (tx, rx)
            })
            .collect();

        let block_size = parsed.options.block_size;
        let buffer_size = parsed.options.buffer_size;
        let premature_termination_signaled = Arc::new(AtomicBool::new(false));
        let channel_size = 2;

        //we spawn one reading thread per input file for reading & decompressing.
        let (raw_tx_read1, raw_rx_read1) = bounded(channel_size);
        let (reader_read1, reader_read2, reader_index1, reader_index2) = input_files.transpose();
        let has_read2 = reader_read2.is_some();
        let has_index1 = reader_index1.is_some();
        let has_index2 = reader_index2.is_some();
        let premature_termination_signaled2 = premature_termination_signaled.clone();
        let thread_read1 = thread::spawn(move || {
            parse_and_send(
                reader_read1,
                raw_tx_read1,
                buffer_size,
                block_size,
                premature_termination_signaled2,
            );
        });
        let (mut raw_rx_read2, thread_read2) = match reader_read2 {
            Some(reader_read2) => {
                let (raw_tx_read2, raw_rx_read2) = bounded(channel_size);
                let premature_termination_signaled2 = premature_termination_signaled.clone();
                let thread_read2 = thread::spawn(move || {
                    parse_and_send(
                        reader_read2,
                        raw_tx_read2,
                        buffer_size,
                        block_size,
                        premature_termination_signaled2,
                    );
                });
                (Some(raw_rx_read2), Some(thread_read2))
            }
            None => (None, None),
        };
        let (mut raw_rx_index1, thread_index1) = match reader_index1 {
            Some(reader_index1) => {
                let (raw_tx_index1, raw_rx_index1) = bounded(channel_size);
                let premature_termination_signaled2 = premature_termination_signaled.clone();
                let thread_index1 = thread::spawn(move || {
                    parse_and_send(
                        reader_index1,
                        raw_tx_index1,
                        buffer_size,
                        block_size,
                        premature_termination_signaled2,
                    );
                });
                (Some(raw_rx_index1), Some(thread_index1))
            }
            None => (None, None),
        };
        let (mut raw_rx_index2, thread_index2) = match reader_index2 {
            Some(reader_index2) => {
                let (raw_tx_index2, raw_rx_index2) = bounded(channel_size);
                let premature_termination_signaled2 = premature_termination_signaled.clone();
                let thread_index2 = thread::spawn(move || {
                    parse_and_send(
                        reader_index2,
                        raw_tx_index2,
                        buffer_size,
                        block_size,
                        premature_termination_signaled2,
                    );
                });
                (Some(raw_rx_index2), Some(thread_index2))
            }
            None => (None, None),
        };

        let input_channel = channels[0].0.clone(); //where the blocks of fastq reads are sent off
                                                   //to.
        let premature_termination_signaled2 = premature_termination_signaled.clone();
        let combiner = thread::spawn(move || {
            //I need to receive the blocks (from all four input threads)
            //and then, match them up into something that's the same length!
            let mut block_no = 1; // for the sorting later on.
            loop {
                let block_read1 = match raw_rx_read1.recv() {
                    Ok(block) => block,
                    Err(_) => {
                        break;
                    }
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
                        Err(_) => panic!("Block for read1 received, but not for index1!"),
                    }
                } else {
                    None
                };

                let block_index2 = if has_index2 {
                    match raw_rx_index2.as_mut().unwrap().recv() {
                        Ok(block) => Some(block),
                        Err(_) => panic!("Block for read1 received, but not for index2!"),
                    }
                } else {
                    None
                };

                let out = (
                    block_no,
                    io::FastQBlocksCombined {
                        block_read1,
                        block_read2,
                        block_index1,
                        block_index2,
                    },
                );
                block_no += 1;
                match input_channel.send(out) {
                    Ok(_) => {}
                    Err(e) => {
                        if premature_termination_signaled2
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            break;
                        } else {
                            panic!("Error sending combined block to next stage: {:?}", e);
                        }
                    }
                }
            }
            premature_termination_signaled2.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let thread_count = parsed.options.thread_count;
        //stage processors.
        // println!("Thread count {}", thread_count);
        let mut processors = Vec::new();
        let output_prefix = Arc::new(output_prefix);
        for stage in stages.iter_mut() {
            for transform in stage.transforms.iter_mut() {
                transform
                    .initialize(&output_prefix, &output_directory)
                    .unwrap();
            }
        }

        for (stage_no, stage) in stages.into_iter().enumerate() {
            let local_thread_count = if stage.needs_serial { 1 } else { thread_count };
            for _ in 0..local_thread_count {
                let mut stage = stage.clone();
                let input_rx2 = channels[stage_no].1.clone();
                let output_tx2 = channels[stage_no + 1].0.clone();
                let premature_termination_signaled = premature_termination_signaled.clone();
                let output_prefix = output_prefix.clone();
                let output_directory = output_directory.clone();
                let processor = if stage.needs_serial {
                    thread::spawn(move || {
                        //we need to ensure the blocks are passed on in order
                        let mut last_block_outputted = 0;
                        let mut buffer = Vec::new();
                        'outer: loop {
                            match input_rx2.recv() {
                                Ok((block_no, block)) => {
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
                                                    &premature_termination_signaled,
                                                    &mut stage,
                                                );
                                                if !do_continue {
                                                    if stage.can_terminate {
                                                        break 'outer;
                                                    }
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                        for transform in stage.transforms.iter_mut() {
                            transform
                                .finalize(&output_prefix, &output_directory)
                                .unwrap();
                        }
                    })
                } else {
                    thread::spawn(move || loop {
                        match input_rx2.recv() {
                            Ok(block) => {
                                handle_stage(
                                    block,
                                    &output_tx2,
                                    &premature_termination_signaled,
                                    &mut stage,
                                );
                            }
                            Err(_) => {
                                return;
                            }
                        }
                    })
                };
                processors.push(processor);
            }
        }

        let output_channel = (channels[channels.len() - 1]).1.clone();
        drop(channels); //we must not hold a reference here across the join at the end
        let output = thread::spawn(move || {
            let mut last_block_outputted = 0;
            let mut buffer = Vec::new();
            loop {
                match output_channel.recv() {
                    Ok((block_no, block)) => {
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
                                output_block(to_output.1, &mut output_files);
                            } else {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            for (ii, infix) in ["1", "2", "i1", "i2"].iter().enumerate() {
                if let Some(hasher) = output_files.hashers[ii].take() {
                    let result = hasher.finalize();
                    let str_result = hex::encode(result);
                    let mut hash_file = std::fs::File::create(
                        output_directory.join(format!("{}_{}.sha256", output_prefix, infix)),
                    )
                    .expect("Failed to create hash output file");
                    //let mut bufwriter = BufWriter::new(hash_file);
                    hash_file
                        .write_all(str_result.as_bytes())
                        .expect("failed to fill hash output file");
                }
            }
        });
        //promote all panics to actual process failures with exit code != 0
        let mut errors = Vec::new();
        if let Err(e) = output.join() {
            errors.push(format!("Failure in output thread: {e:?}"));
        }
        if let Err(e) = combiner.join() {
            errors.push(format!("Failure in read-combination-thread thread {e:?}"));
        }
        for p in processors {
            if let Err(e) = p.join() {
                errors.push(format!("Failure in processor thread {e:?}"));
            }
        }
        if let Err(e) = thread_read1.join() {
            errors.push(format!("Failure in read1 thread {e:?}"));
        }
        if let Some(thread_read2) = thread_read2 {
            if let Err(e) = thread_read2.join() {
                errors.push(format!("Failure in read2 thread {e:?}"));
            }
        }
        if let Some(thread_index1) = thread_index1 {
            if let Err(e) = thread_index1.join() {
                errors.push(format!("Failure in index1 thread {e:?}"));
            }
        }
        if let Some(thread_index2) = thread_index2 {
            if let Err(e) = thread_index2.join() {
                errors.push(format!("Failure in index2 thread {e:?}"));
            }
        }
        if !errors.is_empty() {
            panic!("Error in threads occured: {:?}", errors);
        }

        //ok all this needs is a buffer that makes sure we reorder correctly at the end.
        //and then something block based, not single reads to pass between the threads.
        drop(parsed);
    }

   Ok(())
}

fn handle_stage(
    block: (usize, io::FastQBlocksCombined),
    output_tx2: &crossbeam::channel::Sender<(usize, io::FastQBlocksCombined)>,
    premature_termination_signaled: &Arc<AtomicBool>,
    stage: &mut Stage,
) -> bool {
    let mut out_block = block.1;
    let mut do_continue = true;
    let mut stage_continue;
    for stage in stage.transforms.iter_mut() {
        (out_block, stage_continue) = stage.transform(out_block, block.0);
        do_continue = do_continue && stage_continue;
    }
    match output_tx2.send((block.0, out_block)) {
        Ok(_) => {}
        Err(e) => {
            if premature_termination_signaled.load(std::sync::atomic::Ordering::Relaxed) {
                return false;
            } else {
                panic!("Error sending combined block to next stage: {:?}", e);
            }
        }
    };
    if !do_continue {
        if !stage.needs_serial {
            panic!("Non serial stages must not return do_continue = false")
        }
        premature_termination_signaled.store(true, std::sync::atomic::Ordering::Relaxed);
        return false;
    }
    true
}

fn output_block(block: io::FastQBlocksCombined, output_files: &mut OutputFiles) {
    let buffer_size = 1024 * 1024 * 10;
    let mut buffer = Vec::with_capacity(buffer_size);
    output_block_inner(
        output_files.read1.as_mut(),
        Some(&block.block_read1),
        &mut buffer,
        buffer_size,
        &mut output_files.hashers[0],
    );
    output_block_inner(
        output_files.read2.as_mut(),
        block.block_read2.as_ref(),
        &mut buffer,
        buffer_size,
        &mut output_files.hashers[1],
    );
    output_block_inner(
        output_files.index1.as_mut(),
        block.block_index1.as_ref(),
        &mut buffer,
        buffer_size,
        &mut output_files.hashers[2],
    );
    output_block_inner(
        output_files.index2.as_mut(),
        block.block_index2.as_ref(),
        &mut buffer,
        buffer_size,
        &mut output_files.hashers[3],
    );
}

fn output_block_inner<'a>(
    output_file: Option<&mut Writer<'a>>,
    block: Option<&io::FastQBlock>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    hasher: &mut Option<sha2::Sha256>,
) {
    if let Some(of) = output_file {
        let mut pseudo_iter = block.unwrap().get_pseudo_iter();
        while let Some(read) = pseudo_iter.next() {
            read.append_as_fastq(buffer);
            if buffer.len() > buffer_size {
                of.write_all(&buffer).unwrap();
                if let Some(hasher) = hasher {
                    hasher.update(&buffer);
                }
                buffer.clear();
            }
        }
        if let Some(hasher) = hasher {
            hasher.update(&buffer);
        }

        of.write_all(&buffer).unwrap();
    }
    buffer.clear()
}

fn format_seconds_to_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}
