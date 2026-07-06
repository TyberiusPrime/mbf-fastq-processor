use std::fs::File;
use std::io::{Seek, Write};
use std::num::NonZero;
use std::os::fd::FromRawFd;

use fastqrab_io::io::input::DecompressionOptions;
use fastqrab_io::io::parsers::{Parser, PodFastqParser, ThreadCount};

#[cfg(feature = "afl-positive-control")]
const CANARY_MARKER: &[u8] = b"AFL_POSITIVE_CONTROL";

// target_reads_per_block is now only an API hint; a tiny buffer_size still
// forces many short reads → many decode-chunk / block boundaries, exercising
// the partial-read state machine on short inputs.
const TARGET_READS_PER_BLOCK: NonZero<usize> = NonZero::new(4).unwrap();
const BUFFER_SIZE: usize = 16;

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

// Mirrors `sniff_compression` in fastqrab-io: any input starting with these
// magics would be intercepted by open_decompressed_reader and decoded
// out-of-process, which is exactly what this parser-only fuzzer avoids. The
// decompression layer only recognizes gzip and zstd now (bzip2/xz were dropped
// when niffler was replaced by the out-of-process decompressor), so an input
// starting with any other bytes reaches the fastq parser as-is.
fn looks_compressed(data: &[u8]) -> bool {
    matches!(
        data,
        [0x1f, 0x8b, ..]               // gzip
        | [0x28, 0xb5, 0x2f, 0xfd, ..] // zstd
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

        let mut parser = match PodFastqParser::new(
            file,
            None,
            TARGET_READS_PER_BLOCK,
            BUFFER_SIZE,
            1,
            DecompressionOptions {
                thread_count: ThreadCount(NonZero::<usize>::MIN),
            },
        ) {
            Ok(p) => p,
            Err(_) => return,
        };

        loop {
            match parser.parse() {
                Ok(res) => {
                    #[cfg(feature = "afl-positive-control")]
                    {
                        let names = &res.output.names;
                        for i in 0..names.len() {
                            let name: &[u8] = names.get(i);
                            if name
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
