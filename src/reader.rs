use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, BufRead};

pub enum Reader {
    Raw(BufReader<File>),
    Gzip(GzDecoder<File>),
}

impl Reader {
    pub fn new<P>(path: P) -> io::Result<Self>
    where
        P: Into<std::path::PathBuf>,
    {
        let file = File::open(path)?;
        Ok(Self::create_reader(file)?)
    }

    fn create_reader(mut file: File) -> io::Result<Self> {
        let mut magic_bytes = [0u8; 2];
        file.read_exact(&mut magic_bytes)?;

        match &magic_bytes {
            b"\x1F\x8B" => {
                // Gzip header
                let decoder = GzDecoder::new(file);
                Ok(Reader::Gzip(decoder))
            }
            _ => {
                // Raw file
                Ok(Reader::Raw(BufReader::with_capacity(1024 * 1024, file)))
            }
        }
    }
}

impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            Reader::Raw(reader) => reader.fill_buf(),
            Reader::Gzip(decoder) => decoder.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Reader::Raw(reader) => reader.consume(amt),
            Reader::Gzip(decoder) => decoder.consume(amt),
        }
    }
}

pub struct LineIterator {
    reader: Reader,
}

impl LineIterator {
    pub fn new<P>(path: P) -> io::Result<Self>
    where
        P: Into<std::path::PathBuf>,
    {
        Ok(Self {
            reader: Reader::new(path)?,
        })
    }
}

impl Iterator for LineIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        self.reader.read_line(&mut line).ok()?;
        if line.is_empty() {
            None
        } else {
            Some(line.trim().to_string())
        }
    }
}
