#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use bstr::BString;
use flate2::write::GzEncoder;
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Seek, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::{fmt, marker::PhantomData, process::Output};

mod config;
mod transformations;
use config::{check_config, Config, ConfigInput, ConfigOutput, FileFormat};

#[derive(Clone)]
pub struct FastQRead {
    name: Vec<u8>,
    seq: Vec<u8>,
    qual: Vec<u8>,
}

impl FastQRead {
    fn fo_fastq(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(b'@');
        out.extend_from_slice(&self.name);
        out.push(b'\n');
        out.extend_from_slice(&self.seq);
        out.push(b'\n');
        out.push(b'+');
        out.push(b'\n');
        out.extend_from_slice(&self.qual);
        out.push(b'\n');
        out
    }

    fn cut_start(&self, n: usize) -> Self {
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[n..].to_vec(),
            qual: self.qual[n..].to_vec(),
        }
    }

    fn cut_end(&self, n: usize) -> Self {
        let remaining = (self.seq.len() as isize - n as isize).max(0) as usize;
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    fn max_len(&self, n: usize) -> Self {
        let remaining = self.seq.len().min(n);
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    fn prefix(&self, seq: &[u8], qual: &Vec<u8>) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&seq);
        new_seq.extend_from_slice(&self.seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&qual);
        new_qual.extend_from_slice(&self.qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }

    fn postfix(&self, seq: &[u8], qual: &Vec<u8>) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&self.seq);
        new_seq.extend_from_slice(&seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&self.qual);
        new_qual.extend_from_slice(&qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }
}

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
    read1: FastQRead,
    read2: Option<FastQRead>,
    index1: Option<FastQRead>,
    index2: Option<FastQRead>,
}

impl Molecule {
    fn replace_read1(&self, read1: FastQRead) -> Molecule {
        Molecule {
            read1,
            read2: self.read2.clone(),
            index1: self.index1.clone(),
            index2: self.index2.clone(),
        }
    }

    fn replace_read2(&self, read2: Option<FastQRead>) -> Molecule {
        Molecule {
            read1: self.read1.clone(),
            read2: read2,
            index1: self.index1.clone(),
            index2: self.index2.clone(),
        }
    }
    fn replace_index1(&self, index1: Option<FastQRead>) -> Molecule {
        Molecule {
            read1: self.read1.clone(),
            read2: self.read2.clone(),
            index1,
            index2: self.index2.clone(),
        }
    }

    fn replace_index2(&self, index2: Option<FastQRead>) -> Molecule {
        Molecule {
            read1: self.read1.clone(),
            read2: self.read2.clone(),
            index1: self.index1.clone(),
            index2,
        }
    }
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

enum Reader {
    Raw(BufReader<std::fs::File>),
    Gzip(BufReader<flate2::read::GzDecoder<BufReader<std::fs::File>>>),
    Zstd(BufReader<zstd::stream::Decoder<'static, BufReader<std::fs::File>>>),
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

impl Reader {
    fn _next_read<T: BufRead>(reader: &mut T) -> Option<FastQRead> {
        let mut line1 = Vec::new();
        let mut line2 = Vec::new();
        let mut line3 = Vec::new();
        let mut line4 = Vec::new();
        let mut dummy: [u8; 1] = [0];
        match reader.read_exact(&mut dummy) {
            Ok(()) => (),
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => return None,
                _ => panic!("Problem reading fastq"),
            },
        }
        if dummy[0] != b'@' {}
        let more = reader
            .read_until(b'\n', &mut line1)
            .expect("Could not read line 1.");
        if more == 0 {
            panic!("File truncated");
        }
        reader
            .read_until(b'\n', &mut line2)
            .expect("Could not read line 2.");
        reader
            .read_until(b'\n', &mut line3) //we don't care about that one'
            .expect("Could not read line.");
        reader
            .read_until(b'\n', &mut line4)
            .expect("Could not read line 4.");
        line1.pop();
        line2.pop();
        line4.pop();

        if line2.len() != line4.len() {
            dbg!(&std::str::from_utf8(&line1));
            dbg!(&std::str::from_utf8(&line2));
            dbg!(&std::str::from_utf8(&line3));
            dbg!(&std::str::from_utf8(&line4));
            panic!("Truncated fastq file")
        }

        Some(FastQRead {
            name: line1,
            seq: line2,
            qual: line4,
        })
    }
    fn next_read(&mut self) -> Option<FastQRead> {
        match self {
            Reader::Raw(reader) => Reader::_next_read(reader),
            Reader::Gzip(reader) => Reader::_next_read(reader),
            Reader::Zstd(reader) => Reader::_next_read(reader),
        }
    }
}

struct InputSet {
    read1: Reader,
    read2: Option<Reader>,
    index1: Option<Reader>,
    index2: Option<Reader>,
}

impl<'a> Iterator for &mut InputSet {
    type Item = Molecule;

    fn next(&mut self) -> Option<Self::Item> {
        let read1 = self.read1.next_read();
        match read1 {
            Some(read1) => Some(Molecule {
                read1,
                read2: match &mut self.read2 {
                    Some(x) => x.next_read(),
                    None => None,
                },
                index1: match &mut self.index1 {
                    Some(x) => x.next_read(),
                    None => None,
                },
                index2: match &mut self.index2 {
                    Some(x) => x.next_read(),
                    None => None,
                },
            }),
            None => None,
        }
    }
}

struct InputFiles {
    sets: Vec<InputSet>,
    current: usize,
}

impl Iterator for &mut InputFiles {
    type Item = Molecule;

    fn next(&mut self) -> Option<Self::Item> {
        use std::iter::Iterator;
        match (&mut self.sets[self.current]).next() {
            Some(x) => Some(x),
            None => {
                self.current += 1;
                if self.current < self.sets.len() {
                    (&mut self.sets[self.current]).next()
                } else {
                    None
                }
            }
        }
    }
}

fn open_file(path: &str) -> Result<Reader> {
    let mut fh = std::fs::File::open(path).context("Could not open file.")?;
    let mut magic = [0; 4];
    fh.read_exact(&mut magic)?;
    fh.seek(std::io::SeekFrom::Start(0))?;
    let bufreader = BufReader::new(fh);
    if magic[0] == 0x1f && magic[1] == 0x8b {
        let gz = BufReader::new(flate2::read::GzDecoder::new(bufreader));
        Ok(Reader::Gzip(gz))
    } else if magic[0] == 0x28 && magic[1] == 0xb5 && magic[2] == 0x2f && magic[3] == 0xfd {
        let zstd = zstd::stream::Decoder::with_buffer(bufreader)?;
        Ok(Reader::Zstd(BufReader::new(zstd)))
    } else {
        let reader = Reader::Raw(bufreader);
        Ok(reader)
    }
}

fn open_input_files<'a>(input_config: crate::config::ConfigInput) -> Result<InputFiles> {
    let mut sets = Vec::new();
    for (ii, read1_filename) in (&input_config.read1).into_iter().enumerate() {
        // we can assume all the others are either of the same length, or None
        let read1 = open_file(&read1_filename)?;
        let read2 = input_config.read2.as_ref().map(|x| open_file(&x[ii]));
        //bail if it's an Error
        let read2 = match read2 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        let index1 = input_config.index1.as_ref().map(|x| open_file(&x[ii]));
        let index1 = match index1 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        let index2 = input_config.index2.as_ref().map(|x| open_file(&x[ii]));
        let index2 = match index2 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        sets.push(InputSet {
            read1,
            read2,
            index1,
            index2,
        });
    }

    Ok(InputFiles { sets, current: 0 })
}

#[derive(Default)]
struct OutputFiles<'a> {
    read1: Option<Writer<'a>>,
    read2: Option<Writer<'a>>,
    index1: Option<Writer<'a>>,
    index2: Option<Writer<'a>>,
    reports: Vec<Writer<'a>>,
    inspects: Vec<(
        Option<Writer<'a>>,
        Option<Writer<'a>>,
        Option<Writer<'a>>,
        Option<Writer<'a>>,
    )>,
}

fn open_raw_output_file<'a>(path: &str) -> Result<Writer<'a>> {
    let fh = std::fs::File::create(path).context("Could not open file.")?;
    let bufwriter = BufWriter::new(fh);
    Ok(Writer::Raw(bufwriter))
}

fn open_gzip_output_file<'a>(path: &str, level: flate2::Compression) -> Result<Writer<'a>> {
    let fh = std::fs::File::create(path).context("Could not open file.")?;
    let buf_writer = BufWriter::new(fh);
    let gz = GzEncoder::new(buf_writer, level);
    Ok(Writer::Gzip(gz))
}
fn open_zstd_output_file<'a>(path: &str, level: i32) -> Result<Writer<'a>> {
    let fh = std::fs::File::create(path).context("Could not open file.")?;
    let buf_writer = BufWriter::new(fh);
    let encoder = zstd::stream::Encoder::new(buf_writer, level)?;
    let encoder = encoder.auto_finish();
    Ok(Writer::Zstd(encoder))
}
fn open_output_files<'a>(parsed_config: &Config) -> Result<OutputFiles<'a>> {
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
                    });
            let (read1, read2, index1, index2) = match output_config.format {
                //todo: refactor
                FileFormat::Raw => {
                    let read1 = Some(open_raw_output_file(&format!(
                        "{}_1{}",
                        output_config.prefix, suffix
                    ))?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_raw_output_file(&format!(
                            "{}_2{}",
                            output_config.prefix, suffix
                        ))?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_raw_output_file(&format!(
                                "{}_i1{}",
                                output_config.prefix, suffix
                            ))?),
                            Some(open_raw_output_file(&format!(
                                "{}_i2{}",
                                output_config.prefix, suffix
                            ))?),
                        )
                    } else {
                        (None, None)
                    };
                    (read1, read2, index1, index2)
                }
                FileFormat::Gzip => {
                    let read1 = Some(open_gzip_output_file(
                        &format!("{}_1{}", output_config.prefix, suffix),
                        flate2::Compression::default(),
                    )?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_gzip_output_file(
                            &format!("{}_2{}", output_config.prefix, suffix),
                            flate2::Compression::default(),
                        )?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_gzip_output_file(
                                &format!("{}_i1{}", output_config.prefix, suffix),
                                flate2::Compression::default(),
                            )?),
                            Some(open_gzip_output_file(
                                &format!("{}_i2{}", output_config.prefix, suffix),
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
                        &format!("{}_1{}", output_config.prefix, suffix),
                        5,
                    )?);
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(open_zstd_output_file(
                            &format!("{}_2{}", output_config.prefix, suffix),
                            5,
                        )?),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(open_zstd_output_file(
                                &format!("{}_i1{}", output_config.prefix, suffix),
                                5,
                            )?),
                            Some(open_zstd_output_file(
                                &format!("{}_i2{}", output_config.prefix, suffix),
                                5,
                            )?),
                        )
                    } else {
                        (None, None)
                    };

                    (read1, read2, index1, index2)
                }
            };
            let reports = Vec::new();
            let inspects = Vec::new();
            //todo: open report files.
            OutputFiles {
                read1,
                read2,
                index1,
                index2,
                reports,
                inspects,
            }
        }
        None => OutputFiles::default(),
    })
}

struct Work {
    block_no: usize, //so we can enforce the order
    block: Vec<Molecule>,
}

/// Split into transforms we can do parallelized
/// and transforms taht
fn split_transforms_into_stages(
    transforms: &[transformations::Transformation],
) -> Vec<(Vec<transformations::Transformation>, bool)> {
    if transforms.is_empty() {
        return Vec::new();
    }
    let mut stages: Vec<(Vec<transformations::Transformation>, bool)> = Vec::new();
    let mut current_stage = Vec::new();
    let mut last = None;
    for transform in transforms {
        let need_serial = transform.needs_serial();
        if Some(need_serial) != last {
            if !current_stage.is_empty() {
                stages.push((current_stage, last.take().unwrap()));
            }
            last = Some(need_serial);
            current_stage = Vec::new();
        }
        current_stage.push(transform.clone());
    }
    stages.push((current_stage, last.take().unwrap()));
    stages
}

use itertools::Itertools;
fn main() -> Result<()> {
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let raw_config = ex::fs::read_to_string(toml_file).context("Could not read toml file.")?;
    let parsed = toml::from_str::<Config>(&raw_config).context("Could not parse toml file.")?;
    check_config(&parsed)?;
    {
        let mut input_files = open_input_files(parsed.input.clone())?;
        let mut output_files = open_output_files(&parsed)?;

        use crossbeam::channel::bounded;
        let channel_size = 50;

        let stages = split_transforms_into_stages(&parsed.transform);
        //dbg!(&stages);

        let channels: Vec<_> = (0..stages.len() + 1)
            .into_iter()
            .map(|_| {
                let (tx, rx) = bounded(channel_size);
                (tx, rx)
            })
            .collect();

        let block_size = parsed.options.block_size;
        let input_channel = channels[0].0.clone();
        let reader = thread::spawn(move || {
            let mut block_no = 1;
            for block in &(&mut input_files).chunks(block_size) {
                let block: Vec<_> = block.collect();
                input_channel.send(Work { block_no, block }).unwrap();
                block_no += 1;
            }
        });

        let thread_count = parsed.options.thread_count;
        for (stage_no, (stage, needs_serial)) in stages.into_iter().enumerate() {
            let local_thread_count = if needs_serial { 1 } else { thread_count };
            for thread_ii in 0..local_thread_count {
                let mut stage = stage.clone();
                let input_rx2 = channels[stage_no].1.clone();
                let output_tx2 = channels[stage_no + 1].0.clone();
                let processor = thread::spawn(move || loop {
                    match input_rx2.recv() {
                        Ok(block) => {
                            let mut out_block = block.block;
                            for stage in stage.iter_mut() {
                                out_block = stage.transform(out_block);
                            }
                            output_tx2
                                .send(Work {
                                    block_no: block.block_no,
                                    block: out_block,
                                })
                                .unwrap();
                        }
                        Err(e) => {
                            return;
                        }
                    }
                });
            }
        }

        let output_channel = (channels[channels.len() - 1]).1.clone();
        let output = thread::spawn(move || {
            let mut last_block_outputted = 0;
            let mut buffer = Vec::new();
            loop {
                match output_channel.recv() {
                    Ok(block) => {
                        buffer.push(block);
                        loop {
                            let mut send = None;
                            for (ii, block) in buffer.iter().enumerate() {
                                if block.block_no - 1 == last_block_outputted {
                                    last_block_outputted += 1;
                                    send = Some(ii);
                                    break;
                                }
                            }
                            if let Some(send_idx) = send {
                                let to_output = buffer.remove(send_idx);
                                output_block(to_output.block, &mut output_files);
                            } else {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        break;
                    }
                }
            }
        });
        drop(channels); //we must not hold a reference here across the join at the end
        let _ = output.join();
        let _ = reader.join();
        //ok all this needs is a buffer that makes sure we reorder correctly at the end.
        //and then something block based, not single reads to pass between the threads.
        drop(parsed);
    }
    Ok(())
}

fn output_block(block: Vec<Molecule>, output_files: &mut OutputFiles) {
    for molecule in block {
        if let Some(of) = &mut output_files.read1 {
            of.write_all(&molecule.read1.fo_fastq()).unwrap();
        }

        if let Some(of) = &mut output_files.read2 {
            of.write_all(&molecule.read2.unwrap().fo_fastq()).unwrap();
        }
        if let Some(of) = &mut output_files.index1 {
            of.write_all(&molecule.index1.unwrap().fo_fastq()).unwrap();
        }
        if let Some(of) = &mut output_files.index2 {
            of.write_all(&molecule.index2.unwrap().fo_fastq()).unwrap();
        }
    }
}
