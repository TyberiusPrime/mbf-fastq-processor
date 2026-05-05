//! Test-only writer that fails after a configurable byte budget, plus the
//! configuration types describing such a simulated failure.
//!
//! Used by [`chunked_writer`](super::chunked_writer) to inject reproducible
//! write errors into the output pipeline during tests.

use std::io::{self, Write};

#[derive(Clone, Debug)]
pub struct SimulatedWriteFailure {
    pub remaining_bytes: usize,
    pub error: SimulatedWriteError,
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

pub(super) struct FailForTestWriter<T: Write> {
    inner: T,
    remaining_bytes: usize,
    error: SimulatedWriteError,
    failure_emitted: bool,
}

impl<T: Write> FailForTestWriter<T> {
    pub(super) fn new(inner: T, config: SimulatedWriteFailure) -> Self {
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

    pub(super) fn finish(self) -> T {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn fail_for_test_writer_errors_after_budget() -> io::Result<()> {
        let cursor = Cursor::new(Vec::new());
        let failure = SimulatedWriteFailure {
            remaining_bytes: 4,
            error: SimulatedWriteError::Other,
        };
        let mut writer = FailForTestWriter::new(cursor, failure);
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
        let mut writer = FailForTestWriter::new(cursor, failure);
        let err = writer
            .write(b"abcde")
            .expect_err("should fail after budget is exhausted, even in one write");
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert!(err.to_string().contains("SimulatedFailure"));
        let subsequent = writer
            .write(b"h")
            .expect_err("subsequent writes must keep failing");
        assert_eq!(subsequent.kind(), io::ErrorKind::Other);
        let _ = writer.finish();
    }
}
