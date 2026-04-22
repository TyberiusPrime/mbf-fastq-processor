use anyhow::{Context, Result};
use noodles::bam::bai;
use noodles::{bam, bgzf, sam};
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
/// * `segment_tails` are the per-segment file suffixes derived from config (e.g. `["_read1.bam"]`).
#[allow(clippy::too_many_arguments)]
pub fn merge_demultiplexed_bam(
    output_directory: &Path,
    prefix: &str,
    sep: &str,
    demultiplex_infos: &[(usize, OptDemultiplex)],
    demultiplex_step_infos: &[DemultiplexStepInfo],
    merge_label: &str,
    segment_tails: &[String],
    index: bool,
) -> Result<()> {
    // ── find the merge step ──────────────────────────────────────────────────
    let merge_step_info = demultiplex_step_infos
        .iter()
        .find(|s| s.in_label == merge_label)
        .unwrap_or_else(|| {
            panic!("No demultiplex step found with in_label='{merge_label}' (this is a bug)")
        });

    let merge_mask = merge_step_info.merge_mask; //the bits for the step we're combining

    // ── final tag→name mapping ───────────────────────────────────────────────
    let final_tag_to_name = match demultiplex_infos.last() {
        Some((_, OptDemultiplex::Yes(info))) => &info.tag_to_name,
        _ => {
            // No demultiplexing active — nothing to merge.
            return Ok(());
        }
    };

    // ── build merge groups: other_tag → (other_name, [(combined_tag, combined_name, merge_part_name)]) ──
    let mut groups: BTreeMap<Tag, (String, Vec<(Tag, String, String)>)> = BTreeMap::new();

    for (combined_tag, combined_name_opt) in final_tag_to_name {
        let Some(combined_name) = combined_name_opt else {
            continue; // no output file for this tag
        };
        let other_tag = combined_tag & !merge_mask;

        let merge_part_tag = combined_tag & merge_mask;
        let merge_part_name = if merge_part_tag == 0 {
            "no-barcode".to_string()
        } else {
            merge_step_info
                .local_tag_to_name
                .get(&merge_part_tag)
                .cloned()
                .expect("missing merge-part-tag?")
        };

        let other_name = strip_step_name(combined_name, &merge_part_name, sep);

        let entry = groups
            .entry(other_tag)
            .or_insert_with(|| (other_name, Vec::new()));
        entry.1.push((*combined_tag, combined_name.clone(), merge_part_name));
    }

    if groups.is_empty() {
        return Ok(());
    }

    // ── read the shared header once and build reference-name → position map ──
    // All source files from the same pipeline run share an identical header.
    // The merge-part name (e.g. "lane1") is the reference sequence name, so
    // looking it up in the header gives the sort key without peeking at records.
    let first_tail = segment_tails.first().map(String::as_str).unwrap_or("");
    let header = {
        let (_, srcs) = groups.values().next().expect("groups non-empty");
        let (_, name, _) = srcs.first().expect("group has at least one source");
        let path = output_directory.join(format!("{prefix}{sep}{name}{first_tail}"));
        read_bam_header(&path)?
    };
    let ref_order: indexmap::IndexMap<String, usize> = header
        .reference_sequences()
        .keys()
        .enumerate()
        .map(|(i, name)| (name.to_string(), i))
        .collect();

    // ── merge each group ─────────────────────────────────────────────────────

    for (_, (other_name, mut sources)) in groups {
        // Sort by header position of the merge-part reference name; unmatched sort last.
        sources.sort_by_key(|(_, _, merge_part_name)| {
            ref_order
                .get(merge_part_name.as_str())
                .copied()
                .map(|id| id + 1)
                .unwrap_or(usize::MAX)
        });

        for tail in segment_tails {
            let dst_name = if other_name.is_empty() {
                format!("{prefix}{tail}")
            } else {
                format!("{prefix}{sep}{other_name}{tail}")
            };
            let dst_path = output_directory.join(&dst_name);

            let src_paths: Vec<PathBuf> = sources
                .iter()
                .map(|(_, name, _)| output_directory.join(format!("{prefix}{sep}{name}{tail}")))
                .collect();

            merge_bam_files(&header, &src_paths, &dst_path)?;

            for src in src_paths.iter().map(PathBuf::as_path) {
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

fn read_bam_header(path: &Path) -> Result<sam::Header> {
    let f = std::fs::File::open(path)
        .with_context(|| format!("Cannot open BAM for header: {}", path.display()))?;
    let mut reader = bam::io::Reader::new(f);
    reader
        .read_header()
        .with_context(|| format!("Cannot read header from: {}", path.display()))
}

/// Merge BAM files by streaming decompressed record bytes from each source into a single output.
///
/// The caller supplies the header (read once from any source); all sources share the same header.
fn merge_bam_files(header: &sam::Header, src_paths: &[PathBuf], dst_path: &Path) -> Result<()> {
    let dst_file = ex::fs::File::create(dst_path)
        .with_context(|| format!("Cannot create merged BAM: {}", dst_path.display()))?;
    let mut writer =
        bam::io::Writer::from(bgzf::io::Writer::new(std::io::BufWriter::new(dst_file)));
    writer
        .write_header(header)
        .context("Failed to write BAM header")?;

    for src_path in src_paths {
        let f = std::fs::File::open(src_path)
            .with_context(|| format!("Cannot open source BAM: {}", src_path.display()))?;
        let mut reader = bam::io::Reader::new(f);
        reader
            .read_header()
            .with_context(|| format!("Cannot read header from: {}", src_path.display()))?;
        std::io::copy(reader.get_mut(), writer.get_mut())
            .with_context(|| format!("Error streaming records from: {}", src_path.display()))?;
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
        indexer
            .add_record(ctx, chunk)
            .context("Failed to add record to BAM index")?;

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
