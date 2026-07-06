use anyhow::anyhow;
use rustc_hash::FxHashMap;
use std::sync::{
    Condvar,
    atomic::{AtomicUsize, Ordering},
};

use crate::transformations::{hamming_correct::CountsFromReport, prelude::*};

fn build_atomic_counts(len: usize) -> Vec<AtomicUsize> {
    (0..len).map(|_| AtomicUsize::new(0)).collect()
}

/// This transformation counts exact hamming matches until we've got enough counts to over to
/// `HammingCorrect` in `ByMajority` mode.
///
/// It's created in `expand_transformations`
#[tpd(no_verify)]
#[derive(JsonSchema, Debug, Clone)]
pub struct HammingExactCounter {
    in_label: TagLabel,

    #[tpd(skip)]
    #[schemars(skip)]
    pub majority_data: Arc<MajorityData>,
}

#[derive(Debug, Clone)]
pub struct MajorityData {
    pub seq_to_name: Arc<IndexMap<BString, String>>,
    /// FxHash-backed sequence -> position-in-`seq_to_name` lookup. The default
    /// `IndexMap` hasher (`SipHash`) dominates the hot path on short DNA keys.
    pub seq_to_idx: Arc<FxHashMap<BString, usize>>,
    /// One counter per barcode in `seq_to_name`, indexed by its position there.
    pub barcode_counts: Arc<Vec<AtomicUsize>>,
    pub barrier: Arc<(Mutex<bool>, Condvar)>,
    blocks_counted: Arc<AtomicUsize>,
    /// Number of leading reads to count as the `ByMajority` warm-up sample
    /// (= `on_tie_min_molecules_to_start`). The warm-up covers whole blocks up
    /// to and including the one holding the `reads_to_count`-th read.
    pub reads_to_count: usize,
    pub start_counting_in_hamming_at_this_block_no: Arc<AtomicUsize>,
    pub total_reads_considered: Arc<AtomicUsize>, // for verification purposes, not actual logic
}

impl MajorityData {
    pub fn load_from_report(
        &self,
        mut report_file: impl std::io::Read,
        cfr: &CountsFromReport,
    ) -> Result<()> {
        //report json from the handle opened for cfr.filename, get the cfr.report_name,
        //then get 'tag_histogram',then the cfr.tag_name
        let mut raw = String::new();
        report_file.read_to_string(&mut raw).map_err(|err| {
            //cov:excl-start - outside our control
            anyhow!(
                "Failed to read counts from report file {}: {err}",
                cfr.filename
            )
            //cov:excl-stop
        })?; // cov:excl-line
        let json = serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
            anyhow!(
                "Failed to parse json for counts from report file {}: {err}",
                cfr.filename
            )
        })?;
        let entry = json.get(&cfr.report_name).ok_or_else(|| {
            anyhow!(
                "Report name {} not found in counts from report file {}",
                cfr.report_name,
                cfr.filename
            )
        })?;
        let tag_histogram = entry.get("histogram").ok_or_else(|| {
            anyhow!(
                "'tag_histogram' not found in report {} in counts from report file {}",
                cfr.report_name,
                cfr.filename
            )
        })?;
        let histogram = tag_histogram
            .get(&cfr.tag_name)
            .ok_or_else(|| anyhow!("Tag name {} not found in 'tag_histogram' of report {} in counts from report file {}", cfr.tag_name, cfr.report_name, cfr.filename))?;
        //now turn histogram into a string: usize map...
        for (barcode, count) in histogram.as_object().ok_or_else(|| anyhow!("Expected 'tag_histogram' entry for tag {} in report {} in counts from report file {} to be an object", cfr.tag_name, cfr.report_name, cfr.filename))? {
            if !barcode.is_empty() {
                let count = count.as_u64().ok_or_else(|| anyhow!("Expected count for barcode {} in 'tag_histogram' entry for tag {} in report {} in counts from report file {} to be a u64", barcode, cfr.tag_name, cfr.report_name, cfr.filename))?;
                if let Some(idx) = self.seq_to_idx.get(BStr::new(barcode.as_bytes())) {
                    self.barcode_counts[*idx].store(count.try_into().expect("Did not fit usize (32 bit system?)"), Ordering::SeqCst);
                } else {
                    //cov:excl-start
                    return Err(anyhow!("Barcode {} found in 'tag_histogram' entry \
                        for tag {} in report {} in counts from report file {}, \
                        but not found in barcodes used. Check that the barcodes section & your report matches", barcode, cfr.tag_name, cfr.report_name, cfr.filename));
                    //cov:excl-stop
                }
            }
        }
        Ok(())
    }
}

impl PartialHammingExactCounter {
    pub(crate) fn new(
        in_label: TagLabel,
        seq_to_name: Arc<IndexMap<BString, String>>,
        reads_to_count: usize,
    ) -> Self {
        let seq_to_idx: FxHashMap<BString, usize> = seq_to_name
            .keys()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        Self {
            in_label: TomlValue::new_ok_unplaced(in_label),
            majority_data: Some(Arc::new(MajorityData {
                barcode_counts: Arc::new(build_atomic_counts(seq_to_name.len())),
                seq_to_idx: Arc::new(seq_to_idx),
                seq_to_name,
                reads_to_count,
                blocks_counted: Arc::new(AtomicUsize::new(0)),
                barrier: Arc::new((Mutex::new(false), Condvar::new())),
                // We need to wait for the counter to be done before we can count reads.
                start_counting_in_hamming_at_this_block_no: Arc::new(AtomicUsize::new(0)), //updated later.
                total_reads_considered: Arc::new(AtomicUsize::new(0)),
            })),
        }
    }
}

impl TagUser for PartialTaggedVariant<PartialHammingExactCounter> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_label
                        .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                ],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl HammingExactCounter {
    fn signal_downstream_go(&self, count_after_block_no: usize) -> Result<()> {
        self.majority_data
            .start_counting_in_hamming_at_this_block_no
            .store(count_after_block_no, Ordering::SeqCst);
        let (lock, cvar) = &*self.majority_data.barrier;
        let mut ready = lock.lock().map_err(|err| {
            //cov:excl-start
            anyhow!("Mutex poisoned while waiting for majority data to be ready: {err}")
        })?; //cov:excl-stop
        *ready = true;
        cvar.notify_all();
        Ok(())
    }
}

impl Step for HammingExactCounter {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        //the 0 reads to count special case
        //is handled in HammingCorrect by not blocking at all in that case.

        // A block belongs to the warm-up sample iff its first read falls within
        // the first `reads_to_count` reads of the stream. Because blocks are
        // contiguous, sequentially numbered ranges of the global read stream,
        // `first_read_sequential_number` makes this an exact, order-independent
        // decision even though this stage runs in parallel and out of order.
        let reads_to_count = self.majority_data.reads_to_count;
        let first_read = block.first_read_sequential_number;
        if first_read < reads_to_count {
            // block no is 1 based.
            let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");
            let counts = &*self.majority_data.barcode_counts;
            match input_tags {
                TagColumn::Location(col) => {
                    for hits in col {
                        let idx = if hits.0.is_empty() {
                            continue;
                        } else {
                            let seq = hits.0;
                            self.majority_data.seq_to_idx.get(seq.as_ref()).copied()
                        };
                        if let Some(idx) = idx {
                            counts[idx].fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                TagColumn::String(items) => {
                    for item in items.iter() {
                        let idx = match item {
                            None => continue,
                            Some(bstr) => self.majority_data.seq_to_idx.get(bstr).copied(),
                        };
                        if let Some(idx) = idx {
                            counts[idx].fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                TagColumn::Numeric(_) | TagColumn::Bool(_) => unreachable!(), //cov:excl-line
            }
            self.majority_data
                .total_reads_considered
                .fetch_add(input_tags.len(), Ordering::SeqCst);
            self.majority_data
                .blocks_counted
                .fetch_add(1, Ordering::SeqCst);

            // This block holds the `reads_to_count`-th read (the "stop") iff the
            // threshold lands inside its range; that makes it the last warm-up
            // block. If the stream ends before reaching the threshold, the final
            // block takes that role instead. Either way it is the highest-numbered
            // warm-up block, so it owns the hand-off to `HammingCorrect`.
            let holds_stop = first_read + block.len() >= reads_to_count;
            if block.is_final || holds_stop {
                // Wait until every earlier warm-up block has been counted, so the
                // counts handed to HammingCorrect are complete. Warm-up blocks are
                // exactly 1..=block_no and each increments once, so
                // blocks_counted == our block_no means they are all in.
                while self.majority_data.blocks_counted.load(Ordering::SeqCst) < block.block_no() {
                    // yeah it's a busy wait. Shouldn't last long though.
                    std::thread::yield_now(); // cov:excl-line
                }
                self.signal_downstream_go(block.block_no())?;
            }
        }

        Ok((block, true))
    }

    #[mutants::skip]
    fn needs_serial(&self) -> bool {
        // this is fine to run in parallel
        false
    }
}
