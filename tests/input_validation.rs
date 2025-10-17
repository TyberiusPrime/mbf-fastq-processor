use std::io::Write;
use std::num::NonZeroUsize;

use mbf_fastq_processor::config::Config;
use noodles::bam;
use noodles::sam::alignment::io::Write as SamAlignmentWrite;
use noodles::sam::{
    self,
    alignment::record::Flags as SamFlags,
    alignment::record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
    header::record::value::{Map, map::ReferenceSequence},
};
use tempfile::NamedTempFile;

fn write_test_bam(path: &std::path::Path) -> anyhow::Result<()> {
    let reference_length = NonZeroUsize::new(100).unwrap();
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
    *mapped.sequence_mut() = SamSequence::from(b"AC".to_vec());
    *mapped.quality_scores_mut() = SamQualityScores::from(vec![30, 30]);
    writer.write_alignment_record(&header, &mapped)?;

    writer.try_finish()?;
    Ok(())
}

#[test]
fn fails_when_fasta_fake_quality_missing() -> anyhow::Result<()> {
    let mut fasta = NamedTempFile::new()?;
    write!(fasta, ">read1\nACGT\n")?;
    fasta.flush()?;

    let config_toml = format!(
        r#"
        [input]
        read1 = ["{path}"]
        "#,
        path = fasta.path().display()
    );

    let mut config: Config = toml::from_str(&config_toml)?;
    let err = config.check().unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("fasta_fake_quality"));
    Ok(())
}

#[test]
fn fails_when_bam_include_flags_missing() -> anyhow::Result<()> {
    let bam_file = NamedTempFile::new()?;
    write_test_bam(bam_file.path())?;

    let config_toml = format!(
        r#"
        [input]
        read1 = ["{path}"]
        "#,
        path = bam_file.path().display()
    );

    let mut config: Config = toml::from_str(&config_toml)?;
    let err = config.check().unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("bam_include_mapped") || msg.contains("bam_include_unmapped"));
    Ok(())
}

#[test]
fn fails_when_bam_filters_disable_all_reads() -> anyhow::Result<()> {
    let bam_file = NamedTempFile::new()?;
    write_test_bam(bam_file.path())?;

    let config_toml = format!(
        r#"
        [input]
        read1 = ["{path}"]

        [input.options]
        bam_include_mapped = false
        bam_include_unmapped = false
        "#,
        path = bam_file.path().display()
    );

    let mut config: Config = toml::from_str(&config_toml)?;
    let err = config.check().unwrap_err();
    assert!(format!("{err:?}").contains("At least one"));
    Ok(())
}

#[test]
fn fails_when_segment_mixes_input_formats() -> anyhow::Result<()> {
    let mut fasta = NamedTempFile::new()?;
    write!(fasta, ">read1\nACGT\n")?;
    fasta.flush()?;

    let mut fastq = NamedTempFile::new()?;
    write!(fastq, "@read2\nACGT\n+\n!!!!\n")?;
    fastq.flush()?;

    let config_toml = format!(
        r#"
        [input]
        read1 = ["{fasta}", "{fastq}"]

        [input.options]
        fasta_fake_quality = 30
        "#,
        fasta = fasta.path().display(),
        fastq = fastq.path().display()
    );

    let mut config: Config = toml::from_str(&config_toml)?;
    let err = config.check().unwrap_err();
    assert!(format!("{err:?}").contains("mixes input formats"));
    Ok(())
}
