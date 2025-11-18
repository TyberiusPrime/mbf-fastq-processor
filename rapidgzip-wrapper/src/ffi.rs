use std::os::raw::{c_char, c_int};

// Opaque handle type for the ParallelGzipReader
#[repr(C)]
pub struct RapidGzipReader {
    _private: [u8; 0],
}

// Error codes
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RapidGzipError {
    Ok = 0,
    InvalidHandle = -1,
    OpenFailed = -2,
    ReadFailed = -3,
    SeekFailed = -4,
    Eof = -5,
    Unknown = -99,
}

impl RapidGzipError {
    pub fn from_i32(code: i32) -> Self {
        match code {
            0 => RapidGzipError::Ok,
            -1 => RapidGzipError::InvalidHandle,
            -2 => RapidGzipError::OpenFailed,
            -3 => RapidGzipError::ReadFailed,
            -4 => RapidGzipError::SeekFailed,
            -5 => RapidGzipError::Eof,
            _ => RapidGzipError::Unknown,
        }
    }
}

// Seek modes
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RapidGzipSeekMode {
    Set = 0,
    Cur = 1,
    End = 2,
}

unsafe extern "C" {
    pub fn rapidgzip_open(
        filepath: *const c_char,
        num_threads: usize,
        out_reader: *mut *mut RapidGzipReader,
    ) -> c_int;

    pub fn rapidgzip_open_fd(
        fd: c_int,
        num_threads: usize,
        out_reader: *mut *mut RapidGzipReader,
    ) -> c_int;

    pub fn rapidgzip_read(
        reader: *mut RapidGzipReader,
        buffer: *mut u8,
        size: usize,
        out_bytes_read: *mut usize,
    ) -> c_int;

    pub fn rapidgzip_seek(
        reader: *mut RapidGzipReader,
        offset: i64,
        whence: RapidGzipSeekMode,
        out_position: *mut u64,
    ) -> c_int;

    pub fn rapidgzip_tell(
        reader: *mut RapidGzipReader,
        out_position: *mut u64,
    ) -> c_int;

    pub fn rapidgzip_eof(
        reader: *mut RapidGzipReader,
        out_eof: *mut c_int,
    ) -> c_int;

    pub fn rapidgzip_set_crc32_enabled(
        reader: *mut RapidGzipReader,
        enabled: c_int,
    ) -> c_int;

    pub fn rapidgzip_size(
        reader: *mut RapidGzipReader,
        out_size: *mut u64,
    ) -> c_int;

    pub fn rapidgzip_close(reader: *mut RapidGzipReader);
}
