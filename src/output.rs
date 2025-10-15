/// Handles transparent compressed file writing
/// and optional hashing at both the compressed and uncompressed levels.
use flate2::write::GzEncoder;
use sha2::Digest;
use std::io::{self, BufWriter, Write};

use crate::config::FileFormat;
use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct SimulatedWriteFailure {
    pub remaining_bytes: Option<usize>,
    pub error: SimulatedWriteError,
}

impl SimulatedWriteFailure {
    fn into_writer<T: Write>(self, inner: T) -> FailForTestWriter<T> {
        FailForTestWriter::new(inner, self)
    }
}

#[derive(Clone, Debug)]
pub enum SimulatedWriteError {
    RawOs(i32),
    Other,
}

impl SimulatedWriteError {
    fn build_error(&self) -> io::Error {
        match self {
            SimulatedWriteError::RawOs(code) => io::Error::from_raw_os_error(*code),
            SimulatedWriteError::Other => {
                io::Error::new(io::ErrorKind::Other, "SimulatedFailure".to_string())
            }
        }
    }
}

struct FailForTestWriter<T: Write> {
    inner: T,
    remaining_bytes: Option<usize>,
    error: SimulatedWriteError,
    failure_emitted: bool,
}

impl<T: Write> FailForTestWriter<T> {
    fn new(inner: T, config: SimulatedWriteFailure) -> Self {
        FailForTestWriter {
            inner,
            remaining_bytes: config.remaining_bytes,
            error: config.error,
            failure_emitted: false,
        }
    }

    fn make_error(&self) -> io::Error {
        self.error.build_error()
    }

    fn finish(self) -> T {
        self.inner
    }
}

impl<T: Write> Write for FailForTestWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.failure_emitted {
            return Err(self.make_error());
        }

        if let Some(remaining) = self.remaining_bytes {
            if remaining == 0 {
                self.failure_emitted = true;
                return Err(self.make_error());
            }

            let allowed = remaining.min(buf.len());
            let written = self.inner.write(&buf[..allowed])?;
            let new_remaining = remaining.saturating_sub(written);
            self.remaining_bytes = Some(new_remaining);

            if allowed < buf.len() || written < allowed {
                self.failure_emitted = true;
                return if written > 0 {
                    Err(self.make_error())
                } else {
                    Err(self.make_error())
                };
            }

            if new_remaining == 0 {
                self.failure_emitted = true;
            }

            Ok(written)
        } else {
            self.inner.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

enum CompressedWriter<'a, T: Write> {
    Raw(HashingFileWriter<BufWriter<T>>),
    Gzip(GzEncoder<HashingFileWriter<BufWriter<T>>>),
    Zstd(zstd::stream::Encoder<'a, HashingFileWriter<BufWriter<T>>>),
}

impl<'a, T: Write> CompressedWriter<'a, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            CompressedWriter::Raw(inner) => inner,
            CompressedWriter::Gzip(inner) => inner.finish().unwrap(),
            CompressedWriter::Zstd(inner) => inner.finish().unwrap(),
        }
    }
}

impl<'a, T: Write> Write for CompressedWriter<'a, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CompressedWriter::Raw(inner) => inner.write(buf),
            CompressedWriter::Gzip(inner) => inner.write(buf),
            CompressedWriter::Zstd(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            CompressedWriter::Raw(inner) => inner.flush(),
            CompressedWriter::Gzip(inner) => inner.flush(),
            CompressedWriter::Zstd(inner) => inner.flush(),
        }
    }
}

enum Compressed<'a, T: Write> {
    Normal(CompressedWriter<'a, T>),
    FailForTest(FailForTestWriter<CompressedWriter<'a, T>>),
}

impl<'a, T: Write> Compressed<'a, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            Compressed::Normal(inner) => inner.finish(),
            Compressed::FailForTest(inner) => inner.finish().finish(),
        }
    }
}

impl<'a, T: Write> Write for Compressed<'a, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Compressed::Normal(inner) => inner.write(buf),
            Compressed::FailForTest(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Compressed::Normal(inner) => inner.flush(),
            Compressed::FailForTest(inner) => inner.flush(),
        }
    }
}

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
        failure: Option<SimulatedWriteFailure>,
    ) -> Result<Self> {
        let mut compressed_hasher = if hash_compressed {
            Some(sha2::Sha256::new())
        } else {
            None
        };
        let mut uncompressed_hasher = if hash_uncompressed {
            Some(sha2::Sha256::new())
        } else {
            None
        };

        let base_writer = match compression_format {
            FileFormat::Raw | FileFormat::Bam => {
                let file_writer = BufWriter::new(writer);
                CompressedWriter::Raw(HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher.take(),
                })
            }
            FileFormat::Gzip => {
                let file_writer = BufWriter::new(writer);
                let compression = match compression_level {
                    Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                    None => flate2::Compression::default(),
                };
                CompressedWriter::Gzip(GzEncoder::new(
                    HashingFileWriter {
                        file_writer,
                        hasher: compressed_hasher.take(),
                    },
                    compression,
                ))
            }
            FileFormat::Zstd => {
                let file_writer = BufWriter::new(writer);
                let level = i32::from(compression_level.unwrap_or(5)).clamp(1, 22);
                CompressedWriter::Zstd(
                    zstd::stream::Encoder::new(
                        HashingFileWriter {
                            file_writer,
                            hasher: compressed_hasher.take(),
                        },
                        level,
                    )
                    .context("Failed to create zstd encoder")?,
                )
            }
            FileFormat::None => unreachable!(),
        };

        let compressed = match failure {
            Some(failure_cfg) => Compressed::FailForTest(failure_cfg.into_writer(base_writer)),
            None => Compressed::Normal(base_writer),
        };

        let compressed_writer = HashingFileWriter {
            file_writer: compressed,
            hasher: uncompressed_hasher.take(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FileFormat;
    use std::io::{self, Cursor, Write};

    #[test]
    fn fail_for_test_writer_errors_after_budget() -> io::Result<()> {
        let cursor = Cursor::new(Vec::new());
        let failure = SimulatedWriteFailure {
            remaining_bytes: Some(4),
            error: SimulatedWriteError::Other,
        };

        let mut writer = HashedAndCompressedWriter::new(
            cursor,
            FileFormat::Raw,
            false,
            false,
            None,
            Some(failure),
        )
        .expect("create writer");

        writer.write_all(b"abcd")?;
        let err = writer
            .write(b"efg")
            .expect_err("should fail after budget is exhausted");
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert!(err.to_string().contains("SimulatedFailure"));
        let subsequent = writer
            .write(b"h")
            .expect_err("subsequent writes must keep failing");
        assert_eq!(subsequent.kind(), io::ErrorKind::Other);

        let _ = writer.finish();

        Ok(())
    }
}
