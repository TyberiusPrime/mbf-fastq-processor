use anyhow::{Context, Result, bail};
use bstr::{BStr, BString};
use noodles::sam::alignment::{
    RecordBuf,
    io::Write as SamAlignmentWrite,
    record::Flags as SamFlags,
    record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
};
use noodles::{bam, bgzf, sam};
use std::sync::Arc;

use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::io::reads::{Tags, WrappedFastQRead, WrappedFastQReadCommon};
use fastqrab_dna::dna::TagValue;

pub mod compressed_output;

pub struct BamOutput {
    pub writer: bam::io::Writer<
        bgzf::io::multithreaded_writer::MultithreadedWriter<
            HashedAndCompressedWriter<ex::fs::File>,
        >,
    >,
    pub header: Arc<sam::Header>,
}

/// Write a single read to a BAM file.
///
/// # Parameters
/// - `bam_output`: the open BAM writer with its header
/// - `read`: the read to write
/// - `read_index`: the position of this read in the current block (used for tag lookup)
/// - `segment_index`: 0-based index among segments in a paired/multi-segment set
/// - `segment_count`: total number of segments (1 for single-end)
/// - `comment_separation_char`: character used to split the read name into name+CO tag
/// - `tags`: per-read tags for this block (may be empty)
/// - `bam_tag_mappings`: list of `(bam_tag_bytes, fastqrab_tag_name)` pairs to export (Feature A)
/// - `reference_tag`: if `Some(tag_name)`, look up that tag's value in the BAM reference
///   sequences and set the read's RNAME / alignment start accordingly (Feature B)
#[allow(clippy::too_many_arguments)]
pub fn write_read_to_bam(
    bam_output: &mut BamOutput,
    read: &WrappedFastQRead<'_>,
    read_index: usize,
    segment_index: usize,
    segment_count: usize,
    comment_separation_char: u8,
    tags: &Tags,
    bam_tag_mappings: &[([u8; 2], &str)],
    reference_tag: Option<&str>,
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
        .map(|&q| q.saturating_sub(33)) //todo: evaluate if this shouldn't fail instead.
        .collect::<Vec<u8>>();
    let (name, comment) = {
        // BAM may not have spaces in read names.
        // So we split on the first space, and put the rest in the comment field.
        // If there is no space, the comment field is None.
        if let Some(space_pos) = read
            .name()
            .iter()
            .position(|&c| c == comment_separation_char)
        {
            (
                &read.name()[..space_pos],
                Some(&read.name()[space_pos + 1..]),
            )
        } else {
            (read.name(), None)
        }
    };

    // --- build auxiliary tag data --------------------------------
    let mut data_fields: Vec<(Tag, Value)> = Vec::new();

    if let Some(comment) = comment {
        let tag = Tag::from([b'C', b'O']);
        data_fields.push((tag, Value::String(BString::from(comment))));
    }

    for (bam_tag_bytes, fastqrab_tag_name) in bam_tag_mappings {
        if let Some(tag_values) = tags.get(*fastqrab_tag_name) {
            if let Some(tag_value) = tag_values.get(read_index) {
                let value_opt: Option<Value> = match tag_value {
                    TagValue::String(s) => Some(Value::String(s.clone())),
                    TagValue::Location(hits) => {
                        // Join all hit sequences with commas
                        if hits.0.is_empty() {
                            None
                        } else {
                            let joined = hits
                                .0
                                .iter()
                                .map(|h| h.sequence.as_ref())
                                .collect::<Vec<_>>()
                                .join(b",".as_ref());
                            Some(Value::String(BString::from(joined)))
                        }
                    }
                    TagValue::Numeric(n) => Some(Value::Float(*n as f32)),
                    TagValue::Bool(b) => Some(Value::UInt8(u8::from(*b))),
                    TagValue::Missing => None,
                };
                if let Some(value) = value_opt {
                    data_fields.push((Tag::from(*bam_tag_bytes), value));
                }
            }
        }
    }

    let data: Data = data_fields.into_iter().collect();

    // --- assign reference ----------------------------------------
    let mut reference_sequence_id: Option<usize> = None;
    if let Some(ref_tag_name) = reference_tag {
        if let Some(tag_values) = tags.get(ref_tag_name) {
            //missing > not 'aligned'
            if let Some(tag_value) = tag_values.get(read_index) {
                let ref_name = tag_value.to_bstr();
                // Look up the reference name in the BAM header
                let key: &[u8] = &ref_name;
                if !key.is_empty() {
                    if let Some(idx) = bam_output.header.reference_sequences().get_index_of(key) {
                        reference_sequence_id = Some(idx);
                        flags.remove(SamFlags::UNMAPPED);
                    } else {
                        bail!(
                            "Error in Bam tag-to-reference output: \n\
                            the value '{ref_name}' for tag '{ref_tag_name}' was not a valid reference sequence.\n\
                           Check that your output.bam.tag_to_reference.from_bam|from_barcodes derived values match\n\
                           with the actual tag values. Read name involved: '{}'",
                            BStr::new(read.name()),
                        );
                    }
                } // else stay at 'not aligned'
            }
        }
    }

    // Query or read names may contain any printable ASCII characters in the range [!-~] apart from '@', so
    // that SAM alignment lines can be easily distinguished from header lines.
    let mut record_builder = RecordBuf::builder()
        .set_name(name)
        .set_flags(flags)
        .set_sequence(SamSequence::from(read.seq().to_vec()))
        .set_quality_scores(SamQualityScores::from(adjusted_quality_scores))
        .set_data(data);

    if let Some(ref_id) = reference_sequence_id {
        record_builder = record_builder
            .set_reference_sequence_id(ref_id)
            .set_alignment_start(noodles_core::Position::MIN);
    }

    let record = record_builder.build();

    if let Err(e) = bam_output
        .writer
        .write_alignment_record(&bam_output.header, &record)
    {
        let mut res = Err(e).context("Failed to write BAM record");
        let name: BString = name.into();
        if name.len() > 254 {
            res = res.context(format!(
                "The read name exceeded the 254 byte limited of the SAM/BAM spec.\n\
                    Shorten your read name, or set output.bam.comment_separation_char\n\
                    to split your read name into a name and a 'CO' tag (which may exceed 254 bytes).\n\
                    Read name (length: {len}): '{name}'",
                len = name.len()
            ));
        } else if name.is_empty() {
            res = res.context("Empty read name not supported by BAM. Check you Rename steps?");
        }
        //bam only allows printable characters. [!-?A-~]
        if name.iter().any(|&c| c < 33 || c > 126 || c == b'@') {
            res = res.context(format!(
                "The read name contains characters that are not allowed in the SAM/BAM spec.\n\
                    Remove or replace these characters, or set output.bam.comment_separation_char\n\
                    to split your read name into a name and a 'CO' tag (which may contain these characters).\n\
                    Read name: '{name}'"
            ));
        }
        return res;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::reads::{FastQBlock, FastQElement, FastQRead};
    use fastqrab_config::CompressionFormat;
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
            first_read_sequential_number: 0,
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
            let bgzf_writer = bgzf::io::multithreaded_writer::MultithreadedWriter::new(hashed_writer);
            let mut writer = bam::io::Writer::from(bgzf_writer);
            let header = Arc::new(sam::Header::from_str("@HD\tVN:1.6\tSO:unsorted\n").unwrap());
            writer.write_header(&header).unwrap();
            let mut bam_output = BamOutput { writer, header };

            let block = make_read_block();
            write_read_to_bam(
                &mut bam_output,
                &block.get(0),
                0,
                segment_index,
                segment_count,
                b' ',
                &Default::default(),
                &[],
                None,
            )
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
