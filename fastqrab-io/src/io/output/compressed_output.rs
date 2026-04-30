use anyhow::{Context, Result};
/// Handles transparent compressed file writing
/// and optional hashing at both the compressed and uncompressed levels.
use flate2::write::GzEncoder;
use gzp::{
    ZWriter,
    deflate::Gzip,
    par::compress::{ParCompress, ParCompressBuilder},
};
use sha2::Digest;
use std::io::{self, BufWriter, Write};

use crate::CompressionFormat;

#[derive(Clone, Debug)]
pub struct SimulatedWriteFailure {
    pub remaining_bytes: usize,
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
    remaining_bytes: usize,
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

        if self.remaining_bytes == 0 {
            self.failure_emitted = true;
            return Err(self.make_error());
        }

        let allowed = self.remaining_bytes.min(buf.len());
        let written = self.inner.write(&buf[..allowed])?;
        let new_remaining = self.remaining_bytes.saturating_sub(written);
        self.remaining_bytes = new_remaining;
        if buf.len() > allowed {
            self.failure_emitted = true;
            return Err(self.make_error());
        }

        Ok(written)
    }

    #[mutants::skip]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Thin wrapper around gzp's concrete parallel writer type.
/// `Send` is derived automatically because all fields of `ParCompress` are `Send`.
struct ParallelWriter<T: Write + Send + 'static>(ParCompress<'static, Gzip, T>);

impl<T: Write + Send + 'static> Write for ParallelWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<T: Write + Send + 'static> ZWriter<T> for ParallelWriter<T> {
    fn finish(&mut self) -> Result<T, gzp::GzpError> {
        self.0.finish()
    }
}

enum CompressedWriter<T: Write + Send + 'static> {
    Raw(HashingFileWriter<BufWriter<T>>),
    GzipSingle(GzEncoder<HashingFileWriter<BufWriter<T>>>),
    GzipParallel(ParallelWriter<HashingFileWriter<BufWriter<T>>>),
    Zstd(zstd::stream::Encoder<'static, HashingFileWriter<BufWriter<T>>>),
}

impl<T: Write + Send + 'static> CompressedWriter<T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            CompressedWriter::Raw(inner) => inner,
            CompressedWriter::GzipSingle(inner) => inner
                .finish()
                .expect("Compression finalization failed unexpectedly. Disk full or similar unrecoverable condition?"),
            CompressedWriter::GzipParallel(mut inner) => inner
                .finish()
                .expect("Compression finalization failed unexpectedly. Disk full or similar unrecoverable condition?"),
            CompressedWriter::Zstd(inner) => inner
                .finish()
                .expect("Compression finalization failed unexpectedly. Disk full or similar unrecoverable condition?"),
        }
    }
}

impl<T: Write + Send + 'static> Write for CompressedWriter<T> {
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

enum Compressed<T: Write + Send + 'static> {
    Normal(CompressedWriter<T>),
    FailForTest(FailForTestWriter<CompressedWriter<T>>),
}

impl<T: Write + Send + 'static> Compressed<T> {
    fn finish(self) -> HashingFileWriter<BufWriter<T>> {
        match self {
            Compressed::Normal(inner) => inner.finish(),
            Compressed::FailForTest(inner) => inner.finish().finish(),
        }
    }
}

impl<T: Write + Send + 'static> Write for Compressed<T> {
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

pub struct HashedAndCompressedWriter<T: std::io::Write + Send + 'static> {
    compressed_writer: HashingFileWriter<Compressed<T>>,
}

pub type OutputWriter = HashedAndCompressedWriter<ex::fs::File>;
// cov:excl-start
impl std::fmt::Debug for OutputWriter {
    #[mutants::skip] // don't care that it's never used, it' s useful when you need to debug
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputWriter").finish_non_exhaustive()
    }
}
// cov:excl-stop

impl<T: std::io::Write + Send + 'static> HashedAndCompressedWriter<T> {
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
                if let Some(threads) = compression_threads
                    && threads > 1
                {
                    // Use real multi-threaded gzip compression with gzp.
                    // Use ParCompressBuilder directly (not ZBuilder) to get the concrete
                    // ParCompress type, which auto-derives Send without unsafe.
                    let compression = match compression_level {
                        Some(level) => flate2::Compression::new(u32::from(level).clamp(0, 9)),
                        None => flate2::Compression::default(),
                    };
                    let parallel_writer = ParCompressBuilder::<Gzip>::new()
                        .num_threads(threads)?
                        .compression_level(compression)
                        .from_writer(hashing_writer);
                    let sendable_writer = ParallelWriter(parallel_writer);
                    CompressedWriter::GzipParallel(sendable_writer)
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

    /// # Panics
    /// if the hashing writer finish fails 
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

impl<T: std::io::Write + Send + 'static> std::io::Write for HashedAndCompressedWriter<T> {
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
    use crate::CompressionFormat;
    use std::io::{self, Cursor, Write};

    #[test]
    fn fail_for_test_writer_errors_after_budget() -> io::Result<()> {
        let cursor = Cursor::new(Vec::new());
        let failure = SimulatedWriteFailure {
            remaining_bytes: 4,
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
    fn fail_for_test_writer_errors_after_budget_single_write() {
        let cursor = Cursor::new(Vec::new());
        let failure = SimulatedWriteFailure {
            remaining_bytes: 4,
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

    }
}
