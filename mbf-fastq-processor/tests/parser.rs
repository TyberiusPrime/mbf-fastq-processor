use std::path::PathBuf;

use mbf_fastq_processor::io::{FastQBlock, parsers::ThreadCount};

#[test]
fn test_fastq_bufsize_variations_windows_file() {
    let filename = "../test_cases/sample_data/zstd/input_read1.fq.zst";
    let mut bufsizes = vec![4, 16, 64, 256, 1024, 65365];
    bufsizes.extend(950..1001);
    test_bufsize_variations(filename, &bufsizes);
}

fn test_bufsize_variations(input_fastq_filename: &str, bufsize_range: &[usize]) {
    let filename = input_fastq_filename;

    let mut last: Option<Vec<FastQBlock>> = None;

    for bufsize in bufsize_range.iter() {
        dbg!(bufsize);
        let file = ex::fs::File::open(filename).unwrap();

        let input_file =
            mbf_fastq_processor::io::input::InputFile::Fastq(file, Some(PathBuf::from(filename)));
        let mut p = input_file
            .get_parser(
                10000,
                *bufsize,
                ThreadCount(1),
                &mbf_fastq_processor::config::InputOptions {
                    bam_include_mapped: None,
                    bam_include_unmapped: None,
                    fasta_fake_quality: None,
                    read_comment_character: b' ',
                    build_rapidgzip_index: Some(false),
                    use_rapidgzip: Some(false),
                    threads_per_segment: Some(1),
                },
            )
            .unwrap();
        let mut here = Vec::new();
        loop {
            let pr = p.parse().unwrap();
            here.push(pr.fastq_block);
            if pr.was_final {
                break;
            }
        }

        if let Some(last) = last {
            assert_eq!(last.len(), here.len());
            for (b1, b2) in last.iter().zip(here.iter()) {
                assert_eq!(b1.len(), b2.len());
                for (r1, r2) in b1.entries.iter().zip(b2.entries.iter()) {
                    assert_eq!(r1.name.get(&b1.block), r2.name.get(&b2.block));
                    assert_eq!(r1.seq.get(&b1.block), r2.seq.get(&b2.block));
                    assert_eq!(r1.qual.get(&b1.block), r2.qual.get(&b2.block));
                }
            }
        }

        last = Some(here);
    }
}
