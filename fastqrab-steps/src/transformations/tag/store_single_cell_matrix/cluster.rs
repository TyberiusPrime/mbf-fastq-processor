use std::{sync::OnceLock, time::Instant};

use super::{CellIdx, GeneIdx, Umi, helpers::hamming_bp_16};
use disjoint::DisjointSet;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

pub fn aggregate_to_matrix_cluster(
    entries: &[super::ObservedEvent],
    umi_length: u16,
) -> Vec<(GeneIdx, CellIdx, u32)> {
    debug_assert!(!entries.is_empty(), "Checked in caller");
    // Find split indices where (cell, gene) key changes
    // TODO: use chunk_by!
    let splits: Vec<usize> = (1..entries.len())
        .filter(|&i| {
            entries[i].gene != entries[i - 1].gene || entries[i].cell != entries[i - 1].cell
        })
        .collect();
    // Build range pairs [start, end) for each group
    let mut starts = Vec::with_capacity(splits.len() + 1);
    starts.push(0);
    let mut ends = splits.clone();
    ends.push(entries.len());
    starts.extend(splits);
    // Process each (gene, cell) group in parallel
    let results: Vec<(GeneIdx, CellIdx, u32)> = starts
        .into_par_iter()
        .zip(ends.into_par_iter())
        .filter_map(|(start, end)| {
            let cell_id = entries[start].cell;
            let gene_id = entries[start].gene;
            let key = (gene_id, cell_id);
            //todo: we don't need to check all of them for is_n()
            //just the last one.
            let mut seen: Vec<Umi> = entries[start..end]
                .iter()
                .filter(|e| !e.umi.is_n())
                .map(|e| e.umi)
                .collect();
            seen.dedup();
            if seen.is_empty() {
                None
            } else {
                let count = umi_cluster_count(&seen, umi_length);
                Some((key.0, key.1, count))
            }
        })
        .collect();
    results
}

pub fn umi_cluster_count(umis: &[Umi], umi_length: u16) -> u32 {
    if umis.len() == 1 {
        return 1;
    }
    debug_assert!(umis.len() >= 2);

    let values = umis;
    let n = values.len();

    //these branches do the same thing
    //but they differ in their O(n) and
    //we hence benchmark them on premise
    //to decide which ones to use
    let mut uf = DisjointSet::with_len(n);
    if n <= pairwise_threshold() { //mutants::skip - performance only
        pairwise_union(values, &mut uf);
    } else {
        //cov:excl-start
        neighbor_union_hash(values, &mut uf, umi_length);
        // but we verify accuracy in  test_pairwise_neighbor_aggreement
    }
    //cov:excl-stop

    let mut roots = FxHashSet::default();

    for i in 0..n {
        roots.insert(uf.root_of(i));
    }

    roots.len().try_into().expect("exceeded u32")
}

#[inline]
#[expect(clippy::needless_range_loop, reason = " Clearest way to state this")]
fn pairwise_union(values: &[Umi], uf: &mut DisjointSet) {
    for i in 0..values.len() {
        let x = values[i];

        for j in (i + 1)..values.len() {
            let y = values[j];

            // dist <= 1
            if hamming_bp_16(x, y) <= 1 {
                uf.join(i, j);
            }
        }
    }
}

#[inline]
#[mutants::skip]
fn neighbor_union_hash(values: &[Umi], uf: &mut DisjointSet, umi_length: u16) {
    assert!(
        umi_length <= 16,
        "UMI length must be at most 16bp to use neighbor_union_hash"
    );
    let mut index = FxHashMap::default();

    for (i, &x) in values.iter().enumerate() {
        index.insert(x, i);
    }

    for (i, &x) in values.iter().enumerate() {
        // dist == 1 basepair neighbors
        for bp in 0..umi_length {
            let shift = bp * 2;
            let current = (x.0 >> shift) & 0b11; //going the other ways hould be fine as well.

            for replacement in 0..4u32 {
                if replacement == current {
                    continue;
                }
                // Clear the 2 bits at this basepair, then set the replacement
                let y = (x.0 & !(0b11 << shift)) | (replacement << shift); 
                let y = Umi(y);

                if let Some(&j) = index.get(&y) {
                    uf.join(i, j);
                }
            }
        }
    }
}

/// It is unclear where the crossover between the O(n^2) pairwise, and O(32n) neighbor based
/// approach is. So we benchmark on the real system and make a decision.
static PAIRWISE_THRESHOLD: OnceLock<usize> = OnceLock::new();

#[mutants::skip]
fn pairwise_threshold() -> usize {
    *PAIRWISE_THRESHOLD.get_or_init(calibrate_pairwise_threshold)
}

#[mutants::skip]
fn calibrate_pairwise_threshold() -> usize {
    let candidates = [
        32usize,
        64,
        128,
        256,
        512,
        1024,
        1024 + 512,
        2048,
        2048 + 1024,
        4096,
    ];

    for &n in &candidates {
        // structured but non-trivial data
        let data: Vec<Umi> = (0..n.try_into().expect("candidates > u32::max"))
            .map(|x| x ^ (x << 1) ^ (x >> 1))
            .map(Umi)
            .collect();
        let hash_data: FxHashSet<Umi> = data.iter().copied().collect();
        assert!(hash_data.len() == data.len()); // ensure no duplicates, which would break the
        // benchmark

        // ----------------------------
        // PAIRWISE + DSU
        // ----------------------------
        let t_pair = {
            let mut uf = DisjointSet::with_len(n);

            let start = Instant::now();

            pairwise_union(&data, &mut uf);

            std::hint::black_box(uf);

            start.elapsed()
        };

        // ----------------------------
        // NEIGHBOR + DSU
        // ----------------------------
        let t_neighbor = {
            let mut uf = DisjointSet::with_len(n);

            let start = Instant::now();

            neighbor_union_hash(&data, &mut uf, 16);

            std::hint::black_box(uf);

            start.elapsed()
        };

        if t_neighbor < t_pair {
            // dbg!(format!(
            //     "Final threshold: {n}, time for pairwise: {t_pair:?}, time for neighbor: {t_neighbor:?}"
            // ));
            return n;
        }
    }
    *candidates.last().expect("must have candidate") //cov:excl-line might or might not get here
    // depending on runtime.
}

#[test]
fn test_pairwise_neighbor_aggreement() {
    let values = [Umi(91), Umi(93), Umi(94)]; //if you were mutating bitwise instead of bytewise, this will fail
    let n = values.len();
    let mut uf = DisjointSet::with_len(n);
    neighbor_union_hash(&values, &mut uf, 16);
    let mut roots = FxHashSet::default();
    for i in 0..n {
        roots.insert(uf.root_of(i));
    }
    let l_hash = roots.len();
    let mut uf = DisjointSet::with_len(n);
    pairwise_union(&values, &mut uf);
    let mut roots = FxHashSet::default();
    for (i, value) in values.iter().enumerate() {
        println!("{}, {}, {}", i, value.0, uf.root_of(i));
        roots.insert(uf.root_of(i));
    }
    let l_pairwise = roots.len();
    assert_eq!(
        l_hash, l_pairwise,
        "pairwise and neighbor approaches should give the same result"
    );
}
