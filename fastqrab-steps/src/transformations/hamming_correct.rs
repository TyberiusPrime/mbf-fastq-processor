use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::anyhow;
use hamming_resonate::HammingResonator;
use indexmap::IndexMap;

use super::hamming_exact_counter::MajorityData;
use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_dna::dna::init_hamming_resonator;

/// Correct a tag (extracted region) to known barcodes

#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct HammingCorrect {
    /// Input tag to correct
    pub in_label: TagLabel,
    /// Output tag to store corrected result
    pub out_label: TagLabel,
    /// Reference to barcodes section
    pub barcodes: TagLabel,
    /// Maximum hamming distance for correction
    pub max_hamming_distance: u8,

    #[tpd(default)]
    pub output: HammingOutput,

    /// What to do when no match is found
    pub on_no_match: OnNoMatch,

    /// What to do when more than one match an the same distance is found
    pub on_tie: OnTie,

    #[tpd(alias = "by_majority_min_molecules_to_start")]
    pub on_tie_min_molecules_to_start: usize,
    #[tpd(alias = "by_majority_threshold")]
    pub on_tie_threshold: f64,

    #[tpd(default)]
    pub on_tie_dump_counts: bool,
    #[tpd(skip)]
    #[schemars(skip)]
    count_writer: Arc<Mutex<Option<ChunkedRecordWriter>>>,

    /// names are considered identical if they match up to the first `name_split_character`
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "name_split_char")]
    pub name_split_character: Option<u8>,

    #[tpd(skip)]
    #[schemars(skip)]
    seq_to_name: Arc<IndexMap<BString, String>>,

    #[tpd(skip)]
    #[schemars(skip)]
    resonator: Arc<HammingResonator>,

    #[tpd(skip)]
    #[schemars(skip)]
    pub majority_data: Option<Arc<MajorityData>>,

    #[tpd(skip)]
    #[schemars(skip)]
    reads_in_this_step: AtomicUsize, // for verification that we really count correctly between
                                     // HammingExactCounter and HammingCorrect in ByMajority mode
}

#[tpd]
#[derive(Debug, JsonSchema, Default)]
pub enum HammingOutput {
    #[tpd(alias = "DNA", alias = "barcodes")]
    #[default]
    Barcode,
    #[tpd(alias = "labels")]
    Label,
}

impl VerifyIn<PartialConfig> for PartialHammingCorrect {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        //defaults first, error returns later
        self.reads_in_this_step = Some(AtomicUsize::new(0));
        self.on_tie_min_molecules_to_start.or(1_000_000);
        self.on_tie_threshold.or(0.975);

        if let Some(out_label) = self.out_label.as_ref()
            && let Some(in_label) = self.in_label.as_ref()
            && out_label == in_label
        {
            let spans = vec![
                (self.in_label.span(), "The same as out_label".to_string()),
                (self.out_label.span(), "The same as in_label".to_string()),
            ];
            self.out_label.state = TomlValueState::Custom { spans };
            self.out_label.help =
                Some("Please use different tag names for the input and output labels to avoid overwriting the source tag.".to_string())
                ;
        }
        //0 for max_hamming_distance is a perfectly valid value if you want to use it as a lookup
        if let Some(barcodes_to_use) = self.barcodes.as_ref()
            && let Some(Some(barcodes_data)) = parent.barcodes.as_ref()
        {
            if let Some(barcodes_section) = barcodes_data.map.get(barcodes_to_use) {
                if let Some(barcodes_section) = barcodes_section.as_ref()
                    && let Some(seq_to_name) = &barcodes_section.seq_to_name
                    && let Some(max_hamming_distance) = self.max_hamming_distance.as_ref()
                {
                    self.resonator = Some(Arc::new(init_hamming_resonator(
                        seq_to_name,
                        *max_hamming_distance,
                    )?)); // cov:excl-line // we check length before, so this shouldn't fail.
                    self.seq_to_name = Some(seq_to_name.clone());
                } //cov:excl-line
            // otherwise the barcode section wasn't ok and we'll never
            // be turned into a concrete HammingCorrect.
            } else {
                self.barcodes.help = Some(offer_alternatives(
                    barcodes_to_use.as_ref(),
                    &barcodes_data.map.keys().collect::<Vec<_>>(),
                ));

                self.barcodes.state = TomlValueState::ValidationFailed {
                    message: "Barcodes section not found".to_string(),
                };
                return Ok(());
            }
        } else if self.barcodes.as_ref().is_some()
            && !matches!(parent.barcodes.value.as_ref(), Some(Some(_)))
        {
            self.barcodes.help = Some(
                "Add a [barcodes.<name>] section in your TOML to define the barcodes.".to_string(),
            );
            self.barcodes.state =
                TomlValueState::new_validation_failed("No barcodes sections defined".to_string());
            return Ok(());
        }

        self.on_tie_threshold.verify(|x| {
            if *x < 0.0 || *x >= 1.0 {
                Err(ValidationFailure::new(
                    "Must be >= 0 <= 1".to_string(),
                    Some("Supply a valid fraction between (0..1)".to_string()),
                ))
            } else {
                Ok(())
            }
        });
        if matches!(self.on_tie.as_ref(), Some(OnTie::ByEditProbability)) {
            self.max_hamming_distance.verify(|dist| {
                if *dist != 1 {
                    Err(ValidationFailure::new(
                        "ByEditProbability requires max_hamming_distance == 1".to_string(),
                        Some("Set max_hamming_distance to 1".to_string()),
                    ))
                } else {
                    Ok(())
                }
            });
        }

        if matches!(
            self.on_tie.as_ref(),
            Some(OnTie::ByMajority | OnTie::ByEditProbability)
        ) {
            let blocks_in_flight: usize = parent
                .options
                .as_ref()
                .and_then(|options| options.max_blocks_in_flight.as_ref())
                .copied()
                .unwrap_or_else(fastqrab_config::default_blocks_in_flight);
            let reads_per_block = parent
                .options
                .as_ref()
                .and_then(|options| options.block_size.as_ref())
                .copied()
                .unwrap_or_else(|| fastqrab_config::default_block_size().into());
            let reads_wanted = *self
                .on_tie_min_molecules_to_start
                .as_ref()
                .expect("just set above");
            if !reads_wanted.is_multiple_of(reads_per_block) {
                return Err(ValidationFailure::new(
                    format!(
                        "on_tie_min_molecules_to_start must be a multiple of options.block_size ({reads_per_block})"
                    ),
                    Some(
                        "Adjust either on_tie_min_molecules_to_start or options.block_size"
                            .to_string(),
                    ),
                ));
            }

            if blocks_in_flight * reads_per_block < reads_wanted {
                return Err(ValidationFailure::new(
                    "Not enough reads 'in flight' for ByMajority|ByEditProbability".to_string(),
                    Some(format!(
                        "Using on_tie=ByMajority (or ByEditProbability) must first collect enough data. \n\
                    It is configured to require {reads_wanted} molecules.\n\
                    Your options.blocks_in_flight * options.reads_per_block only yield {reads_available} molecules.\n\
                    Increase either one.\n\
                    Having a total number of reads below {reads_wanted} is not a problem,\n\
                    ByMajority|ByEditProbability will simply use all reads.",
                        reads_available = blocks_in_flight * reads_per_block,
                    )),
                ));
            }
        }
        self.majority_data = Some(None); //get's overwritten in expand_transformations for ByMajority, empty default otherwise
        self.count_writer = Some(Arc::new(Mutex::new(None)));

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[tpd]
pub enum OnNoMatch {
    Remove,
    Empty,
    Keep,
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[tpd]
pub enum OnTie {
    Remove,
    Empty,
    Keep,
    First,
    FirstStrict,
    Fail,
    ByMajority, // cell ranger like, 0.975, but we update blockwise, instead of reading everything
    // at once
    ByEditProbability,
}

impl TagUser for PartialTaggedVariant<PartialHammingCorrect> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let input_kinds = if let Some(OnTie::ByEditProbability) = inner.on_tie.as_ref() {
                &[TagValueType::Location][..] //needs qualities
            } else {
                &[TagValueType::String, TagValueType::Location][..]
            };
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(
                    match inner.output.as_ref().unwrap_or(&HammingOutput::Barcode) {
                        HammingOutput::Barcode => TagValueType::Location,
                        HammingOutput::Label => TagValueType::String,
                    },
                ),
                used_tags: vec![inner.in_label.to_used_tag(input_kinds)],
                used_barcodes: inner.barcodes.as_ref().cloned().into_iter().collect(),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }

    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        if let Some(inner) = self.toml_value.as_ref() {
            if inner.on_tie_dump_counts.as_ref().is_some_and(|x| *x)
                && let Some(in_label) = inner.in_label.as_ref()
            {
                return vec![OutputDeclaration {
                    id: "counts".to_string(),
                    target: WriteTargetConfig::new(
                        vec![format!("{}.counts", in_label)],
                        "tsv".to_string(),
                    ),
                    sink_config: SinkConfig::new_uncompressed_unhashed(),
                    format: FileFormat::Text,
                    chunk_policy: ChunkPolicy::no_chunks(),
                    bam_options: None,
                    singleton: true,
                    span: inner.on_tie_dump_counts.span(),
                }];
            }
        }
        return vec![];
    }
}

#[derive(Debug)]
enum MatchResult<'a> {
    NoMatch,
    OneMatch(&'a BStr, bool),
    Tie(Vec<(&'a BStr, BString)>),
}

impl HammingCorrect {
    fn match_sequence(&self, sequence: &BStr) -> Result<MatchResult<'_>> {
        use MatchResult::{NoMatch, OneMatch, Tie};

        if let Some((key, _value)) = self.seq_to_name.get_key_value(sequence) {
            Ok(OneMatch(key.as_ref(), true))
        } else {
            let matched = self
                .resonator
                .query(sequence)
                .map_err(|e| anyhow::anyhow!("HammingCorrect query failed: {e}"))?;
            if matched.is_empty() {
                Ok(NoMatch)
            } else if matched.len() == 1 {
                Ok(OneMatch(matched[0].0, matched[0].1 == 0))
            } else {
                let mut matched_plus_seq: Vec<_> = matched
                    .iter()
                    .map(|(seq, dist)| {
                        let seq_name: BString = self
                            .seq_to_name
                            .get(*seq)
                            .expect("Must be in there?!")
                            .as_bytes()
                            .into();
                        let seq_name = match self.name_split_character {
                            Some(split_char) => seq_name
                                .splitn(2, |&c| c == split_char)
                                .next()
                                .unwrap_or(&seq_name)
                                .into(),
                            None => seq_name,
                        };
                        (*seq, dist, seq_name)
                    })
                    .collect();

                matched_plus_seq.sort_by_key(|(seq, dist, _name)| (*dist, *seq));
                // is there a best one? take that
                if matched_plus_seq[0].1 < matched_plus_seq[1].1 {
                    Ok(OneMatch(matched_plus_seq[0].0, *matched_plus_seq[0].1 == 0))
                } else {
                    let first_different = matched_plus_seq
                        .iter()
                        .position(|(_seq, dist, _name)| *dist > matched_plus_seq[0].1)
                        .unwrap_or(matched_plus_seq.len());
                    matched_plus_seq.truncate(first_different);
                    Ok(Tie(matched_plus_seq
                        .into_iter()
                        .map(|(seq, _dist, name)| (seq, name))
                        .collect()))
                }
            }
        }
    }

    fn output(&self, matched_seq: &BStr, input_tag: &TagValue, output_barcode: bool) -> TagValue {
        if let TagValue::Location(hit_sequences) = input_tag
            && output_barcode
        {
            let new_hit = Hit {
                sequence: matched_seq.into(),
                location: if hit_sequences.0.len() == 1 {
                    hit_sequences.0[0].location.clone()
                } else {
                    None
                },
            };
            TagValue::Location(Hits::new_multiple(vec![new_hit]))
        } else {
            TagValue::String(if output_barcode {
                matched_seq.into()
            } else {
                self.seq_to_name
                    .get(matched_seq)
                    .expect("Cant have a matching barcode without a label. One of the correction functions is returning an uncorrected barcode when it should return TagValue::Missing?")
                    .as_bytes()
                    .into()
            })
        }
    }

    fn output_empty(was_location: bool, output_barcode: bool) -> TagValue {
        if was_location && output_barcode {
            TagValue::Location(Hits(vec![]))
        } else {
            TagValue::String("".into())
        }
    }
}

impl Step for HammingCorrect {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        if self.on_tie_dump_counts {
            let mut count_dump_file = output_files.take("counts");
            let writer = count_dump_file
                .remove(&0)
                .expect("tag 0 writer must exist, this is singleton:true");
            *self.count_writer.lock().expect("poisoned") = Some(writer);
        }

        Ok(None)
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");
        self.reads_in_this_step
            .fetch_add(input_tags.len(), Ordering::SeqCst);

        let mut output_hits = Vec::new();

        let (mut barcode_counts, count_here) =
            if matches!(self.on_tie, OnTie::ByMajority | OnTie::ByEditProbability) {
                //ByMajority is serialized anyway, so no trouble keeping this lock for the runtime
                //of this function
                let mj = self
                    .majority_data
                    .as_ref()
                    .expect("ByMajority means we have .majority");
                if mj.blocks_to_count > 0 {
                    let (guard, cv) = &*mj.barrier.clone();
                    let _guard = cv
                .wait_while(
                    guard.lock().map_err(|err| {
                        anyhow!("Mutex poisoned while waiting for majority data to be ready: {err}") // cov:excl-line
                    })?, // cov:excl-line
                    |counting_done| !*counting_done,
                )
                .expect("mutex inside condvar poisoned");
                }
                let count_here = block.block_no()
                    > mj.start_counting_in_hamming_at_this_block_no
                        .load(Ordering::Acquire);
                if count_here {
                    mj.total_reads_considered
                        .fetch_add(input_tags.len(), Ordering::SeqCst);
                }
                (
                    Some(mj.barcode_counts.lock().expect("Mutex poisoned?")),
                    count_here,
                )
            } else {
                (None, false)
            };
        let output_barcode = matches!(self.output, HammingOutput::Barcode);
        let mut read_iter = block.get_pseudo_iter();
        for input_tag in input_tags {
            let read = read_iter
                .pseudo_next()
                .context("Read & tag count mismatch!?")?;
            let hit = match input_tag {
                TagValue::Missing => None,
                TagValue::Location(hits) => {
                    let seq = hits.joined_sequence(None);
                    Some(self.match_sequence(BStr::new(&seq))?)
                }
                TagValue::String(bstring) => Some(self.match_sequence(bstring.as_ref())?),
                TagValue::Numeric(_) | TagValue::Bool(_) => {
                    unreachable!("Validation was meant to prevent this situation. Bug?") // cov:excl-line
                }
            };

            output_hits.push(match hit {
                None => TagValue::Missing,
                Some(hit) => {
                    let was_location = matches!(input_tag, TagValue::Location(_));
                    match hit {
                        MatchResult::NoMatch => {
                            match self.on_no_match {
                                OnNoMatch::Remove => {
                                    // Create empty tag value
                                    TagValue::Missing
                                }
                                OnNoMatch::Empty => {
                                    Self::output_empty(was_location, output_barcode)
                                }
                                OnNoMatch::Keep => {
                                    // Keep original hit unchanged
                                    input_tag.clone()
                                }
                            }
                        }
                        MatchResult::OneMatch(matched_seq, was_exact) => {
                            if was_exact && let Some(barcode_counts) = barcode_counts.as_mut()
                                //&& matches!(self.on_tie, OnTie::ByMajority | OnTie::ByEditProbability)
                                && count_here {
                                        barcode_counts
                                    //matched_seq == query_seq here.
                                                    .entry(matched_seq.into())
                                                    .and_modify(|count| *count = count.saturating_add(1))
                                                    .or_insert(1);
                                }
                            self.output(matched_seq, input_tag, output_barcode)
                        }
                        MatchResult::Tie(items) => {
                            match self.on_tie {
                                OnTie::Remove => TagValue::Missing,
                                OnTie::Empty =>  {
                                    Self::output_empty(was_location, output_barcode)
                                }
                                OnTie::Keep => input_tag.clone(),
                                OnTie::First =>{
                                    self.output(items[0].0, input_tag, output_barcode)
                                }
                                OnTie::FirstStrict => {
                                    //self.name_split_character splitting happens in match
                                    let all_the_same = items.iter().all(|(_seq, name)| *name == items[0].1);
                                    if all_the_same {
                                        self.output(items[0].0, input_tag, output_barcode)
                                    } else {
                                        TagValue::Missing
                                    }
                                }
                                OnTie::Fail => {
                                    let query_seq = match input_tag {
                                        TagValue::Location(hit) => hit.joined_sequence(None),
                                        TagValue::String(seq) => seq.to_vec(),
                                        _ => unreachable!() // cov:excl-line
                                    };
                                    bail!(
                                    "HammingCorrect on in_label={} \n\
                                     Uncorrectable sequence '{}', \n\
                                     matches multiple sequences within hamming distance: {:?}.\n\
                                     Set `on_tie` to one of Keep, Remove, Empty, First, or FirstStrict to resolve ties.\n\
                                     If using FirstStrict, consider setting `name_split_character`?",
                                    self.in_label,
                                    BStr::new(&query_seq),
                                    items
                                );
                                }
                                OnTie::ByMajority => {
                                    let mut best: Option<(&BStr, usize)> = None;
                                    let barcode_counts = barcode_counts.as_ref().expect("Barcode_counts must be set in OnTie::ByMajority");
                                    let mut total = 0;
                                    for item in &items {
                                        //add a laplace of 1
                                        //which avoids a total of 0 in the extreme case of ,
                                        //we ain't seen any of these.
                                        let count = barcode_counts.get(item.0).copied().unwrap_or(0) + 1;
                                        best = Some(match best {
                                            Some(ibest) => if ibest.1 < count {(item.0, count)} else {ibest},
                                            None => (item.0, count)
                                            });
                                        total += count;
                                    }
                                    let best = best.expect("Items can't have been empty");
                                    #[expect(clippy::cast_precision_loss, reason="If lengths reach f64 imprecison region, precision loss would be acceptable")]
                                    if best.1 as f64 / total as f64 >= self.on_tie_threshold {
                                        self.output(best.0, input_tag, output_barcode)
                                    } else {
                                        TagValue::Missing
                                    }

                                }
                                OnTie::ByEditProbability => {
                                    let barcode_counts = barcode_counts.as_ref().expect("Barcode_counts must be set in OnTie::ByEditProbability");
                                    let candidates: Vec<_> = items
                                        .iter()
                                        .map(|(seq, _name)| (*seq, barcode_counts.get(*seq).copied().unwrap_or(0)))
                                        .collect();
                                    let (observed_sequence, observed_qualities) =
                                            if let TagValue::Location(hit) = input_tag {
                                                (hit.joined_sequence(None), read.hit_to_qualities(hit))
                                            } else if let TagValue::String(_seq) = input_tag { // cov:excl-line
                                                unreachable!("Validation should have prevented ByEditProbability on String tags") // cov:excl-line
                                            } else {
                                                unreachable!() // cov:excl-line
                                            };
                                    let observed_qualities = observed_qualities.with_context(||format!("Hamming correction with ByEditProbability impossible.\n\
                                            The location tag {in_label} has lost it's location data (due to editing), can't retrieve qualities.\n\
                                            Maybe you can reorder your steps?", in_label=self.in_label))?;
                                    if let Some(best_seq) = correct_barcode_via_base_editing_likelihood(
                                        self.on_tie_threshold,
                                        BStr::new(&observed_sequence),
                                        &observed_qualities,
                                        &candidates,
                                    ) {
                                        self.output(best_seq, input_tag, output_barcode)
                                    } else {
                                        TagValue::Missing
                                    }
                                }
                            }
                        }
                    }
                }
            }
            );
        }
        // Add the corrected tags to the output
        block.tags.insert(self.out_label.clone(), output_hits);

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        self.majority_data.is_some()
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        if let Some(mj) = self.majority_data.as_ref() {
            assert_eq!(
                mj.total_reads_considered.load(Ordering::Acquire),
                self.reads_in_this_step.load(Ordering::Acquire),
                "Mismatch between OnTie::ByMajority considered reads and total reads in this step - bug in your count_here decision making"
            );
            if let Some(mut writer) = self.count_writer.lock().expect("Mutex poisoned").take() {
                let barcode_counts = mj.barcode_counts.lock().expect("Mutex poisoned");
                let mut records = barcode_counts
                    .iter()
                    .map(|(seq, count)| format!("{}\t{}\n", seq, count))
                    .collect::<Vec<_>>();
                records.sort(); //sort by sequence for easier diffing between runs
                writer.write_text_record(b"Barcode\tCount\n")?;
                for record in records {
                    writer.write_text_record(&record.as_bytes())?;
                }
                let _ = writer.finish()?;
            }
        }
        Ok(None)
    }
}

/// Compute the posterior over Hamming-1 'known' neighbors of `observed`,
/// weighting each by the per-base error probability at the differing position
/// and a Laplace-smoothed prior count. Returns the corrected barcode if its
/// posterior mass exceeds `p_threshold`, otherwise `None` (ambiguous).
///
/// `qual` is Illumina Phred+33 (clamped to Q66, matching `BC_MAX_QV`).
/// `candidates` must each be at Hamming distance exactly 1 from `observed`
/// and have the same length.
pub fn correct_barcode_via_base_editing_likelihood<'a>(
    p_threshold: f64,
    observed: &BStr,
    qual: &[u8],
    candidates: &'a [(&'a BStr, usize)],
) -> Option<&'a BStr> {
    debug_assert_eq!(observed.len(), qual.len());

    let mut total = 0.0_f64;
    let mut best: Option<(f64, &'a BStr)> = None;

    for &(cand, raw_count) in candidates {
        debug_assert_eq!(cand.len(), observed.len());

        let diff_pos = observed
            .iter()
            .zip(cand.iter())
            .position(|(a, b)| a != b)
            .expect("Candidates must be at 1 hamming distance from observed sequence.");

        let qv = qual[diff_pos].min(66); // we clamp it here to prevent sequencer overconfidence /
        // actually let the prior from the barcode count count.
        let phred_edit_probability = 10f64.powf(-(f64::from(qv) - 33.0) / 10.0);
        #[expect(
            clippy::cast_precision_loss,
            reason = "If counts reach f64 imprecison region, precision loss would be acceptable"
        )]
        let likelihood = phred_edit_probability * (1 + raw_count) as f64;

        total += likelihood;

        // Tiebreak on likelihood ties by lex-greater sequence
        // hence tuple comparison.
        let tup = Some((likelihood, cand));
        if tup > best {
            best = tup;
        }
    }

    let (best_like, best_seq) = best?;
    (best_like / total >= p_threshold).then_some(best_seq)
}

#[cfg(test)]
mod test_correct_barcode_via_base_editing_likelihood {
    use super::correct_barcode_via_base_editing_likelihood;
    use bstr::BStr;

    fn b(s: &[u8]) -> &BStr {
        BStr::new(s)
    }

    #[test]
    fn single_candidate_passes_threshold() {
        // With one candidate, mass = 1.0, regardless of quality / count.
        let observed = b(b"ACGT");
        let qual = b"IIII"; // Q40
        let candidates = [(b(b"ACGA"), 0usize)];
        let res = correct_barcode_via_base_editing_likelihood(0.5, observed, qual, &candidates);
        assert_eq!(res, Some(b(b"ACGA")));
    }

    #[test]
    fn no_candidates_returns_none() {
        let observed = b(b"ACGT");
        let qual = b"IIII";
        let candidates: [(&BStr, usize); 0] = [];
        let res = correct_barcode_via_base_editing_likelihood(0.5, observed, qual, &candidates);
        assert_eq!(res, None);
    }

    #[test]
    fn picks_lower_quality_position_when_counts_equal() {
        // Differing positions get different qualities.
        // Higher edit-probability (lower Q) wins when counts match.
        let observed = b(b"AAAAAAAA");
        // pos 0 = '!' (Q0, p_edit = 1.0); pos 4 = 'I' (Q40, p_edit = 1e-4).
        let qual: &[u8] = &[b'!', b'I', b'I', b'I', b'I', b'I', b'I', b'I'];
        // cand1 differs at pos 0 (low quality -> high p_edit -> wins)
        // cand2 differs at pos 4 (high quality -> low p_edit -> loses)
        let candidates = [(b(b"TAAAAAAA"), 0usize), (b(b"AAAATAAA"), 0usize)];
        let res = correct_barcode_via_base_editing_likelihood(0.5, observed, qual, &candidates);
        assert_eq!(res, Some(b(b"TAAAAAAA")));
    }

    #[test]
    fn higher_count_wins_when_qualities_equal() {
        let observed = b(b"AAAAAAAA");
        let qual: &[u8] = &[b'I'; 8];
        // Both differ at pos 0; counts decide.
        let candidates = [(b(b"TAAAAAAA"), 1usize), (b(b"CAAAAAAA"), 99usize)];
        let res = correct_barcode_via_base_editing_likelihood(0.5, observed, qual, &candidates);
        assert_eq!(res, Some(b(b"CAAAAAAA")));
    }

    #[test]
    fn ambiguous_returns_none_below_threshold() {
        // Two equal candidates -> each gets 0.5 of the mass; 0.99 threshold not reached.
        let observed = b(b"AAAAAAAA");
        let qual: &[u8] = &[b'!', b'I', b'I', b'I', b'!', b'I', b'I', b'I'];
        // Both diff positions are Q0 -> p_edit = 1, counts equal -> 50/50 split.
        let candidates = [(b(b"TAAAAAAA"), 5usize), (b(b"AAAATAAA"), 5usize)];
        let res = correct_barcode_via_base_editing_likelihood(0.99, observed, qual, &candidates);
        assert_eq!(res, None);
        //but if both of them reach the threshold, we take the lexicographic first one
        let res = correct_barcode_via_base_editing_likelihood(0.50, observed, qual, &candidates);
        assert_eq!(res, Some(b(b"TAAAAAAA")));
    }

    #[test]
    fn quality_clamped_to_q66() {
        // Quality bytes above Q66 (b'!' + 66 = 99 = 'c') get clamped to Q66.
        // Two candidates at the same diff position, with extremely high quality,
        // should still be distinguishable by count.
        let observed = b(b"AAAA");
        let qual: &[u8] = &[b'~', b'~', b'~', b'~']; // '~' = 126, beyond Q66
        let candidates = [(b(b"TAAA"), 0usize), (b(b"CAAA"), 10usize)];
        let res = correct_barcode_via_base_editing_likelihood(0.5, observed, qual, &candidates);
        assert_eq!(res, Some(b(b"CAAA")));
    }
}
