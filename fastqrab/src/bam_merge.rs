use anyhow::{Context, Result};
use noodles::{bam, sam};
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

// Standard BGZF EOF block (28 bytes), per the SAM/BGZF specification, section 4.1.2.
const BGZF_EOF: [u8; 28] = [
    0x1f, 0x8b, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x06, 0x00, 0x42, 0x43, 0x02, 0x00,
    0x1b, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

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
    reads_per_tag: &BTreeMap<Tag, u64>,
) -> Result<()> {
    // ── find the merge step ──────────────────────────────────────────────────
    let merge_step_info = demultiplex_step_infos
        .iter()
        .find(|s| s.in_label == merge_label)
        .unwrap_or_else(|| {
            panic!("No demultiplex step found with in_label='{merge_label}' (this is a bug)")
        });

    let merge_mask = merge_step_info.merge_mask;

    //  final tag→name mapping
    let final_tag_to_name = match demultiplex_infos.last() {
        Some((_, OptDemultiplex::Yes(info))) => &info.tag_to_name,
        _ => {
            // No demultiplexing active — nothing to merge.
            return Ok(());
        }
    };

    // build merge groups: other_tag → (other_name, [(combined_tag, combined_name, merge_part_name)])
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
        entry
            .1
            .push((*combined_tag, combined_name.clone(), merge_part_name));
    }

    if groups.is_empty() {
        return Ok(());
    }

    // read the shared header once and build reference-name → position map
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

    // merge each group

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

            let spans = merge_bam_files(&src_paths, &dst_path)?;

            for src in src_paths.iter().map(PathBuf::as_path) {
                std::fs::remove_file(src)
                    .with_context(|| format!("Failed to remove intermediary: {}", src.display()))?;
            }

            if index {
                let bai_path = dst_path.with_extension("bam.bai");
                write_merged_bai(
                    &bai_path,
                    &header,
                    &ref_order,
                    &sources,
                    &spans,
                    reads_per_tag,
                )?;
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

/// Merge BAM files by copying raw BGZF blocks verbatim (no decompression/recompression).
///
/// All source BAMs share the same header.  The header is copied from the first source.
/// Because `build_bam_output` flushes after writing the header, the header occupies exactly
/// the first `header_end_coffset` bytes of every source file, ending on a BGZF block boundary.
///
/// Returns the BGZF virtual-offset span `(v_beg, v_end)` for each source file, where the
/// virtual offset is `coffset << 16` (uoffset is always 0 at every BGZF block boundary).
fn merge_bam_files(src_paths: &[PathBuf], dst_path: &Path) -> Result<Vec<(u64, u64)>> {
    // ── locate where header ends in every source (all identical) ────────────
    let header_end_coffset: u64 = {
        let f = std::fs::File::open(&src_paths[0])
            .with_context(|| format!("Cannot open BAM: {}", src_paths[0].display()))?;
        let mut reader = bam::io::Reader::new(f);
        reader
            .read_header()
            .with_context(|| format!("Cannot read header from: {}", src_paths[0].display()))?;
        let vpos = u64::from(reader.get_ref().virtual_position());
        let uoffset = vpos & 0xFFFF;
        anyhow::ensure!(
            uoffset == 0,
            "BAM header must end on a BGZF block boundary (uoffset={uoffset}); \
             ensure build_bam_output flushes after write_header"
        );
        vpos >> 16
    };

    let mut dst = ex::fs::File::create(dst_path)
        .with_context(|| format!("Failed to create BAM file: {}", dst_path.display()))?;

    // ── copy header bytes verbatim from first source ─────────────────────────
    {
        let src = std::fs::File::open(&src_paths[0])
            .with_context(|| format!("Cannot open intermediate BAM: {}", src_paths[0].display()))?;
        let copied = std::io::copy(&mut src.take(header_end_coffset), &mut dst)
            .with_context(|| format!("Error copying header from: {}", src_paths[0].display()))?;
        if copied != header_end_coffset {
            anyhow::bail!(
                "Unexpected header size: copied {copied} bytes but expected {header_end_coffset} bytes"
            );
        }
    }

    // ── copy record blocks from each source (skip header + EOF block) ────────
    let mut total_compressed = header_end_coffset;
    let mut spans = Vec::with_capacity(src_paths.len());

    for src_path in src_paths {
        let v_beg = total_compressed << 16;

        let mut src = std::fs::File::open(src_path)
            .with_context(|| format!("Cannot open source BAM: {}", src_path.display()))?;
        let src_size = src
            .seek(SeekFrom::End(0))
            .with_context(|| format!("Cannot seek to end: {}", src_path.display()))?;
        src.seek(SeekFrom::Start(header_end_coffset))
            .with_context(|| format!("Cannot seek to end of header: {}", src_path.display()))?;

        let records_size = src_size
            .saturating_sub(header_end_coffset)
            .saturating_sub(BGZF_EOF.len() as u64);
        total_compressed += std::io::copy(&mut src.take(records_size), &mut dst)
            .with_context(|| format!("Error copying records from: {}", src_path.display()))?;

        let virtual_offset = total_compressed << 16;
        spans.push((v_beg, virtual_offset));
    }

    // ── terminate with BGZF EOF block ────────────────────────────────────────
    dst.write_all(&BGZF_EOF)
        .context("Failed to write BGZF EOF")?;

    Ok(spans)
}

/// Write a BAI index for a merged BAM whose reads all sit at reference position 1 (1-based).
///
/// All reads for a given reference come from one source file and are contiguous in the merged
/// BAM.  We therefore write a single chunk per reference rather than per-record chunks.
/// Virtual offsets are `coffset << 16` (uoffset 0) because `merge_bam_files` flushes between
/// sources so every chunk starts on a BGZF block boundary.
///
/// Bin 4681 covers [0, 16 384) on every reference — correct for reads at position 0 (0-based)
/// shorter than 16 384 bp, which holds for all typical short-read sequencing data.
fn write_merged_bai(
    bai_path: &Path,
    header: &sam::Header,
    ref_order: &indexmap::IndexMap<String, usize>,
    sources: &[(Tag, String, String)], // (combined_tag, combined_name, merge_part_name)
    spans: &[(u64, u64)],              // (v_beg, v_end) per source, same order as sources
    reads_per_tag: &BTreeMap<Tag, u64>,
) -> Result<()> {
    let n_ref = header.reference_sequences().len();

    // Build per-reference span + read count from sources.
    // Sources not in ref_order (e.g. "no-barcode") contribute to n_no_coor.
    let mut ref_spans: Vec<Option<(u64, u64, u64)>> = vec![None; n_ref];
    let mut n_no_coor: u64 = 0;

    for (i, (combined_tag, _, merge_part_name)) in sources.iter().enumerate() {
        let n_reads = reads_per_tag.get(combined_tag).copied().unwrap_or(0);
        if let Some(&ref_id) = ref_order.get(merge_part_name.as_str()) {
            let (v_beg, v_end) = spans[i];
            ref_spans[ref_id] = Some((v_beg, v_end, n_reads));
        } else {
            n_no_coor += n_reads;
        }
    }

    let f = ex::fs::File::create(bai_path)
        .with_context(|| format!("Cannot create BAI: {}", bai_path.display()))?;
    let mut w = std::io::BufWriter::new(f);

    // magic + n_ref
    w.write_all(b"BAI\x01")?;
    w.write_all(&(n_ref as u32).to_le_bytes())?;

    for ref_id in 0..n_ref {
        if let Some((v_beg, v_end, n_reads)) = ref_spans[ref_id] {
            // n_bin = 2: one real bin (4681) + metadata pseudo-bin (37450)
            w.write_all(&2u32.to_le_bytes())?;

            // bin 4681: covers [0, 16384) — one chunk spanning the entire source
            w.write_all(&4681u32.to_le_bytes())?;
            w.write_all(&1u32.to_le_bytes())?; // n_chunk
            w.write_all(&v_beg.to_le_bytes())?; // chunk_beg
            w.write_all(&v_end.to_le_bytes())?; // chunk_end

            // pseudo-bin 37450: metadata (ref_beg, ref_end, n_mapped, n_unmapped)
            w.write_all(&37450u32.to_le_bytes())?;
            w.write_all(&2u32.to_le_bytes())?; // n_chunk (4 u64s encoded as 2 chunks)
            w.write_all(&v_beg.to_le_bytes())?; // ref_beg
            w.write_all(&v_end.to_le_bytes())?; // ref_end
            w.write_all(&n_reads.to_le_bytes())?; // n_mapped
            w.write_all(&0u64.to_le_bytes())?; // n_unmapped

            // linear index: one 16kbp interval, covering position 0
            w.write_all(&1u32.to_le_bytes())?; // n_intv
            w.write_all(&v_beg.to_le_bytes())?; // ioffset[0]
        } else {
            // No reads for this reference.
            w.write_all(&0u32.to_le_bytes())?; // n_bin
            w.write_all(&0u32.to_le_bytes())?; // n_intv
        }
    }

    // Optional trailing field: unplaced unmapped reads.
    w.write_all(&n_no_coor.to_le_bytes())?;

    w.flush()
        .with_context(|| format!("Failed to flush BAI: {}", bai_path.display()))?;

    Ok(())
}
