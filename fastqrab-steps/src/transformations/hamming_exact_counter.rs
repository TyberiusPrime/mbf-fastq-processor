use anyhow::anyhow;
use bstr::ByteSlice;
use std::sync::{
    Condvar,
    atomic::{AtomicUsize, Ordering},
};

use crate::transformations::prelude::*;

/// This transformation counts exact hamming matches until we've got enough counts to over to
/// HammingCorrect in ByMajority mode.
///
/// It's created in expand_transformations
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
    seq_to_name: Arc<IndexMap<BString, String>>,
    pub barcode_counts: Arc<Mutex<IndexMap<BString, usize>>>,
    pub barrier: Arc<(Mutex<bool>, Condvar)>,
    blocks_counted: Arc<AtomicUsize>,
    pub blocks_to_count: usize,
    pub start_counting_in_hamming_at_this_block_no: Arc<AtomicUsize>,
    pub total_reads_considered: Arc<AtomicUsize>, // for verification purposes, not actual logic
}

impl PartialHammingExactCounter {
    pub(crate) fn new(
        in_label: TagLabel,
        seq_to_name: Arc<IndexMap<BString, String>>,
        blocks_to_count: usize,
    ) -> Self {
        Self {
            in_label: TomlValue::new_ok_unplaced(in_label),
            majority_data: Some(Arc::new(MajorityData {
                barcode_counts: Arc::new(Mutex::new(IndexMap::new())),
                seq_to_name,
                blocks_to_count,
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
        //the 0 blocks to count special case
        //is handled in HammingCorrect by not blocking at all in that case.

        if block.block_no() <= self.majority_data.blocks_to_count {
            // block no is 1 based.
            let mut local_exact_barcode_match_counter: IndexMap<BString, usize> = IndexMap::new();
            let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");
            for input_tag in input_tags {
                let seq = match input_tag {
                    TagValue::Missing => continue,
                    TagValue::Numeric(_) | TagValue::Bool(_) => unreachable!(), //cov:excl-line
                    TagValue::Location(hits) => BString::new(hits.joined_sequence(None)),
                    TagValue::String(bstring) => bstring.clone(),
                };
                let is_exact = self.majority_data.seq_to_name.contains_key(seq.as_bstr());
                if is_exact {
                    local_exact_barcode_match_counter
                        .entry(seq)
                        .and_modify(|count| *count = count.saturating_add(1))
                        .or_insert(1);
                }
            }
            self.majority_data
                .total_reads_considered
                .fetch_add(input_tags.len(), Ordering::SeqCst);
            {
                let mut bc = self.majority_data.barcode_counts.lock().map_err(|err| {
                    //cov:excl-start
                    anyhow!("Mutex poisoned while waiting for majority data to be ready: {err}")
                })?; //cov:excl-stop
                for (key, value) in local_exact_barcode_match_counter.into_iter() {
                    bc.entry(key)
                        .and_modify(|count| *count = count.saturating_add(value))
                        .or_insert(value);
                }
            }
            let mut counted = self
                .majority_data
                .blocks_counted
                .fetch_add(1, Ordering::SeqCst)
                + 1;
            if block.is_final {
                // we need to somehow delay for the other concurrent blocks to have been counted.
                // which means that blocks_counted == our block number, since the final block
                // always has the highest block_no().
                while self.majority_data.blocks_counted.load(Ordering::SeqCst) < block.block_no() {
                    //yeah it's a busy wait. Shouldn't last long though.
                    std::thread::yield_now();
                }
                counted = block.block_no(); // or reload blocks_counted, but this is cheaper
            }
            if block.is_final || counted == self.majority_data.blocks_to_count {
                self.signal_downstream_go(counted)?
            }
        }

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        // this is fine to run in parallel
        false
    }
}
