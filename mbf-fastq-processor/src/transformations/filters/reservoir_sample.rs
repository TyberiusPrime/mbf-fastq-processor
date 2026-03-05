#![allow(clippy::unnecessary_wraps)]
//eserde false positives
use crate::dna::TagValue;
use crate::io::FastQBlock;
use crate::transformations::{extend_seed, prelude::*};
use rand::Rng;

#[derive(Clone, Debug, Default)]
struct ReservoirBuffer {
    segments: Vec<FastQBlock>,
    count: usize,
    tags: IndexMap<TagLabel, Vec<TagValue>>,
}

/// Fairly sample reads (expensive!)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReservoirSample {
    pub n: usize,
    pub seed: u64,
    #[tpd(skip, default)]
    #[schemars(skip)]
    runtime_data: Option<Arc<Mutex<DemultiplexedData<ReservoirBuffer>>>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    rng: Option<Arc<Mutex<Option<rand_chacha::ChaChaRng>>>>,
}

impl VerifyIn<PartialConfig> for PartialReservoirSample {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.n.verify(|n| {
            if *n == 0 {
                Err(ValidationFailure::new(
                    "n must be > 0. Set to a positive integer.",
                    None,
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialReservoirSample> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        TagUsageInfo {
            must_see_all_tags: true,
            ..Default::default()
        }
    }
}

impl Step for ReservoirSample {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &std::path::Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> anyhow::Result<Option<DemultiplexBarcodes>> {
        use rand_chacha::rand_core::SeedableRng;
        let extended_seed = extend_seed(self.seed);
        self.rng = Some(Arc::new(Mutex::new(Some(
            rand_chacha::ChaChaRng::from_seed(extended_seed),
        ))));
        self.runtime_data = Some(Arc::new(Mutex::new(DemultiplexedData::new())));
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut rng_lock = self.rng.as_ref().expect("rng not set in init").lock();
        let rng = rng_lock
            .as_mut()
            .expect("rng mutex poisoned")
            .as_mut()
            .expect("rng must be initialized before process()");

        let mut data_lock = self
            .runtime_data
            .as_ref()
            .expect("runtime_data not set in init")
            .lock();
        let data = data_lock.as_mut().expect("runtime_data mutex poisoned");

        let block_len = block.len();
        for pos in 0..block_len {
            let demultiplex_tag = block.output_tags.as_ref().map_or(0, |tags| tags[pos]);
            let buf = data.entry(demultiplex_tag).or_default();
            buf.count += 1;

            if buf.segments.is_empty() || buf.segments[0].len() < self.n {
                for (segment_no, segment) in block.segments.iter().enumerate() {
                    if buf.segments.len() <= segment_no {
                        buf.segments.push(FastQBlock::empty());
                    }
                    buf.segments[segment_no].append_read(&segment.get(pos));
                }
                for (label, values) in &block.tags {
                    buf.tags
                        .entry(label.clone())
                        .or_default()
                        .push(values[pos].clone());
                }
            } else {
                //algorithm R
                let j = rng.random_range(1..=buf.count);
                if j <= self.n {
                    for (ii, segment) in block.segments.iter().enumerate() {
                        buf.segments[ii].replace_read(j - 1, &segment.get(pos));
                    }
                    for (label, values) in &block.tags {
                        if let Some(tag_buf) = buf.tags.get_mut(label) {
                            tag_buf[j - 1] = values[pos].clone();
                        }
                    }
                }
            }
        }

        if block.is_final {
            //we gotta copy it all back together, so no easy just hand out our internal
            //storage, I suppose.
            let mut output = block.empty();
            let all_data = data.replace(DemultiplexedData::new());
            for (demultiplex_tag, buf) in all_data {
                if let Some(demultiplex_tags) = output.output_tags.as_mut() {
                    for _ in 0..buf.segments[0].len() {
                        demultiplex_tags.push(demultiplex_tag);
                    }
                }
                for (segment_no, molecule) in buf.segments.iter().enumerate() {
                    for read_idx in 0..molecule.entries.len() {
                        output.segments[segment_no].append_read(&molecule.get(read_idx));
                    }
                }
                for (label, values) in &buf.tags {
                    output
                        .tags
                        .entry(label.clone())
                        .or_default()
                        .extend(values.iter().cloned());
                }
            }
            Ok((output, true))
        } else {
            // Return empty block to continue processing, but preserve tag keys
            // so downstream steps (e.g. StoreTagsInTable) can discover tag labels
            // before the final block arrives.
            let mut empty = block.empty();
            for label in block.tags.keys() {
                empty.tags.insert(label.clone(), Vec::new());
            }
            Ok((empty, true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
