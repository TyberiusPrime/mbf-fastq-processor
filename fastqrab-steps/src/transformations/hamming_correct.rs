use std::sync::Condvar;

use hamming_resonate::HammingResonator;
use indexmap::IndexMap;

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

    pub on_tie_min_molecules_for_frequency: Option<usize>,

    /// names are considered identical if they match up to the first name_split_character
    /// for must-have-hamming-distance considerations
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
    majority: Option<Arc<Mutex<MajorityData>>>,
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

#[derive(Debug)]
struct MajorityData {
    barcode_counts: IndexMap<BString, usize>,
    threshold: f64,
    reads_still_needed: usize,
    barrier: Arc<(Mutex<bool>, Condvar)>,
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
            match barcodes_data.map.get(barcodes_to_use) {
                Some(barcodes_section) => {
                    if let Some(barcodes_section) = barcodes_section.as_ref()
                        && let Some(seq_to_name) = &barcodes_section.seq_to_name
                        && let Some(max_hamming_distance) = self.max_hamming_distance.as_ref()
                    {
                        self.resonator = Some(Arc::new(init_hamming_resonator(
                            seq_to_name,
                            *max_hamming_distance,
                        )?));
                        self.seq_to_name = Some(seq_to_name.clone());
                    }
                    // otherwise the barcode section wasn't ok and we'll never
                    // be turned into a concrete HammingCorrect.
                }
                None => {
                    self.barcodes.help = Some(offer_alternatives(
                        barcodes_to_use.as_ref(),
                        &barcodes_data.map.keys().collect::<Vec<_>>(),
                    ));

                    self.barcodes.state = TomlValueState::ValidationFailed {
                        message: "Barcodes section not found".to_string(),
                    };
                    return Ok(());
                }
            }
        } else {
            let barcodes_empty = if let Some(Some(barcode_data)) = parent.barcodes.as_ref() {
                barcode_data.keys.is_empty() 
                } else {
                    matches!(parent.barcodes.as_ref(), Some(None))
            };
        if barcodes_empty 
        {
            return Err(ValidationFailure::new(
                "HammingCorrect step requires a barcodes section to be defined in the config.",
                Some(&format!("See {}", crate::link_docs("barcodes"))),
            ));
        }
        }

        if let Some(OnTie::ByMajority) = self.on_tie.as_ref() {
            self.on_tie_min_molecules_for_frequency
                .or_with(|| Some(1_000_000));
            let majority = MajorityData {
                barcode_counts: IndexMap::new(),
                threshold: 0.975, //subject to configuration later on?
                reads_still_needed: *self
                    .on_tie_min_molecules_for_frequency
                    .as_ref()
                    .and_then(|x| x.as_ref())
                    .expect("just set"),
                barrier: Arc::new((Mutex::new(false), Condvar::new())),
            };

            let blocks_in_flight: usize = parent
                .options
                .as_ref()
                .and_then(|options| options.max_blocks_in_flight.as_ref())
                .map(|x| *x)
                .expect("Blocks in flight should be set at this point in config");
            let reads_per_block = parent
                .options
                .as_ref()
                .and_then(|options| options.block_size.as_ref())
                .expect("reads_per_block must be set at this point");
            if blocks_in_flight * reads_per_block < majority.reads_still_needed {
                return Err(ValidationFailure::new(
                    "Not enough reads 'in flight' for ByMajority".to_string(),
                    Some(format!(
                        "Using on_tie=ByMajority must first collect enough data. \n\
                    It s configure to require {reads_still_needed} molecules.\n\
                    Your options.blocks_in_flight * options.reads_per_block only yield {reads_available} molecules\n\
                    Increase either one.\n\
                    Having a total number of reads below {reads_still_needed} is not a problem,\n\
                    ByMajority will simply use all reads.",
                        reads_available = blocks_in_flight * reads_per_block,
                        reads_still_needed = majority.reads_still_needed
                    )),
                ));
            }
            self.majority = Some(Some(Arc::new(Mutex::new(majority))));
        } else {
            self.majority = Some(None);
        }

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
}

impl TagUser for PartialTaggedVariant<PartialHammingCorrect> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(
                    match inner.output.as_ref().unwrap_or(&HammingOutput::Barcode) {
                        HammingOutput::Barcode  =>TagValueType::Location,
                        HammingOutput::Label => TagValueType::String
                    }),
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

#[derive(Debug)]
enum MatchResult<'a> {
    NoMatch,
    OneMatch(&'a BStr, bool),
    Tie(Vec<(&'a BStr, BString)>),
}

impl HammingCorrect {
    fn match_sequence(&self, sequence: &BStr) -> Result<MatchResult<'_>> {
        use MatchResult::*;
        let matched = self
            .resonator
            .query(&BStr::new(&sequence))
            .map_err(|e| anyhow::anyhow!("HammingCorrect query failed: {e}"))?;
        if matched.is_empty() {
            return Ok(NoMatch);
        } else if matched.len() == 1 {
            return Ok(OneMatch(matched[0].0, matched[0].1 == 0));
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
                return Ok(OneMatch(matched_plus_seq[0].0, false));
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
                    .expect("Cant have a matching barcode without a label")
                    .as_bytes()
                    .into()
            })
        }
    }
}

impl Step for HammingCorrect {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let mut output_hits = Vec::new();

        let mut hamming_hits = Vec::with_capacity(input_tags.len());
        let mut mj = if matches!(self.on_tie, OnTie::ByMajority) {
            Some(
                self.majority
                    .as_ref()
                    .expect("ByMajority means we have .majority")
                    .lock()
                    .expect("mutex poisened"),
            )
        } else {
            None
        };

        // we first match them, and count them if OnTie::ByMajority
        // to gain the maximum amount of information available.
        let mut skip_counting_until = 1;
        for input_tag in input_tags {
            let hit = match input_tag {
                TagValue::Missing => None,
                TagValue::Location(hits) => {
                    let seq = hits.joined_sequence(None);
                    Some(self.match_sequence(BStr::new(&seq))?)
                }
                TagValue::String(bstring) => Some(self.match_sequence(bstring.as_ref())?),
                TagValue::Numeric(_) | TagValue::Bool(_) => {
                    unreachable!("Validation was meant to prevent this situation. Bug?")
                }
            };
            if matches!(self.on_tie, OnTie::ByMajority)
                && let Some(mj) = mj.as_mut()
                && mj.reads_still_needed > 0
                && let Some(MatchResult::OneMatch(seq, was_exact)) = hit
                && was_exact
            {
                mj.reads_still_needed -= 1; //doesn't matter if we could use it, we decrease the
                // counter  - or our whole 'do we get enough reads' checking might be off.
                skip_counting_until += 1;

                let hits = self.resonator.query(seq)?;
                if hits.len() == 1 && hits[0].1 == 0 {
                    //only count perfect hits
                    mj.barcode_counts
                        .entry(seq.into())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
                if mj.reads_still_needed == 0 {
                    let mut ready = mj.barrier.0.lock().unwrap();
                    *ready = true;
                    mj.barrier.1.notify_all();
                }
            }
            hamming_hits.push(hit);
        }
        if matches!(self.on_tie, OnTie::ByMajority) {
            if block.is_final {
                let mj = mj.as_mut().expect("Must be there if ByMajority is set");
                let mut ready = mj.barrier.0.lock().unwrap();
                *ready = true;
                mj.barrier.1.notify_all();
            }

            let (lock, cvar) = &*Arc::clone(
                &mj.as_ref()
                    .expect("Must be there if ByMajority is set")
                    .barrier,
            );
            let _guard = cvar
                .wait_while(lock.lock().unwrap(), |ready| !*ready)
                .expect("Mutex poisened in condvar");
        }

        let output_barcode = matches!(self.output, HammingOutput::Barcode);
        for (read_no_in_block, (input_tag, hit)) in input_tags.iter().zip(hamming_hits).enumerate()
        {
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
                                    // Create hit with empty sequence
                                    if was_location && output_barcode {
                                        TagValue::Location(Hits(vec![]))
                                    }
                                    else {
                                        TagValue::String("".into())
                                    }
                                }
                                OnNoMatch::Keep => {
                                    // Keep original hit unchanged
                                    input_tag.clone()
                                }
                            }
                        }
                        MatchResult::OneMatch(matched_seq, was_exact) => {
                            if was_exact && let Some(mj) = mj.as_mut() && matches!(self.on_tie, OnTie::ByMajority) {
                                if read_no_in_block > skip_counting_until {
                                        mj.barcode_counts
                                    //matched_seq == query_seq here.
                                                    .entry(matched_seq.into())
                                                    .and_modify(|count| *count = count.saturating_add(1))
                                                    .or_insert(1);
                                }
                            }
                            self.output(matched_seq, input_tag, output_barcode)
                        }
                        MatchResult::Tie(items) => {

                            match self.on_tie {
                                OnTie::Remove => TagValue::Missing,
                                OnTie::Empty =>  {
                                    if was_location && output_barcode {
                                        TagValue::Location(Hits(vec![]))
                                    } else {
                                        TagValue::String("".into())
                                    }
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
                                        _ => unreachable!()
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
                                    let mj = mj.as_ref().expect("Majority must be set in OnTie::ByMajority");
                                    let mut total = 0;
                                    for item in &items {
                                        //add 1 to all counts,
                                        //or otherwise the first one would never get in.
                                        let count = mj.barcode_counts.get(item.0).map(|x| *x).unwrap_or(0) + 1;
                                        best = Some(match best {
                                            Some(ibest) => if ibest.1 < count {(item.0, count)} else {ibest},
                                            None => (item.0, count)
                                            });
                                        total += count;
                                    }
                                    let best = best.expect("Items can't have been empty");
                                    if best.1 as f64 / total as f64 >= mj.threshold {
                                        self.output(best.0, input_tag, output_barcode)
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

    #[doc = " does this transformation need to see all reads, or is it fine to run it in multiple"]
    #[doc = " threads in parallel?"]
    fn needs_serial(&self) -> bool {
        self.majority.is_some()
    }
}
