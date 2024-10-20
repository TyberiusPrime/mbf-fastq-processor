use std::io::{BufRead, BufReader, ErrorKind, Read, Seek};
use ex::Wrapper;
use anyhow::{Result, Context};

use crate::{FastQRead, Molecule};

pub enum Reader {
    Raw(BufReader<std::fs::File>),
    Gzip(BufReader<flate2::read::GzDecoder<BufReader<std::fs::File>>>),
    Zstd(BufReader<zstd::stream::Decoder<'static, BufReader<std::fs::File>>>),
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
    pub fn next_read(&mut self) -> Option<FastQRead> {
        match self {
            Reader::Raw(reader) => Reader::_next_read(reader),
            Reader::Gzip(reader) => Reader::_next_read(reader),
            Reader::Zstd(reader) => Reader::_next_read(reader),
        }
    }
}

pub struct InputSet {
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

pub struct InputFiles {
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

pub fn open_file(path: &str) -> Result<Reader> {
    // it
    let mut fh = ex::fs::File::open(path).context("Could not open file.")?; //open it for real with the required type
    let mut magic = [0; 4];
    fh.read_exact(&mut magic)?;
    fh.seek(std::io::SeekFrom::Start(0))?;
    let bufreader = BufReader::new(fh.into_inner());
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

pub fn open_input_files<'a>(input_config: crate::config::ConfigInput) -> Result<InputFiles> {
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
