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
#[tpd]
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
    blocks_to_count: usize,
    pub start_counting_in_hamming_at_this_block_no: Arc<AtomicUsize>,
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
                barrier: Arc::new((Mutex::new(false), Condvar::new())), // We need to wait for the counter to be done before we
                start_counting_in_hamming_at_this_block_no: Arc::new(AtomicUsize::new(0)), //updated later.
            })),
        }
    }
}

impl VerifyIn<PartialConfig> for PartialHammingExactCounter {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure> {
        unreachable!("Created in code");
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
    fn signal_downstream_go(&self) {
        let (lock, cvar) = &*self.majority_data.barrier;
        let mut ready = lock.lock().expect("mutex poisened");
        *ready = true;
        cvar.notify_all();
    }
}

impl Step for HammingExactCounter {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        if self.majority_data.blocks_to_count == 0 && block.block_no() == 1{
            self.signal_downstream_go();
        } else {
            if block.block_no() < self.majority_data.blocks_to_count {
                dbg!("counting block", block.block_no());
                let mut local_exact_barcode_match_counter: IndexMap<BString, usize> =
                    IndexMap::new();
                let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");
                for input_tag in input_tags {
                    let seq = match input_tag {
                        TagValue::Missing => continue,
                        TagValue::Numeric(_) | TagValue::Bool(_) => unreachable!(),
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
                {
                    let mut bc = self
                        .majority_data
                        .barcode_counts
                        .lock()
                        .expect("mutex poisened");
                    for (key, value) in local_exact_barcode_match_counter.into_iter() {
                        bc.entry(key)
                            .and_modify(|count| *count = count.saturating_add(value))
                            .or_insert(value);
                    }
                }
                let counted = self
                    .majority_data
                    .blocks_counted
                    .fetch_add(1, Ordering::SeqCst)
                    + 1;
                dbg!(counted);
                if block.is_final {
                    dbg!(format!("block was final {}", block.block_no()));
                    // we need to somehow delay for the other concurrent blocks to have been counted.
                    // which means that blocks_counted == our block number,.
                    while self.majority_data.blocks_counted.load(Ordering::SeqCst)
                        < block.block_no()
                    {
                        //yeah it's a busy wait. Shouldn't last long though.
                        std::thread::yield_now();
                    }
                    dbg!("Stopped busy wait");
                }
                if block.is_final
                    || self.majority_data.blocks_counted.load(Ordering::SeqCst)
                        == self.majority_data.blocks_to_count
                {
                    self.signal_downstream_go()
                   
                }
            } else {
                dbg!("Skipping block", block.block_no());
            }
        }

        Ok((block, true))
    }
}
