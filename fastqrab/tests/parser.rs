#![expect(clippy::unwrap_used, reason = "it's tests")]
use bstr::ByteSlice;
use std::num::NonZero;
use std::path::{Path, PathBuf};

use fastqrab_io::io::parsers::{ParserThreadCounts, ThreadCount};

#[path = "common/mod.rs"]
mod common;

/// One read's owned `(name, seq, qual)`, for buffer-size-invariance comparison.
type OwnedRead = (Vec<u8>, Vec<u8>, Vec<u8>);

#[test]
fn test_fastq_bufsize_variations_windows_file() {
    // The input is zstd-compressed, which now always decodes out-of-process; point
    // `find_decompressor` at the freshly built binary so the pipe path can spawn it.
    // SAFETY: set once at the start of this single-threaded test, before any spawn.
    unsafe {
        std::env::set_var("FASTQRAB_DECOMPRESSOR", common::decompressor());
    }

    let filename = "../test_cases/sample_data/zstd/input_read1.fq.zst";
    //verify we have \r\n in that
    let contents: Vec<u8> =
        fastqrab::decompress_file(Path::new(filename)).expect("failed to read test file");
    let query = b"\r\n";
    assert!(contents.contains_str(query));

    let mut bufsizes = vec![4, 16, 64, 256, 1024, 65365];
    bufsizes.extend(950..1001);
    test_bufsize_variations(filename, &bufsizes);
}

fn test_bufsize_variations(input_fastq_filename: &str, bufsize_range: &[usize]) {
    let filename = input_fastq_filename;

    let mut last: Option<Vec<OwnedRead>> = None;

    for bufsize in bufsize_range {
        let file = ex::fs::File::open(filename).unwrap();

        let input_file =
            fastqrab_io::io::input::InputFile::Fastq(file, Some(PathBuf::from(filename)));
        let mut p = input_file
            .get_parser(
                NonZero::new(10_000).expect("can't happen"),
                *bufsize,
                ParserThreadCounts {
                    decompression: ThreadCount(std::num::NonZero::<usize>::MIN),
                    pod_demux: ThreadCount(std::num::NonZero::<usize>::MIN),
                },
                &fastqrab_io::io::input::InputOptions {
                    bam_include_mapped: None,
                    bam_include_unmapped: None,
                    fasta_fake_quality: None,
                    read_comment_character: b' ',
                    use_rapidgzip: false,
                    threads_per_segment: Some(1),
                },
            )
            .unwrap();
        let mut here: Vec<OwnedRead> = Vec::new();
        loop {
            let pr = p.parse().unwrap();
            // get_parser yields either the legacy row block or columnar chunks;
            // flatten both into owned (name, seq, qual) reads for comparison.
            let c = pr.output;
            for i in 0..c.len() {
                let (seq, qual) = c.seq_quals.pair(i);
                here.push((
                    c.names.get(i).as_bytes().to_vec(),
                    seq.as_bytes().to_vec(),
                    qual.as_bytes().to_vec(),
                ));
            }
            if pr.was_final {
                break;
            }
        }

        if let Some(last) = last {
            assert_eq!(last, here, "read stream differs at bufsize {bufsize}");
        }

        last = Some(here);
    }
}
