use anyhow::{Context, Result};
use noodles::bam::bai;
use noodles::{bam, bgzf};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::demultiplex::{OptDemultiplex, Tag};

/// Per-demultiplex-step info needed to compute merge groups at post-processing time.
#[derive(Clone, Debug)]
pub struct DemultiplexStepInfo {
    /// The `in_label` of the Demultiplex step (as a plain string for easy comparison).
    pub in_label: String,
    /// Maps this step's local bit-pattern → barcode name.
    pub local_tag_to_name: BTreeMap<Tag, String>,
    /// OR of all values in local_tag_to_name — the bit-mask for this step's contribution.
    pub merge_mask: Tag,
}

/// Merge the demultiplexed BAM files produced for `merge_label` into combined output files.
///
/// * One merged file per unique combination of the **other** demultiplex levels.
/// * Intermediary files are removed after merging.
/// * If `index` is true a `.bai` index is written beside each merged file.
#[allow(clippy::too_many_arguments)]
pub fn merge_demultiplexed_bam(
    output_directory: &Path,
    prefix: &str,
    suffix: &str,
    sep: &str,
    demultiplex_infos: &[(usize, OptDemultiplex)],
    demultiplex_step_infos: &[DemultiplexStepInfo],
    merge_label: &str,
    index: bool,
) -> Result<()> {
    // ── find the merge step ──────────────────────────────────────────────────
    let (merge_step_idx, merge_step_info) = demultiplex_step_infos
        .iter()
        .enumerate()
        .find(|(_, s)| s.in_label == merge_label)
        .with_context(|| {
            format!("No demultiplex step found with in_label='{merge_label}' (this is a bug)")
        })?;

    let merge_mask = merge_step_info.merge_mask;

    // ── final tag→name mapping ───────────────────────────────────────────────
    let final_tag_to_name = match demultiplex_infos.last() {
        Some((_, OptDemultiplex::Yes(info))) => &info.tag_to_name,
        _ => {
            // No demultiplexing active — nothing to merge.
            return Ok(());
        }
    };

    // ── build merge groups: other_tag → (other_name, [(combined_tag, combined_name)]) ──
    let mut groups: BTreeMap<Tag, (String, Vec<(Tag, String)>)> = BTreeMap::new();

    for (combined_tag, combined_name_opt) in final_tag_to_name {
        let Some(combined_name) = combined_name_opt else {
            continue; // no output file for this tag
        };
        let other_tag = combined_tag & !merge_mask;

        // Derive the "other" name by stripping the merge-step's name from the combined name.
        let merge_part_tag = combined_tag & merge_mask;
        let merge_part_name = if merge_part_tag == 0 {
            // "no-barcode" for the merge step — its name in the combined string is "no-barcode"
            "no-barcode".to_string()
        } else {
            merge_step_info
                .local_tag_to_name
                .get(&merge_part_tag)
                .cloned()
                .unwrap_or_default()
        };

        let other_name = strip_step_name(combined_name, &merge_part_name, sep);

        let entry = groups
            .entry(other_tag)
            .or_insert_with(|| (other_name, Vec::new()));
        entry.1.push((*combined_tag, combined_name.clone()));
    }

    // Sort source files within each group deterministically (by combined_name).
    for (_, (_, sources)) in &mut groups {
        sources.sort_by(|a, b| a.1.cmp(&b.1));
    }

    // ── merge each group ─────────────────────────────────────────────────────
    let _ = merge_step_idx; // suppress warning — used for context only
    let dot_suffix = format!(".{suffix}");

    for (_, (other_name, sources)) in groups {
        // Discover segment tails by listing files for the first source:
        // e.g. "output_sample1_read1.bam" → tail "_read1.bam"
        let first_file_prefix = format!("{prefix}{sep}{}", sources[0].1);
        let segment_tails: Vec<String> = {
            let mut tails: Vec<String> = std::fs::read_dir(output_directory)
                .context("Cannot list output directory")?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|name| name.starts_with(&first_file_prefix) && name.ends_with(&dot_suffix))
                .map(|name| name[first_file_prefix.len()..].to_string())
                .collect();
            tails.sort();
            tails
        };

        for tail in &segment_tails {
            let dst_name = if other_name.is_empty() {
                format!("{prefix}{tail}")
            } else {
                format!("{prefix}{sep}{other_name}{tail}")
            };
            let dst_path = output_directory.join(&dst_name);

            let src_paths: Vec<PathBuf> = sources
                .iter()
                .map(|(_, name)| output_directory.join(format!("{prefix}{sep}{name}{tail}")))
                .collect();

            merge_bam_files(&src_paths, &dst_path)?;

            for src in &src_paths {
                std::fs::remove_file(src)
                    .with_context(|| format!("Failed to remove intermediary: {}", src.display()))?;
            }

            if index {
                let bai_path = dst_path.with_extension("bam.bai");
                index_bam_file(&dst_path, &bai_path)?;
            }
        }
    }

    Ok(())
}

/// Remove the merge-step's name component from a combined demultiplex name.
///
/// Handles the cases where the step is first, last, or in the middle of the
/// combined name (e.g. `"A_B_C"` with step name `"B"` and sep `"_"` → `"A_C"`).
fn strip_step_name(combined: &str, step_name: &str, sep: &str) -> String {
    if combined == step_name {
        return String::new();
    }
    // Middle: sep + name + sep → sep
    let mid = format!("{sep}{step_name}{sep}");
    if let Some(pos) = combined.find(&mid) {
        return format!("{}{sep}{}", &combined[..pos], &combined[pos + mid.len()..]);
    }
    // First: name + sep
    let pfx = format!("{step_name}{sep}");
    if combined.starts_with(&pfx) {
        return combined[pfx.len()..].to_string();
    }
    // Last: sep + name
    let sfx = format!("{sep}{step_name}");
    if combined.ends_with(&sfx) {
        return combined[..combined.len() - sfx.len()].to_string();
    }
    // No match — return as-is (should not happen with well-formed config).
    combined.to_string()
}

/// Merge BAM files by reading and rewriting records via the noodles API.
///
/// The header is taken from the first source file.  All records from every
/// source are written in order to the destination.
fn merge_bam_files(src_paths: &[PathBuf], dst_path: &Path) -> Result<()> {
    // Read header from first source.
    let header = {
        let f = std::fs::File::open(&src_paths[0])
            .with_context(|| format!("Cannot open source: {}", src_paths[0].display()))?;
        let mut reader = bam::io::Reader::new(f);
        reader
            .read_header()
            .with_context(|| format!("Cannot read header from: {}", src_paths[0].display()))?
    };

    let dst_file = ex::fs::File::create(dst_path)
        .with_context(|| format!("Cannot create merged BAM: {}", dst_path.display()))?;
    let mut writer =
        bam::io::Writer::from(bgzf::io::Writer::new(std::io::BufWriter::new(dst_file)));
    writer
        .write_header(&header)
        .context("Failed to write BAM header")?;

    let mut record = bam::Record::default();
    for src_path in src_paths {
        let f = std::fs::File::open(src_path)
            .with_context(|| format!("Cannot open source BAM: {}", src_path.display()))?;
        let mut reader = bam::io::Reader::new(f);
        reader
            .read_header()
            .with_context(|| format!("Cannot read header from: {}", src_path.display()))?;
        loop {
            let n = reader
                .read_record(&mut record)
                .with_context(|| format!("Error reading record from: {}", src_path.display()))?;
            if n == 0 {
                break;
            }
            writer
                .write_record(&header, &record)
                .with_context(|| format!("Error writing record from: {}", src_path.display()))?;
        }
    }

    writer
        .try_finish()
        .context("Failed to finish merged BAM writer")?;
    Ok(())
}

/// Build a BAI index for `bam_path` and write it to `bai_path`.
///
/// Works for both sorted and unsorted BAMs (unmapped reads are counted;
/// mapped reads are indexed by position).
fn index_bam_file(bam_path: &Path, bai_path: &Path) -> Result<()> {
    use noodles::csi::binning_index::{Indexer, index::reference_sequence::bin::Chunk};

    let file = std::fs::File::open(bam_path)
        .with_context(|| format!("Cannot open BAM for indexing: {}", bam_path.display()))?;
    let mut reader = bam::io::Reader::new(file);
    let header = reader
        .read_header()
        .with_context(|| format!("Cannot read header for indexing: {}", bam_path.display()))?;

    let mut indexer = Indexer::default();
    let mut record = bam::Record::default();
    let mut start_pos = reader.get_ref().virtual_position();

    while reader
        .read_record(&mut record)
        .context("Error reading BAM record for indexing")?
        != 0
    {
        let end_pos = reader.get_ref().virtual_position();
        let chunk = Chunk::new(start_pos, end_pos);

        let ctx = alignment_context(&record)?;
        indexer.add_record(ctx, chunk).context("Failed to add record to BAM index")?;

        start_pos = end_pos;
    }

    let index = indexer.build(header.reference_sequences().len());
    bai::fs::write(bai_path, &index)
        .with_context(|| format!("Failed to write BAI: {}", bai_path.display()))?;

    Ok(())
}

fn alignment_context(
    record: &bam::Record,
) -> Result<Option<(usize, noodles_core::Position, noodles_core::Position, bool)>> {
    use noodles::sam::alignment::Record as _;
    Ok(
        match (
            record.reference_sequence_id().transpose()?,
            record.alignment_start().transpose()?,
            record.alignment_end().transpose()?,
        ) {
            (Some(id), Some(start), Some(end)) => {
                let is_mapped = !record.flags().is_unmapped();
                Some((id, start, end, is_mapped))
            }
            _ => None,
        },
    )
}
