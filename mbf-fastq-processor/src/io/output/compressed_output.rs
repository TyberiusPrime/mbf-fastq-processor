/// Handles transparent compressed file writing
/// and optional hashing at both the compressed and uncompressed levels.
use flate2::write::GzEncoder;
use gzp::{ZBuilder, ZWriter, deflate::Gzip};
use sha2::Digest;
use std::io::{self, BufWriter, Write};

use crate::config::CompressionFormat;
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
            SimulatedWriteError::Other => io::Error::other("SimulatedFailure".to_string()),
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
            if buf.len() > allowed {
                self.failure_emitted = true;
                return Err(self.make_error());
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

/// Wrapper for gzp's parallel writer to implement Send
/// SAFETY: gzp parallel writers are internally thread-safe but don't implement Send
/// due to trait object limitations. This wrapper provides the Send impl.
struct SendableParallelWriter<T: Write>(Box<dyn ZWriter<T>>);

impl<T: Write> Write for SendableParallelWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<T: Write> ZWriter<T> for SendableParallelWriter<T> {
    fn finish(&mut self) -> Result<T, gzp::GzpError> {
        self.0.finish()
    }
}

enum CompressedWriter<'a, T: Write + Send + 'static> {
    Raw(HashingFileWriter<BufWriter<T>>),
    GzipSingle(GzEncoder<HashingFileWriter<BufWriter<T>>>),
    GzipParallel(SendableParallelWriter<HashingFileWriter<BufWriter<T>>>),
    Zstd(zstd::stream::Encoder<'a, HashingFileWriter<BufWriter<T>>>),
}

enum CompressedWriterSingleCore<'a, T: Write + Send + 'static> {
    Raw(HashingFileWriter<BufWriter<T>>),
    GzipSingle(GzEncoder<HashingFileWriter<BufWriter<T>>>),
    //because gzp is not Send.
    Zstd(zstd::stream::Encoder<'a, HashingFileWriter<BufWriter<T>>>),
}

impl<T: Write + Send + 'static> CompressedWriter<'_, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            CompressedWriter::Raw(inner) => inner,
            CompressedWriter::GzipSingle(inner) => inner
                .finish()
                .expect("compression finalization should not fail"),
            CompressedWriter::GzipParallel(mut inner) => inner
                .finish()
                .expect("compression finalization should not fail"),
            CompressedWriter::Zstd(inner) => inner
                .finish()
                .expect("compression finalization should not fail"),
        }
    }
}

impl<T: Write + Send + 'static> Write for CompressedWriter<'_, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CompressedWriter::Raw(inner) => inner.write(buf),
            CompressedWriter::GzipSingle(inner) => inner.write(buf),
            CompressedWriter::GzipParallel(inner) => inner.write(buf),
            CompressedWriter::Zstd(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            CompressedWriter::Raw(inner) => inner.flush(),
            CompressedWriter::GzipSingle(inner) => inner.flush(),
            CompressedWriter::GzipParallel(inner) => inner.flush(),
            CompressedWriter::Zstd(inner) => inner.flush(),
        }
    }
}

impl<T: Write + Send + 'static> CompressedWriterSingleCore<'_, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            CompressedWriterSingleCore::Raw(inner) => inner,
            CompressedWriterSingleCore::GzipSingle(inner) => inner
                .finish()
                .expect("compression finalization should not fail"),
            CompressedWriterSingleCore::Zstd(inner) => inner
                .finish()
                .expect("compression finalization should not fail"),
        }
    }
}

impl<T: Write + Send + 'static> Write for CompressedWriterSingleCore<'_, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CompressedWriterSingleCore::Raw(inner) => inner.write(buf),
            CompressedWriterSingleCore::GzipSingle(inner) => inner.write(buf),
            CompressedWriterSingleCore::Zstd(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            CompressedWriterSingleCore::Raw(inner) => inner.flush(),
            CompressedWriterSingleCore::GzipSingle(inner) => inner.flush(),
            CompressedWriterSingleCore::Zstd(inner) => inner.flush(),
        }
    }
}

enum Compressed<'a, T: Write + Send + 'static> {
    Normal(CompressedWriter<'a, T>),
    FailForTest(FailForTestWriter<CompressedWriter<'a, T>>),
}

impl<T: Write + Send + 'static> Compressed<'_, T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            Compressed::Normal(inner) => inner.finish(),
            Compressed::FailForTest(inner) => inner.finish().finish(),
        }
    }
}

impl<T: Write + Send + 'static> Write for Compressed<'_, T> {
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

pub struct HashedAndCompressedWriter<'a, T: std::io::Write + Send + 'static> {
    compressed_writer: HashingFileWriter<Compressed<'a, T>>,
}

pub struct HashedAndCompressedWriterSingleCore<'a, T: std::io::Write + Send + 'static> {
    compressed_writer: HashingFileWriter<CompressedWriterSingleCore<'a, T>>,
}

impl<T: std::io::Write + Send + 'static> HashedAndCompressedWriter<'_, T> {
    pub fn new(
        writer: T,
        compression_format: CompressionFormat,
        hash_uncompressed: bool,
        hash_compressed: bool,
        compression_level: Option<u8>,
        compression_threads: Option<usize>,
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
            CompressionFormat::Uncompressed => {
                let file_writer = BufWriter::new(writer);
                CompressedWriter::Raw(HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher.take(),
                })
            }
            CompressionFormat::Gzip => {
                let file_writer = BufWriter::new(writer);
                let hashing_writer = HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher.take(),
                };

                // Use parallel compression if threads > 1, otherwise use single-threaded
                if let Some(threads) = compression_threads {
                    if threads > 1 {
                        // Use real multi-threaded gzip compression with gzp
                        let mut builder = ZBuilder::<Gzip, _>::new().num_threads(threads);

                        // Set compression level if provided
                        builder = builder.compression_level(match compression_level {
                            Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                            None => flate2::Compression::default(),
                        });

                        let parallel_writer = builder.from_writer(hashing_writer);
                        let sendable_writer = SendableParallelWriter(parallel_writer);
                        CompressedWriter::GzipParallel(sendable_writer)
                    } else {
                        // Single threaded fallback
                        let compression = match compression_level {
                            Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                            None => flate2::Compression::default(),
                        };
                        CompressedWriter::GzipSingle(GzEncoder::new(hashing_writer, compression))
                    }
                } else {
                    // Default to single threaded when threads not specified
                    let compression = match compression_level {
                        Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                        None => flate2::Compression::default(),
                    };
                    CompressedWriter::GzipSingle(GzEncoder::new(hashing_writer, compression))
                }
            }
            CompressionFormat::Zstd => {
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
        let (uncompressed_hasher, inner) = self
            .compressed_writer
            .finish()
            .expect("writer finalization should not fail");
        let inner_hashingwriter = inner.finish();
        let (compressed_hasher, _filehandle) = inner_hashingwriter
            .finish()
            .expect("writer finalization should not fail");

        let uncompressed_hash =
            uncompressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));

        let compressed_hash = compressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));
        (uncompressed_hash, compressed_hash)
    }
}

impl<T: std::io::Write + Send + 'static> std::io::Write for HashedAndCompressedWriter<'_, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.compressed_writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.compressed_writer.flush()
    }
}

impl<T: std::io::Write + Send + 'static> HashedAndCompressedWriterSingleCore<'_, T> {
    pub fn new(
        writer: T,
        compression_format: CompressionFormat,
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
            CompressionFormat::Uncompressed => {
                let file_writer = BufWriter::new(writer);
                CompressedWriterSingleCore::Raw(HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher.take(),
                })
            }
            CompressionFormat::Gzip => {
                let file_writer = BufWriter::new(writer);
                let hashing_writer = HashingFileWriter {
                    file_writer,
                    hasher: compressed_hasher.take(),
                };

                // Use parallel compression if threads > 1, otherwise use single-threaded
                // Default to single threaded when threads not specified
                let compression = match compression_level {
                    Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                    None => flate2::Compression::default(),
                };
                CompressedWriterSingleCore::GzipSingle(GzEncoder::new(hashing_writer, compression))
            }
            CompressionFormat::Zstd => {
                let file_writer = BufWriter::new(writer);
                let level = i32::from(compression_level.unwrap_or(5)).clamp(1, 22);
                CompressedWriterSingleCore::Zstd(
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
        };

        assert!(
            failure.is_none(),
            "failure testing only implementd for multi core variant"
        );
        // let compressed = match failure {
        //     Some(failure_cfg) => Compressed::FailForTest(failure_cfg.into_writer(base_writer)),
        //     None => Compressed::Normal(base_writer),
        // };

        let compressed_writer = HashingFileWriter {
            file_writer: base_writer,
            hasher: uncompressed_hasher.take(),
        };

        Ok(Self { compressed_writer })
    }

    pub fn finish(self) -> (Option<String>, Option<String>) {
        let (uncompressed_hasher, inner) = self
            .compressed_writer
            .finish()
            .expect("writer finalization should not fail");
        let inner_hashingwriter = inner.finish();
        let (compressed_hasher, _filehandle) = inner_hashingwriter
            .finish()
            .expect("writer finalization should not fail");

        let uncompressed_hash =
            uncompressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));

        let compressed_hash = compressed_hasher.map(|hasher| format!("{:x}", hasher.finalize()));
        (uncompressed_hash, compressed_hash)
    }
}

impl<T: std::io::Write + Send + 'static> std::io::Write
    for HashedAndCompressedWriterSingleCore<'_, T>
{
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
    use crate::config::CompressionFormat;
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
            CompressionFormat::Uncompressed,
            false,
            false,
            None,
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
    #[test]
    fn fail_for_test_writer_errors_after_budget_single_write() -> io::Result<()> {
        let cursor = Cursor::new(Vec::new());
        let failure = SimulatedWriteFailure {
            remaining_bytes: Some(4),
            error: SimulatedWriteError::Other,
        };

        let mut writer = HashedAndCompressedWriter::new(
            cursor,
            CompressionFormat::Uncompressed,
            false,
            false,
            None,
            None,
            Some(failure),
        )
        .expect("create writer");

        let err = writer
            .write(b"abcde")
            .expect_err("should fail after budget is exhausted, even in one read");
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
