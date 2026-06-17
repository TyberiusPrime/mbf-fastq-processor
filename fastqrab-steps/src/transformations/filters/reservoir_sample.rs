use bstr::ByteSlice;
use rand::Rng;

use crate::transformations::{extend_seed, prelude::*};
use fastqrab_io::blocks::{FastQChunk, OwnedFastQRead};

#[derive(Clone, Debug, Default)]
struct ReservoirBuffer {
    molecules: Vec<OwnedMolecule>,
    count: usize,
    tags: IndexMap<TagLabel, TagColumnInAssembly>,
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
    ) -> Option<TagUsageInfo<'_>> {
        Some(TagUsageInfo {
            must_see_all_tags: true,
            ..Default::default()
        })
    }
}

impl Step for ReservoirSample {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
        _input_files: &mut StepInputFiles,
    ) -> anyhow::Result<Option<DemultiplexBarcodes>> {
        use rand_chacha::rand_core::SeedableRng;
        let extended_seed = extend_seed(self.seed);
        assert!(self.rng.is_none(), "init called twice");
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

        for (pos, molecule) in block.molecules().enumerate() {
            let demultiplex_tag = block.output_tags.as_ref().map_or(0, |tags| tags[pos]);
            let buf = data.entry(demultiplex_tag).or_default();
            buf.count += 1;

            if buf.molecules.len() < self.n {
                buf.molecules.push(molecule.into());
                for (label, values) in &block.tags {
                    buf.tags
                        .entry(label.clone())
                        .or_insert_with(|| TagColumnInAssembly::new_empty(values))
                        .push_from(values, pos);
                }
            } else {
                //algorithm R
                let j = rng.random_range(1..=buf.count);
                if j <= self.n {
                    buf.molecules[j - 1] = molecule.into();
                    for (label, values) in &block.tags {
                        if let Some(tag_buf) = buf.tags.get_mut(label) {
                            tag_buf.set_slot_from(j - 1, values, pos);
                        }
                    }
                }
            }
        }

        if block.is_final {
            //we gotta copy it all back together, so no easy just hand out our internal
            //storage, I suppose.
            let mut output = block.empty();
            let member_count = output.segments.len();
            let all_data = data.replace(DemultiplexedData::new());
            // Materialize the groups so we can rebuild every segment from the same
            // sampled molecules, preserving group order.
            let groups: Vec<(DemultiplexTag, ReservoirBuffer)> = all_data.into_iter().collect();

            // Rebuild each segment from every sampled molecule's read for that
            // segment index, in group order.
            for segment_no in 0..member_count {
                let reads: Vec<&OwnedFastQRead> = groups
                    .iter()
                    .flat_map(|(_, buf)| buf.molecules.iter().map(move |m| m.get(segment_no)))
                    .collect();
                output.segments[segment_no] = FastQChunk::from_owned_reads(reads);
            }

            if let Some(demultiplex_tags) = output.output_tags.as_mut() {
                for (demultiplex_tag, buf) in &groups {
                    for _ in 0..buf.molecules.len() {
                        demultiplex_tags.push(*demultiplex_tag);
                    }
                }
            }

            // Rebuild tag columns. Scalar columns (String/Numeric/Bool) are just
            // concatenated across groups. Location columns must be re-aliased
            // against the rebuilt segment chunk: the alias builder consumes that
            // segment's entries in order, so we feed one builder per label in the
            // exact same (group, molecule) order used to rebuild the segments
            // above. Output column order follows first appearance across groups.
            let mut labels: Vec<TagLabel> = Vec::new();
            for (_, buf) in &groups {
                for label in buf.tags.keys() {
                    if !labels.contains(label) {
                        labels.push(label.clone());
                    }
                }
            }

            for label in labels {
                let sample = groups
                    .iter()
                    .find_map(|(_, buf)| buf.tags.get(&label))
                    .expect("label came from some group");
                match sample {
                    TagColumnInAssembly::Location { segment, .. } => {
                        let segment = *segment;
                        // Scope the builder so its borrow of `output` ends before
                        // we insert into `output.tags`.
                        let col = {
                            let mut builder = output.location_column_builder(segment);
                            for (_, buf) in &groups {
                                if let Some(TagColumnInAssembly::Location { rows, .. }) =
                                    buf.tags.get(&label)
                                {
                                    for row in rows {
                                        builder.push_row(row);
                                    }
                                }
                            }
                            builder.finish()
                        };
                        output.tags.insert(label, TagColumn::Location(col));
                    }
                    TagColumnInAssembly::String(_) => {
                        let col: StringColumn = groups
                            .iter()
                            .filter_map(|(_, buf)| buf.tags.get(&label))
                            .flat_map(|c| match c {
                                TagColumnInAssembly::String(items) => items.iter(),
                                _ => unreachable!("tag kind is consistent across groups"),
                            })
                            .map(|x| x.as_ref().map(|x| x.as_bstr()))
                            .collect();
                        output.tags.insert(label, TagColumn::String(col));
                    }
                    TagColumnInAssembly::Numeric(_) => {
                        let col: Vec<f64> = groups
                            .iter()
                            .filter_map(|(_, buf)| buf.tags.get(&label))
                            .flat_map(|c| match c {
                                TagColumnInAssembly::Numeric(items) => items.iter().copied(),
                                _ => unreachable!("tag kind is consistent across groups"),
                            })
                            .collect();
                        output.tags.insert(label, TagColumn::Numeric(col));
                    }
                    TagColumnInAssembly::Bool(_) => {
                        let col: Vec<bool> = groups
                            .iter()
                            .filter_map(|(_, buf)| buf.tags.get(&label))
                            .flat_map(|c| match c {
                                TagColumnInAssembly::Bool(items) => items.iter().copied(),
                                _ => unreachable!("tag kind is consistent across groups"),
                            })
                            .collect();
                        output
                            .tags
                            .insert(label, TagColumn::Bool(col.into_iter().collect()));
                    }
                }
            }
            Ok((output, true))
        } else {
            // Return empty block to continue processing, but preserve tag keys
            // (and kinds) so downstream steps (e.g. StoreTagsInTable) can discover
            // tag labels before the final block arrives. A location column needs a
            // source pod to alias, so build an empty one against `empty`'s
            // (zero-row) segment, preserving its source segment.
            let mut empty = block.empty();
            for (label, column) in &block.tags {
                let new_col = match column {
                    TagColumn::Location(_) => {
                        let segment = column.location_segment();
                        TagColumn::Location(empty.location_column_builder(segment).finish())
                    }
                    TagColumn::String(_) => TagColumn::String(StringColumn::empty()),
                    TagColumn::Numeric(_) => TagColumn::Numeric(Vec::new()),
                    TagColumn::Bool(_) => TagColumn::Bool(Vec::<bool>::new().into_iter().collect()),
                };
                empty.tags.insert(label.clone(), new_col);
            }
            Ok((empty, true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}

/// Location tag columns are zero-copy references into the source `FastQChunk`'s
/// buffers. The reservoir keeps its sampled reads as owned molecules (a less
/// compact `Vec`), not a chunk, so it can't hold those aliases — instead it
/// stashes each row's *read-relative* `(start, len)` regions (still valid
/// against the owned read's bytes) plus the source `segment`, and re-aliases
/// them against the rebuilt segment chunk when the final block is emitted (see
/// the `is_final` branch of [`ReservoirSample::apply`]).
#[derive(Clone, Debug)]
pub enum TagColumnInAssembly {
    Location {
        segment: SegmentIndex,
        rows: Vec<SmallVec<[(u32, u32); 1]>>,
    },
    String(Vec<Option<BString>>),
    Numeric(Vec<f64>),
    Bool(Vec<bool>),
}

impl TagColumnInAssembly {
    /// An empty assembly column of the same kind as `source`, carrying the
    /// originating segment for `Location` columns (see
    /// [`TagColumn::location_segment`]).
    fn new_empty(source: &TagColumn) -> Self {
        match source {
            TagColumn::Location(_) => TagColumnInAssembly::Location {
                segment: source.location_segment(),
                rows: Vec::new(),
            },
            TagColumn::String(_) => TagColumnInAssembly::String(Vec::new()),
            TagColumn::Numeric(_) => TagColumnInAssembly::Numeric(Vec::new()),
            TagColumn::Bool(_) => TagColumnInAssembly::Bool(Vec::new()),
        }
    }

    /// Append the value at `pos` of `source` (same kind as `self`, guaranteed by
    /// construction via [`new_empty`](Self::new_empty)).
    fn push_from(&mut self, source: &TagColumn, pos: usize) {
        match self {
            TagColumnInAssembly::Location { rows, .. } => rows.push(source.get_location(pos)),
            TagColumnInAssembly::String(items) => {
                items.push(source.get_string(pos).map(|x| x.to_owned()))
            }
            TagColumnInAssembly::Numeric(items) => items.push(source.get_numeric(pos)),
            TagColumnInAssembly::Bool(items) => items.push(source.get_bool(pos)),
        }
    }

    /// Overwrite slot `slot` with the value at `pos` of `source` (reservoir
    /// replacement). Same-kind precondition as [`push_from`](Self::push_from).
    fn set_slot_from(&mut self, slot: usize, source: &TagColumn, pos: usize) {
        match self {
            TagColumnInAssembly::Location { rows, .. } => rows[slot] = source.get_location(pos),
            TagColumnInAssembly::String(items) => {
                items[slot] = source.get_string(pos).map(ToOwned::to_owned)
            }
            TagColumnInAssembly::Numeric(items) => items[slot] = source.get_numeric(pos),
            TagColumnInAssembly::Bool(items) => items[slot] = source.get_bool(pos),
        }
    }
}
