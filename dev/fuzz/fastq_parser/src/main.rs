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
    let name = c"afl_fastq_input";
    // SAFETY: memfd_create syscall with a valid NUL-terminated name.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), 0) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: memfd_create returned a valid owned fd.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn main() {
    afl::fuzz!(|data: &[u8]| {
        let Ok(mut file) = memfd() else { return };
        if file.write_all(data).is_err() {
            return;
        }
        if file.rewind().is_err() {
            return;
        }

        // Small target_reads_per_block and buf_size force the parser through
        // many block boundaries so the partial-read state machine gets heavy
        // coverage on short inputs.
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
