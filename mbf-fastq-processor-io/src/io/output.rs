use anyhow::{Context, Result};
use bstr::BString;
use noodles::sam::alignment::{
    RecordBuf,
    io::Write as SamAlignmentWrite,
    record::Flags as SamFlags,
    record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
};
use noodles::{bam, bgzf, sam};
use std::sync::Arc;

use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::io::reads::{WrappedFastQRead, WrappedFastQReadCommon};

pub mod compressed_output;

pub struct BamOutput<'a> {
    pub writer: bam::io::Writer<bgzf::io::Writer<HashedAndCompressedWriter<'a, ex::fs::File>>>,
    pub header: Arc<sam::Header>,
}

pub fn write_read_to_bam(
    bam_output: &mut BamOutput<'_>,
    read: &WrappedFastQRead<'_>,
    segment_index: usize,
    segment_count: usize,
) -> Result<()> {
    use noodles::sam::alignment::{
        record::data::field::Tag,
        record_buf::{Data, data::field::Value},
    };
    let mut flags = SamFlags::UNMAPPED;
    if segment_count > 1 {
        flags |= SamFlags::SEGMENTED;
        flags |= SamFlags::MATE_UNMAPPED;
        if segment_index == 0 {
            flags |= SamFlags::FIRST_SEGMENT;
        }
        if segment_index + 1 == segment_count {
            flags |= SamFlags::LAST_SEGMENT;
        }
    }

    // So we survive round tripping from fastq.
    let adjusted_quality_scores = read
        .qual()
        .iter()
        .map(|&q| q.saturating_sub(33))
        .collect::<Vec<u8>>();
    let (name, comment) = {
        // BAM may not have spaces in read names.
        // So we split on the first space, and put the rest in the comment field.
        // If there is no space, the comment field is None.
        if let Some(space_pos) = read.name().iter().position(|&c| c == b' ') {
            (
                &read.name()[..space_pos],
                Some(&read.name()[space_pos + 1..]),
            )
        } else {
            (read.name(), None)
        }
    };
    // Query or read names may contain any printable ASCII characters in the range [!-~] apart from ‘@’, so
    // that SAM alignment lines can be easily distinguished from header lines.
    let mut record = RecordBuf::builder()
        .set_name(name)
        //.set_name(BString::from("hello"))
        .set_flags(flags)
        .set_sequence(SamSequence::from(read.seq().to_vec()))
        .set_quality_scores(SamQualityScores::from(adjusted_quality_scores));
    if let Some(comment) = comment {
        let tag = Tag::from([b'C', b'O']);
        let data: Data = [(tag, Value::String(BString::from(comment)))]
            .into_iter()
            .collect();
        record = record.set_data(data);
    }
    let record = record.build();

    bam_output
        .writer
        .write_alignment_record(&bam_output.header, &record)
        .context("Failed to write BAM record")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::reads::{FastQBlock, FastQElement, FastQRead};
    use mbf_fastq_processor_config::CompressionFormat;
    use noodles::bam::record::Record as BamRecord;
    use std::str::FromStr;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn make_read_block() -> FastQBlock {
        let read = FastQRead::new(
            FastQElement::Owned(b"testread".to_vec()),
            FastQElement::Owned(b"ACGT".to_vec()),
            FastQElement::Owned(b"IIII".to_vec()), // PHRED+33 quality scores
        )
        .unwrap();
        FastQBlock {
            block: Vec::new(),
            entries: vec![read],
        }
    }

    fn write_and_read_flags(segment_index: usize, segment_count: usize) -> SamFlags {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        {
            let file = ex::fs::File::create(&path).unwrap();
            let hashed_writer = compressed_output::HashedAndCompressedWriter::new(
                file,
                CompressionFormat::Uncompressed,
                false,
                false,
                None,
                None,
                None,
            )
            .unwrap();
            let bgzf_writer = bgzf::io::Writer::new(hashed_writer);
            let mut writer = bam::io::Writer::from(bgzf_writer);
            let header = Arc::new(sam::Header::from_str("@HD\tVN:1.6\tSO:unsorted\n").unwrap());
            writer.write_header(&header).unwrap();
            let mut bam_output = BamOutput { writer, header };

            let block = make_read_block();
            write_read_to_bam(&mut bam_output, &block.get(0), segment_index, segment_count)
                .unwrap();
        } // drops bam_output, flushing and finalizing the file

        // Read back the flags of the first record
        let file = std::fs::File::open(&path).unwrap();
        let mut reader = bam::io::Reader::new(file);
        let _header = reader.read_header().unwrap();
        let mut record = BamRecord::default();
        let bytes_read = reader.read_record(&mut record).unwrap();
        assert_ne!(bytes_read, 0, "Expected a record in BAM output");
        record.flags()
    }

    #[test]
    fn test_bam_flags_single_read() {
        // segment_count=1: only UNMAPPED
        let flags = write_and_read_flags(0, 1);
        assert_eq!(flags, SamFlags::UNMAPPED);
    }

    #[test]
    fn test_bam_flags_first_of_two() {
        // segment_count=2, segment_index=0: SEGMENTED | MATE_UNMAPPED | UNMAPPED | FIRST_SEGMENT
        let flags = write_and_read_flags(0, 2);
        assert_eq!(
            flags,
            SamFlags::UNMAPPED
                | SamFlags::SEGMENTED
                | SamFlags::MATE_UNMAPPED
                | SamFlags::FIRST_SEGMENT
        );
    }

    #[test]
    fn test_bam_flags_last_of_two() {
        // segment_count=2, segment_index=1: SEGMENTED | MATE_UNMAPPED | UNMAPPED | LAST_SEGMENT
        let flags = write_and_read_flags(1, 2);
        assert_eq!(
            flags,
            SamFlags::UNMAPPED
                | SamFlags::SEGMENTED
                | SamFlags::MATE_UNMAPPED
                | SamFlags::LAST_SEGMENT
        );
    }

    #[test]
    fn test_bam_flags_first_of_four() {
        // segment_count=4, segment_index=0: FIRST_SEGMENT set
        let flags = write_and_read_flags(0, 4);
        assert_eq!(
            flags,
            SamFlags::UNMAPPED
                | SamFlags::SEGMENTED
                | SamFlags::MATE_UNMAPPED
                | SamFlags::FIRST_SEGMENT
        );
    }

    #[test]
    fn test_bam_flags_middle_of_four() {
        // segment_count=4, index 1 and 2: neither FIRST_SEGMENT nor LAST_SEGMENT
        for idx in [1_usize, 2_usize] {
            let flags = write_and_read_flags(idx, 4);
            assert_eq!(
                flags,
                SamFlags::UNMAPPED | SamFlags::SEGMENTED | SamFlags::MATE_UNMAPPED,
                "Failed for segment_index={idx}"
            );
        }
    }

    #[test]
    fn test_bam_flags_last_of_four() {
        // segment_count=4, segment_index=3: LAST_SEGMENT set
        let flags = write_and_read_flags(3, 4);
        assert_eq!(
            flags,
            SamFlags::UNMAPPED
                | SamFlags::SEGMENTED
                | SamFlags::MATE_UNMAPPED
                | SamFlags::LAST_SEGMENT
        );
    }
}
