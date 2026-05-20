use rustc_hash::FxHashSet;
use rayon::prelude::*;

use super::{CellIdx, GeneIdx, ObservedEvent, Umi};
use crate::transformations::prelude::*; // union_find_rs::prelude also exports a Result

pub fn aggregate_to_matrix_cellranger_like(
    entries: Vec<ObservedEvent>,
    umi_length: u16,
) -> Vec<(GeneIdx, CellIdx, u32)> {
    //cell ranger overview:
    //a) for each umi, assign to highest 1hamming neighbour.
    //b) for each cell-barcode & umi, kick out low-count genes / on tie remove all
    //entries are [cell_idx, gene_idx, umi_enc], sorted.
    //
    //
    //save entries to debug.entries

    let entries: Vec<_> = entries
        .par_chunk_by(|a, b| a.cell == b.cell)
        .map(|cell_chunk| {
            let mut result = Vec::new();
            let cell_idx = cell_chunk[0].cell;
            if cell_idx.is_unmatched() {
                return result;
            }
            let mut counts = count_umis(cell_chunk, umi_length);
            let corrections = correct_umis_to_next_by_hamming(&counts, umi_length);
            //cell ranger 3 special. Count 1 before
            for (raw_key, corrected_key) in &corrections {
                // One read has been counted before determining low-support UMI-genes.
                *counts.get_mut(raw_key).unwrap() -= 1;
                *counts.get_mut(corrected_key).unwrap() += 1;
            }

            let low_support_umigenes = find_umis_with_conflicting_genes(&counts);

            //now, all I care about is the number of remaining umis, per
            //gene, excluding the low_support_umigenes, right?
            for gene_chunk in cell_chunk.chunk_by(|a, b| a.gene == b.gene) {
                let mut distinct = FxHashSet::default();
                for event in gene_chunk {
                    if event.umi.is_homopolymer(umi_length)
                        || event.umi.is_n()
                        || event.gene.is_unmatched()
                    {
                        continue;
                    }
                    let corrected_gene_umi = corrections
                        .get(&(event.gene, event.umi))
                        .copied()
                        .unwrap_or((event.gene, event.umi));
                    if !low_support_umigenes.contains(&corrected_gene_umi) {
                        distinct.insert((event.gene, corrected_gene_umi.1));
                    }
                }
                let gene = gene_chunk[0].gene;
                result.push((gene, cell_idx, distinct.len() as u32));
            }
            result
        })
        .collect();
    entries.into_iter().flatten().collect()
}

fn count_umis(
    cell_gene_umis: &[ObservedEvent],
    umi_length: u16,
) -> FxIndexMap<(GeneIdx, Umi), u32> {
    let mut umi_counts: FxIndexMap<(GeneIdx, Umi), u32> = FxIndexMap::default();
    for event in cell_gene_umis {
        if !event.umi.is_homopolymer(umi_length) && !event.umi.is_n() && !event.gene.is_unmatched()
        {
            umi_counts
                .entry((event.gene, event.umi))
                .and_modify(|c| *c = c.saturating_add(1u32))
                .or_insert(1);
        }
    }
    umi_counts
}

///correct within one cellbarcode/gene
fn correct_umis_to_next_by_hamming(
    umi_counts: &FxIndexMap<(GeneIdx, Umi), u32>,
    umi_length: u16,
) -> FxIndexMap<(GeneIdx, Umi), (GeneIdx, Umi)> {
    //first we need accurate counts.
    let mut umis: Vec<((GeneIdx, Umi), u32)> = umi_counts.iter().map(|(a, b)| (*a, *b)).collect();
    umis.sort_unstable();
    let mut corrections = FxIndexMap::default();
    for gene_chunk in umis.chunk_by(|((gene, _), _), ((b_gene, _), _)| gene == b_gene) {
        for i in 0..gene_chunk.len() {
            let (x, x_count) = gene_chunk[i];
            let mut best = x;
            let mut best_count = x_count;

            let this_umi = x.1;
            for bp in 0..umi_length {
                let shift = bp * 2;
                let current = (this_umi.0 >> shift) & 0b11;

                for replacement in 0..4u32 {
                    if replacement == current {
                        continue;
                    }
                    // Clear the 2 bits at this basepair, then set the replacement
                    let test_umi = (this_umi.0 & !(0b11 << shift)) | (replacement << shift);
                    let test_umi = Umi(test_umi);

                    if let Some(&y_count) = umi_counts.get(&(x.0, test_umi)) {
                        let cmp = y_count.cmp(&best_count);
                        if (cmp == std::cmp::Ordering::Greater)
                            || (cmp == std::cmp::Ordering::Equal && test_umi > best.1)
                        {
                            best = (x.0, test_umi);
                            best_count = y_count;
                        }
                    }
                }
            }

            if best != x {
                corrections.insert(x, best);
            }
        }
    }
    corrections
}

fn find_umis_with_conflicting_genes(
    umi_counts: &FxIndexMap<(GeneIdx, Umi), u32>,
) -> FxHashSet<(GeneIdx, Umi)> {
    let mut low_support_umigenes: FxHashSet<(GeneIdx, Umi)> = FxHashSet::default();
    let mut umigene_count_vec: Vec<_> = umi_counts
        .iter()
        .map(|(&(gene, umi), &count)| (umi, gene, count))
        .collect();
    umigene_count_vec.sort();
    //chunk by umi...
    for gene_counts in umigene_count_vec.chunk_by(|x, y| x.0 == y.0) {
        if let Some((max_count, max_is_tied)) =
            gene_counts
                .iter()
                .copied()
                .fold(None, |acc, (_umi, _gene, count)| match acc {
                    None => Some((count, false)),
                    Some((m, tied)) => {
                        if count > m {
                            Some((count, false))
                        } else if count == m {
                            Some((count, true))
                        } else {
                            Some((m, tied))
                        }
                    }
                })
        {
            for (umi, gene, count) in gene_counts {
                if max_is_tied || *count < max_count {
                    low_support_umigenes.insert((*gene, *umi));
                }
            }
        }
    }
    low_support_umigenes
}

#[test]
fn test_find_umis_with_conflicting_genes() {
    let hm: FxIndexMap<(GeneIdx, Umi), u32> = vec![
        ((GeneIdx(773), Umi(1)), 2u32),
        ((GeneIdx(6396), Umi(1)), 1u32),
    ]
    .into_iter()
    .collect();
    let result = find_umis_with_conflicting_genes(&hm);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_umi_correction() {
    let counts: FxIndexMap<(GeneIdx, Umi), u32> = vec![
        ((GeneIdx(772), Umi(10849786)), 2),
        ((GeneIdx(772), Umi(10849738)), 1),
    ]
    .into_iter()
    .collect();
    dbg!(correct_umis_to_next_by_hamming(&counts, 12));
}
