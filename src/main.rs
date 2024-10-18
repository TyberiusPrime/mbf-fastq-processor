#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use bstr::BString;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::{fmt, marker::PhantomData, process::Output};

mod config;
use config::{check_config, Config, ConfigInput, ConfigOutput, OutputFormat};

struct FastQRead {
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

struct Molecule {
    read1: FastQRead,
    read2: Option<FastQRead>,
    index1: Option<FastQRead>,
    index2: Option<FastQRead>,
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
    Gzip(flate2::read::GzDecoder<BufReader<std::fs::File>>),
}

enum Writer {
    Raw(BufWriter<std::fs::File>),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Writer::Raw(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Writer::Raw(inner) => inner.flush(),
        }
    }
}

impl Reader {
    fn next_read(&mut self) -> Option<FastQRead> {
        match self {
            Reader::Raw(reader) => {
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
                    panic!("Truncated fastq file")
                }

                Some(FastQRead {
                    name: line1,
                    seq: line2,
                    qual: line4,
                })
            }
            Reader::Gzip(_) => todo!(),
        }
    }
}

struct InputSet {
    read1: Reader,
    read2: Option<Reader>,
    index1: Option<Reader>,
    index2: Option<Reader>,
}

impl Iterator for &mut InputSet {
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
    let fh = std::fs::File::open(path).context("Could not open file.")?;
    let bufreader = BufReader::new(fh);
    let reader = Reader::Raw(bufreader);
    Ok(reader)
}

fn open_input_files(parsed_config: &Config) -> Result<InputFiles> {
    let mut sets = Vec::new();
    for (ii, read1_filename) in (&parsed_config.input.read1).into_iter().enumerate() {
        // we can assume all the others are either of the same length, or None
        let read1 = open_file(&read1_filename)?;
        let read2 = parsed_config
            .input
            .read2
            .as_ref()
            .map(|x| open_file(&x[ii]));
        //bail if it's an Error
        let read2 = match read2 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        let index1 = parsed_config
            .input
            .index1
            .as_ref()
            .map(|x| open_file(&x[ii]));
        let index1 = match index1 {
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        let index2 = parsed_config
            .input
            .index2
            .as_ref()
            .map(|x| open_file(&x[ii]));
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
struct OutputFiles {
    read1: Option<Writer>,
    read2: Option<Writer>,
    index1: Option<Writer>,
    index2: Option<Writer>,
    reports: Vec<Writer>,
    inspects: Vec<(
        Option<Writer>,
        Option<Writer>,
        Option<Writer>,
        Option<Writer>,
    )>,
}

fn open_output_files(parsed_config: &Config) -> Result<OutputFiles> {
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let suffix =
                output_config
                    .suffix
                    .as_deref()
                    .unwrap_or_else(|| match output_config.format {
                        OutputFormat::Raw => ".fq",
                        OutputFormat::Gzip => ".gz",
                        OutputFormat::Zstd => ".zst",
                    });
            let (read1, read2, index1, index2) = match output_config.format {
                OutputFormat::Raw => {
                    let read1 = Some(Writer::Raw(BufWriter::new(std::fs::File::create(
                        format!("{}_1{}", output_config.prefix, suffix),
                    )?)));
                    let read2 = match parsed_config.input.read2 {
                        Some(_) => Some(Writer::Raw(BufWriter::new(std::fs::File::create(
                            format!("{}_2{}", output_config.prefix, suffix),
                        )?))),
                        None => None,
                    };
                    let (index1, index2) = if output_config.keep_index {
                        (
                            Some(Writer::Raw(BufWriter::new(std::fs::File::create(
                                format!("{}_i1{}", output_config.prefix, suffix),
                            )?))),
                            Some(Writer::Raw(BufWriter::new(std::fs::File::create(
                                format!("{}_i2{}", output_config.prefix, suffix),
                            )?))),
                        )
                    } else {
                        (None, None)
                    };
                    (read1, read2, index1, index2)
                }
                _ => todo!(),
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
fn read_read(reader: &mut Reader) -> FastQRead {
    match reader {
        Reader::Raw(reader) => {
            //todo: these ain't utf8.
            //and I need to cut off teh @ from the name
            let mut line1 = String::new();
            let mut line2 = String::new();
            let mut line3 = String::new();
            let mut line4 = String::new();
            reader
                .read_line(&mut line1)
                .expect("Could not read line 1.");
            reader
                .read_line(&mut line2)
                .expect("Could not read line 2.");
            reader.read_line(&mut line3).expect("Could not read line.");
            reader
                .read_line(&mut line4)
                .expect("Could not read line 4.");
            let name = line1.into_bytes();
            let seq = line2.into_bytes();
            let qual = line4.into_bytes();

            FastQRead { name, seq, qual }
        }
        Reader::Gzip(_) => todo!(),
    }
}

struct Work {
    block_no: usize,
    block: Vec<Molecule>,
}

use itertools::Itertools;
fn main() -> Result<()> {
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let raw_config = ex::fs::read_to_string(toml_file).context("Could not read toml file.")?;
    let parsed = toml::from_str::<Config>(&raw_config).context("Could not parse toml file.")?;
    check_config(&parsed)?;
    let mut input_files = open_input_files(&parsed)?;
    let mut output_files = open_output_files(&parsed)?;

    use crossbeam::channel::bounded;
    let (input_tx, input_rx) = bounded(50);
    let (output_tx, output_rx) = bounded(50);

    let thread_count = 50;
    let block_size = 10;
    let reader = thread::spawn(move || {
        let mut block_no = 1;
        for block in &(&mut input_files).chunks(block_size) {
            let block: Vec<_> = block.collect();
            input_tx
                .send(Work {
                    block_no: block_no,
                    block,
                })
                .unwrap();
            block_no += 1;
        }
    });
    for ii in 0..thread_count {
        let input_rx2 = input_rx.clone();
        let output_tx2 = output_tx.clone();
        let processor = thread::spawn(move || loop {
            let mut rng = rand::thread_rng();
            use rand::Rng;
            match input_rx2.recv() {
                Ok(block) => {
                    std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(0..50)));
                    output_tx2.send(block).unwrap();
                }
                Err(e) => {
                    return;
                }
            }
        });
    }
    drop(output_tx); //we must not hold a reference here across the join at the end
    let output = thread::spawn(move || {
        let mut last_block_outputted = 0;
        let mut buffer = Vec::new();
        loop {
            match output_rx.recv() {
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
    let _ = output.join();
    //ok all this needs is a buffer that makes sure we reorder correctly at the end.
    //and then something block based, not single reads to pass between the threads.

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

/*

    ///I need an iterator over all 4 inputs.
    ///before that, I need to decompress...
    ///then I need to do all the 'transformation'
    ///and throw it into the outputs
    ///all of this preferentially streaming, buffered multi threaded...
    Ok(())
*/
