use anyhow::{Result, bail};
use bstr::ByteSlice;
use ex::fs::File;
use noodles::bam::bai;
use noodles::bam::{self, record::Record};
use noodles::bgzf;
use noodles::csi::binning_index::{BinningIndex, ReferenceSequence};
use std::num::NonZero;
use std::path::{Path, PathBuf};

use crate::blocks::FastQChunk;
use crate::io::parsers::{ParseResult, Parser, ParserOutput};
use stringpod::{DualStringPodBuilder, StringPod, StringPodBuilder};

type BamReader = bam::io::Reader<bgzf::io::MultithreadedReader<File>>;

pub struct BamParser {
    reader: BamReader,
    target_reads_per_block: NonZero<usize>,
    include_mapped: bool,
    include_unmapped: bool,
    record: Record,
    /// Only used for error messages. `None` when the BAM is read from a
    /// pre-opened handle (e.g. a step's declared input file) that has no path.
    filename: Option<PathBuf>,
    any_seen: bool,
}

/// # Panics
/// when read count > usize limit
pub fn bam_read_count_from_index(
    filename: impl AsRef<Path>,
    include_mapped: bool,
    include_unmapped: bool,
) -> Option<usize> {
    let path = filename.as_ref();
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bam"))
    {
        let candidates = [
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bam.bai");
                idx
            },
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bai");
                idx
            },
        ];

        for index_path in candidates {
            if !index_path.exists() {
                continue;
            }

            match bai::fs::read(&index_path) {
                Ok(index) => {
                    let total_reads: u128 = index
                        .reference_sequences()
                        .iter()
                        .filter_map(|reference| reference.metadata())
                        .map(|metadata| {
                            let mut total = 0u128;
                            if include_mapped {
                                total += u128::from(metadata.mapped_record_count());
                            }
                            if include_unmapped {
                                // that's 'it's on this reference, but we don't know where,
                                // I suppose.
                                total += u128::from(metadata.unmapped_record_count());
                            }
                            total
                        })
                        .sum::<u128>()
                        + (if include_unmapped {
                            u128::from(index.unplaced_unmapped_record_count().unwrap_or(0))
                        } else {
                            0
                        });

                    return Some(
                        total_reads.try_into().expect(
                            "Read count exceeded usize. Your BAM file must be astronomical.",
                        ),
                    );
                }
                Err(error) => {
                    //treat it as a soft error
                    eprintln!(
                        "Warning: Failed to read BAM index {} for {}: {error} - returning an expected read count of zero",
                        index_path.display(),
                        path.display()
                    );
                }
            }
        }
    }
    None
}

impl BamParser {
    pub fn new(
        file: File,
        filename: Option<PathBuf>,
        target_reads_per_block: NonZero<usize>,
        include_mapped: bool,
        include_unmapped: bool,
        cores: std::num::NonZero<usize>,
    ) -> Result<BamParser> {
        let worker_count: std::num::NonZero<_> = cores;
        let bgzf_reader = bgzf::io::MultithreadedReader::with_worker_count(worker_count, file);
        let mut reader = bam::io::Reader::from(bgzf_reader);
        reader.read_header()?;

        Ok(BamParser {
            reader,
            target_reads_per_block,
            include_mapped,
            include_unmapped,
            record: Record::default(),
            filename,
            any_seen: false,
        })
    }

    fn should_yield_record(&self, record: &Record) -> bool {
        let is_mapped = record.reference_sequence_id().is_some();
        (is_mapped && self.include_mapped) || (!is_mapped && self.include_unmapped)
    }
}

impl Parser for BamParser {
    #[mutants::skip] // only used to estimate read count for duplicate filters
    // cov:excl-start
    fn bytes_per_base(&self) -> f64 {
        1.0 // about right
    }
    // cov:excl-stop

    fn parse(&mut self) -> Result<ParseResult> {
        let target: usize = self.target_reads_per_block.into();
        let mut names = StringPodBuilder::with_capacity(0, target);
        let mut seq_quals = DualStringPodBuilder::with_capacity(0, target);
        let mut count = 0usize;
        let mut was_final = false;
        // Reused per record so seq/qual can be materialized into contiguous,
        // equal-length slices for the DualStringPod.
        let mut seq_buf: Vec<u8> = Vec::new();
        let mut qual_buf: Vec<u8> = Vec::new();

        while count < target {
            self.record = Record::default();
            if self.reader.read_record(&mut self.record)? == 0 {
                //nothing read.
                if count == 0 && !self.any_seen {
                    match &self.filename {
                        Some(filename) => bail!(
                            "An input file ({}) provided no reads. Please check your inputs.",
                            filename.display()
                        ),
                        None => bail!("An input file provided no reads. Please check your inputs."),
                    }
                }
                was_final = true;
                break;
            }

            if !self.should_yield_record(&self.record) {
                continue;
            }

            names.push(self.record.name().map(|n| n.as_bytes()).unwrap_or_default());

            let seq = self.record.sequence();
            seq_buf.clear();
            seq_buf.extend(seq.iter());
            let qual = self.record.quality_scores();
            qual_buf.clear();
            if qual.is_empty() {
                // BAM records may omit quality ('*'); synthesize phred-0 so the
                // seq/qual columns stay the same length.
                qual_buf.resize(seq_buf.len(), b'!');
            } else {
                qual_buf.extend(qual.iter().map(|q| q + 33));
            }
            seq_quals.push(seq_buf.as_slice(), qual_buf.as_slice());
            count += 1;
        }

        if count >= target {
            self.any_seen = true;
        }

        Ok(ParseResult {
            output: ParserOutput::Chunk(FastQChunk {
                names: names.finish(),
                seq_quals: seq_quals.finish(),
                pluses: StringPod::new_all_empty(
                    u32::try_from(count).expect("too many reads in a block for u32"),
                ),
            }),
            was_final,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use noodles::bam;
    use noodles::sam::alignment::io::Write;
    use noodles::sam::{
        self,
        alignment::record::Flags as SamFlags,
        alignment::record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
        header::record::value::{Map, map::ReferenceSequence},
    };
    use std::num::NonZeroUsize;
    use tempfile::NamedTempFile;

    fn write_test_bam(path: &std::path::Path) -> Result<()> {
        let reference_length = NonZeroUsize::new(100).expect("100 is non-zero");
        let header = sam::Header::builder()
            .add_reference_sequence("chr1", Map::<ReferenceSequence>::new(reference_length))
            .build();

        let file = std::fs::File::create(path)?;
        let mut writer = bam::io::Writer::new(file);
        writer.write_header(&header)?;

        let mut mapped = sam::alignment::RecordBuf::default();
        *mapped.name_mut() = Some("mapped".into());
        *mapped.flags_mut() = SamFlags::empty();
        *mapped.reference_sequence_id_mut() = Some(0);
        *mapped.sequence_mut() = SamSequence::from(b"ACGT".to_vec());
        *mapped.quality_scores_mut() = SamQualityScores::from(vec![30, 30, 30, 30]);
        writer.write_alignment_record(&header, &mapped)?;

        let mut unmapped = sam::alignment::RecordBuf::default();
        *unmapped.name_mut() = Some("unmapped".into());
        *unmapped.flags_mut() = SamFlags::UNMAPPED;
        *unmapped.sequence_mut() = SamSequence::from(b"TGCA".to_vec());
        *unmapped.quality_scores_mut() = SamQualityScores::from(vec![25, 25, 25, 25]);
        writer.write_alignment_record(&header, &unmapped)?;

        writer.try_finish()?;
        Ok(())
    }

    #[test]
    fn respects_mapped_and_unmapped_filters() -> Result<()> {
        use bstr::ByteSlice;
        let temp = NamedTempFile::new()?;
        write_test_bam(temp.path())?;

        let open = |path: &std::path::Path| -> Result<File> { Ok(File::open(path)?) };

        let file = open(temp.path())?;
        let mut parser = BamParser::new(
            file,
            Some(temp.path().to_owned()),
            NonZero::new(10).unwrap(),
            true,
            false,
            std::num::NonZero::new(1usize).expect("1 is not zero"),
        )?; // cov:excl-line
        let ParseResult {
            output,
            was_final: finished,
        } = parser.parse()?;
        let chunk = output.into_chunk();
        assert!(finished);
        assert_eq!(chunk.len(), 1);
        assert_eq!(chunk.names.get(0).as_bytes(), b"mapped");

        let file = open(temp.path())?;
        let mut parser = BamParser::new(
            file,
            Some(temp.path().to_owned()),
            NonZero::new(10).unwrap(),
            false,
            true,
            std::num::NonZero::new(1usize).expect("1 is not zero"),
        )?; // cov:excl-line
        let ParseResult {
            output,
            was_final: finished,
        } = parser.parse()?;
        let chunk = output.into_chunk();
        assert!(finished);
        assert_eq!(chunk.len(), 1);
        assert_eq!(chunk.names.get(0).as_bytes(), b"unmapped");

        let file = open(temp.path())?;
        let mut parser = BamParser::new(
            file,
            Some(temp.path().to_owned()),
            NonZero::new(10).unwrap(),
            true,
            true,
            std::num::NonZero::new(1usize).expect("1 is not zero"),
        )?; // cov:excl-line
        let ParseResult {
            output,
            was_final: finished,
        } = parser.parse()?;
        let chunk = output.into_chunk();
        assert!(finished);
        assert_eq!(chunk.len(), 2);

        Ok(())
    }

    #[test]
    fn test_bam_read_count_from_index() {
        let path = PathBuf::from("../test_cases/sample_data/bam/input_read1.bam")
            .canonicalize()
            .unwrap();
        // cov:excl-start
        assert!(
            std::fs::metadata(&path).is_ok(),
            "Test BAM file not found at {:?}",
            &path
        );
        // cov:excl-stop
        assert_eq!(bam_read_count_from_index(&path, true, false), Some(0));
        assert_eq!(bam_read_count_from_index(&path, false, false), Some(0));
        assert_eq!(bam_read_count_from_index(&path, false, true), Some(2));
        assert_eq!(bam_read_count_from_index(&path, true, true), Some(2));

        let path =
            "../test_cases/sample_data/bam/input_ERR12828869_10k_1.head_500.all_unaligned.bam";
        assert_eq!(bam_read_count_from_index(path, true, false), Some(0));
        assert_eq!(bam_read_count_from_index(path, false, true), Some(533));
        assert_eq!(bam_read_count_from_index(path, true, true), Some(533));
        let path = "../test_cases/sample_data/bam//input_ERR12828869_10k_1.head_500.bam";
        assert_eq!(bam_read_count_from_index(path, false, true), Some(0));
        assert_eq!(bam_read_count_from_index(path, true, true), Some(533));
    }
}
