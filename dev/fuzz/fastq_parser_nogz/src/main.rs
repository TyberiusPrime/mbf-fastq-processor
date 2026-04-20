use std::fs::File;
use std::io::{Seek, Write};
use std::os::fd::FromRawFd;

use fastqrab_io::io::input::DecompressionOptions;
use fastqrab_io::io::parsers::{FastqParser, Parser};
#[cfg(feature = "afl-positive-control")]
use fastqrab_io::io::reads::WrappedFastQReadCommon;

#[cfg(feature = "afl-positive-control")]
const CANARY_MARKER: &[u8] = b"AFL_POSITIVE_CONTROL";

// memfd_create makes an anonymous, memory-backed file that never touches any
// filesystem — critical for parallel fuzzing throughput, where per-exec
// tempfile::tempfile() creates/unlinks on /tmp and hits vfs lock contention
// at ~1M ops/sec across 32 instances.
fn memfd() -> std::io::Result<File> {
    let name = c"afl_fastq_input_nogz";
    // SAFETY: memfd_create syscall with a valid NUL-terminated name.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: memfd_create returned a valid owned fd.
    Ok(unsafe { File::from_raw_fd(fd) })
}

// Mirrors `niffler::send::compression::bytes2type`: any input starting with
// these magics would be intercepted by FastqParser::new and decompressed,
// which is exactly what this fuzzer wants to avoid.
fn looks_compressed(data: &[u8]) -> bool {
    matches!(
        data,
        [0x1f, 0x8b, ..]                       // gzip
        | [0x42, 0x5a, ..]                     // bzip2
        | [0xfd, 0x37, 0x7a, 0x58, 0x5a, ..]   // xz / lzma
        | [0x28, 0xb5, 0x2f, 0xfd, ..]         // zstd
    )
}

fn main() {
    afl::fuzz!(|data: &[u8]| {
        // Bail before any I/O: AFL won't credit this path with coverage, so
        // it learns to steer mutations away from compression magics.
        if looks_compressed(data) {
            return;
        }

        let Ok(mut file) = memfd() else { return };
        if file.write_all(data).is_err() {
            return;
        }
        if file.rewind().is_err() {
            return;
        }

        // Same tight buf parameters as the gzip-aware fuzzer: forces many
        // block boundaries so the partial-read state machine gets exercised
        // on short inputs.
        let mut parser = match FastqParser::new(
            file,
            None,
            4,
            16,
            DecompressionOptions::Default,
        ) {
            Ok(p) => p,
            Err(_) => return,
        };

        loop {
            match parser.parse() {
                Ok(res) => {
                    #[cfg(feature = "afl-positive-control")]
                    {
                        for i in 0..res.fastq_block.entries.len() {
                            let read = res.fastq_block.get(i);
                            if read
                                .name()
                                .windows(CANARY_MARKER.len())
                                .any(|w| w == CANARY_MARKER)
                            {
                                panic!("afl-positive-control canary tripped");
                            }
                        }
                    }
                    if res.was_final {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}
