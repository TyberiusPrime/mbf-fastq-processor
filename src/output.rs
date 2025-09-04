/// Handles transparent compressed file writing
/// and optional hashing at both the compressed and uncompressed levels.
use flate2::write::GzEncoder;
use sha2::Digest;
use std::io::{BufWriter, Write};

use crate::config::FileFormat;
use anyhow::{Context, Result};

pub struct HashedAndCompressedWriter<'a, T: std::io::Write> {
    compressed_writer: HashingFileWriter<Compressed<'a, T>>,
}

impl<T: std::io::Write> HashedAndCompressedWriter<'_, T> {
    pub fn new(
        writer: T,
        compression_format: FileFormat,
        hash_uncompressed: bool,
        hash_compressed: bool,
        compression_level: Option<u8>,
    ) -> Result<Self> {
        let compressed_hasher = if hash_compressed {
            Some(sha2::Sha256::new())
        } else {
            None
        };
        let uncompressed_hasher = if hash_uncompressed {
            Some(sha2::Sha256::new())
        } else {
            None
        };

        let compressed_writer = match compression_format {
            FileFormat::Raw => {
                let file_writer = BufWriter::new(writer);
                Compressed::Raw(HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher,
                })
            }
            FileFormat::Gzip => {
                let file_writer = BufWriter::new(writer);
                let compression = match compression_level {
                    Some(level) => flate2::Compression::new((level as u32).clamp(0, 9)),
                    None => flate2::Compression::default(),
                };
                Compressed::Gzip(GzEncoder::new(
                    HashingFileWriter {
                        file_writer,
                        hasher: compressed_hasher,
                    },
                    compression,
                ))
            }
            FileFormat::Zstd => {
                let file_writer = BufWriter::new(writer);
                let level = (compression_level.unwrap_or(5) as i32).clamp(1, 22);
                Compressed::Zstd(
                    zstd::stream::Encoder::new(
                        HashingFileWriter {
                            file_writer,
                            hasher: compressed_hasher,
                        },
                        level,
                    )
                    .context("Failed to create zstd encoder")?,
                )
            }
            FileFormat::None => unreachable!(),
        };
        let compressed_writer = HashingFileWriter {
            file_writer: compressed_writer,
            hasher: uncompressed_hasher,
        };

        Ok(Self { compressed_writer })
    }

    pub fn finish(self) -> (Option<String>, Option<String>) {
        let (uncompressed_hasher, inner) = self.compressed_writer.finish().unwrap();
        let inner_hashingwriter = inner.finish();
        let (compressed_hasher, _filehandle) = inner_hashingwriter.finish().unwrap();

        let uncompressed_hash =
            uncompressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));

        let compressed_hash = compressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));
        (uncompressed_hash, compressed_hash)
    }
}

impl<T: std::io::Write> std::io::Write for HashedAndCompressedWriter<'_, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.compressed_writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.compressed_writer.flush()
    }
}

/// Writes to an inner Writer and calculates the hash on the written data
struct HashingFileWriter<T: std::io::Write> {
    file_writer: T,
    hasher: Option<sha2::Sha256>,
}

impl<T: std::io::Write> HashingFileWriter<T> {
    fn finish(mut self) -> Result<(Option<sha2::Sha256>, T)> {
        self.file_writer.flush()?;
        Ok((self.hasher, self.file_writer))
    }
}

impl<T: std::io::Write> std::io::Write for HashingFileWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        //already compressed.
        if let Some(hasher) = &mut self.hasher {
            hasher.update(buf);
        }
        self.file_writer.write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file_writer.flush()
    }
}

enum Compressed<'a, T: std::io::Write> {
    Raw(HashingFileWriter<BufWriter<T>>),
    Gzip(GzEncoder<HashingFileWriter<BufWriter<T>>>),
    Zstd(zstd::stream::Encoder<'a, HashingFileWriter<BufWriter<T>>>),
}

impl<T: std::io::Write> Compressed<'_, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            Compressed::Raw(inner) => inner,
            Compressed::Gzip(inner) => inner.finish().unwrap(),
            Compressed::Zstd(inner) => inner.finish().unwrap(),
        }
    }
}

impl<T: std::io::Write> Write for Compressed<'_, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Compressed::Raw(inner) => inner.write(buf),
            Compressed::Gzip(inner) => inner.write(buf),
            Compressed::Zstd(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Compressed::Raw(inner) => inner.flush(),
            Compressed::Gzip(inner) => inner.flush(),
            Compressed::Zstd(inner) => inner.flush(),
        }
    }
}
