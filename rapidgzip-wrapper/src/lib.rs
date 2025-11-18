mod ffi;

use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::io::RawFd;
use std::path::Path;
use std::ptr;

pub use ffi::{RapidGzipError, RapidGzipSeekMode};

/// A parallel gzip reader that wraps librapidgzip
pub struct ParallelGzipReader {
    reader: *mut ffi::RapidGzipReader,
}

impl ParallelGzipReader {
    /// Open a gzip file for parallel decompression
    ///
    /// # Arguments
    /// * `path` - Path to the gzip file
    /// * `num_threads` - Number of threads to use (0 = auto-detect)
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened or is not a valid gzip file
    pub fn open<P: AsRef<Path>>(path: P, num_threads: usize) -> Result<Self> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| anyhow!("Path contains invalid UTF-8"))?;
        let c_path = CString::new(path_str)
            .context("Failed to create CString from path")?;

        let mut reader: *mut ffi::RapidGzipReader = ptr::null_mut();

        let result = unsafe {
            ffi::rapidgzip_open(c_path.as_ptr(), num_threads, &mut reader)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to open gzip file: {:?}", error));
        }

        if reader.is_null() {
            return Err(anyhow!("Failed to create reader (null pointer)"));
        }

        Ok(Self { reader })
    }

    /// Open a gzip file from a file descriptor
    ///
    /// # Arguments
    /// * `fd` - File descriptor
    /// * `num_threads` - Number of threads to use (0 = auto-detect)
    ///
    /// # Errors
    /// Returns an error if the file descriptor is invalid or not a valid gzip file
    pub fn from_fd(fd: RawFd, num_threads: usize) -> Result<Self> {
        let mut reader: *mut ffi::RapidGzipReader = ptr::null_mut();

        let result = unsafe {
            ffi::rapidgzip_open_fd(fd, num_threads, &mut reader)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to open gzip file from fd: {:?}", error));
        }

        if reader.is_null() {
            return Err(anyhow!("Failed to create reader (null pointer)"));
        }

        Ok(Self { reader })
    }

    /// Get the current position in the decompressed stream
    ///
    /// # Errors
    /// Returns an error if the operation fails
    pub fn tell(&self) -> Result<u64> {
        let mut position: u64 = 0;

        let result = unsafe {
            ffi::rapidgzip_tell(self.reader, &mut position)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to get position: {:?}", error));
        }

        Ok(position)
    }

    /// Check if the reader has reached end-of-file
    ///
    /// # Errors
    /// Returns an error if the operation fails
    pub fn is_eof(&self) -> Result<bool> {
        let mut eof: i32 = 0;

        let result = unsafe {
            ffi::rapidgzip_eof(self.reader, &mut eof)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to check EOF: {:?}", error));
        }

        Ok(eof != 0)
    }

    /// Enable or disable CRC32 verification
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable CRC32 verification
    ///
    /// # Errors
    /// Returns an error if the operation fails
    pub fn set_crc32_enabled(&mut self, enabled: bool) -> Result<()> {
        let result = unsafe {
            ffi::rapidgzip_set_crc32_enabled(self.reader, if enabled { 1 } else { 0 })
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to set CRC32 mode: {:?}", error));
        }

        Ok(())
    }

    /// Get the total size of the decompressed data (if available)
    ///
    /// Returns `None` if the size is not yet determined.
    ///
    /// # Errors
    /// Returns an error if the operation fails
    pub fn size(&self) -> Result<Option<u64>> {
        let mut size: u64 = 0;

        let result = unsafe {
            ffi::rapidgzip_size(self.reader, &mut size)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(anyhow!("Failed to get size: {:?}", error));
        }

        Ok(if size > 0 { Some(size) } else { None })
    }
}

impl Read for ParallelGzipReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut bytes_read: usize = 0;

        let result = unsafe {
            ffi::rapidgzip_read(
                self.reader,
                buf.as_mut_ptr(),
                buf.len(),
                &mut bytes_read,
            )
        };

        let error = ffi::RapidGzipError::from_i32(result);

        match error {
            ffi::RapidGzipError::Ok => Ok(bytes_read),
            ffi::RapidGzipError::Eof => Ok(0),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Read failed: {:?}", error),
            )),
        }
    }
}

impl Seek for ParallelGzipReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let (offset, whence) = match pos {
            SeekFrom::Start(n) => (n as i64, ffi::RapidGzipSeekMode::Set),
            SeekFrom::Current(n) => (n, ffi::RapidGzipSeekMode::Cur),
            SeekFrom::End(n) => (n, ffi::RapidGzipSeekMode::End),
        };

        let mut position: u64 = 0;

        let result = unsafe {
            ffi::rapidgzip_seek(self.reader, offset, whence, &mut position)
        };

        let error = ffi::RapidGzipError::from_i32(result);
        if error != ffi::RapidGzipError::Ok {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Seek failed: {:?}", error),
            ));
        }

        Ok(position)
    }
}

impl Drop for ParallelGzipReader {
    fn drop(&mut self) {
        if !self.reader.is_null() {
            unsafe {
                ffi::rapidgzip_close(self.reader);
            }
        }
    }
}

// Safety: The C++ library handles thread safety internally
unsafe impl Send for ParallelGzipReader {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_reader() {
        // This test will fail until we have the actual rapidgzip library linked
        // For now, it's a placeholder to show the API structure
    }
}
