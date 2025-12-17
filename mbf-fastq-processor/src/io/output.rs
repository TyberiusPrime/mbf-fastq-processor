use anyhow::{Context, Result};
use std::sync::Arc;

use super::reads::WrappedFastQRead;
use crate::io::output::compressed_output::HashedAndCompressedWriter;
use bstr::BString;
use noodles::sam::alignment::{
    RecordBuf,
    io::Write as SamAlignmentWrite,
    record::Flags as SamFlags,
    record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
};
use noodles::{bam, bgzf, sam};

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
        flags |= SamFlags::SEGMENTED | SamFlags::MATE_UNMAPPED;
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
