use bstr::{BString, ByteSlice};
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_dna::dna::init_hamming_resonator;
use hamming_resonate::HammingResonator;

use crate::transformations::prelude::*;

/// Assign each query sequence to the closest entry in a named reference database.
///
/// The reference is a FASTA or FASTQ file (optionally gzip-compressed).  Every
/// sequence in the file must have the same length as the query.  The step
/// builds a Hamming-distance index at initialisation and stores the matched
/// reference name as a string tag.  When no entry is within `max_hamming_distance`
/// the tag is set to `Missing`, which lets a downstream `FilterByTag` discard
/// unassigned reads.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct AssignByHalves {
    /// Tag containing the query sequence (must be a String or Location tag
    /// whose sequences have the same length as the reference entries).
    pub in_label: TagLabel,

    /// Output tag where the matched reference name is stored.
    pub out_label: TagLabel,
    ///
    /// Reference to barcodes section
    pub barcodes: TagLabel,

    /// names are considered identical if they match up to the first `name_split_character`
    /// for must-have-hamming-distance considerations
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "name_split_char")]
    pub name_split_character: Option<u8>,

    // ── built during init ───────────────────────────────────────────────────
    #[tpd(skip)]
    #[schemars(skip)]
    engine: Arc<CellRangerProbeAssigner>,
}

impl VerifyIn<PartialConfig> for PartialAssignByHalves {
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
                Some("Use different tag names for in_label and out_label.".to_string());
        }

        if let Some(barcodes_to_use) = self.barcodes.as_ref()
            && let Some(Some(barcodes_data)) = parent.barcodes.as_ref()
        {
            if let Some(barcodes_section) = barcodes_data.map.get(barcodes_to_use) {
                if let Some(barcodes_section) = barcodes_section.as_ref()
                    && let Some(seq_to_name) = &barcodes_section.seq_to_name
                {
                    self.engine = Some(Arc::new(CellRangerProbeAssigner::new(
                        Arc::new( //todo: remove once toml-pretty-deser supports FxIndexMap
                            seq_to_name
                                .iter()
                                .map(|(x, y)| (x.to_owned(), y.to_owned()))
                                .collect(),
                        ),
                        if let Some(Some(nc)) = self.name_split_character.as_ref() {
                            Some(*nc)
                        } else {
                            None
                        },
                    )?)); // cov:excl-line
                } // cov:excl-line
            // otherwise the barcode section wasn't ok and we'll never
            // be turned into a concrete AssignToReference.
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

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialAssignByHalves> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::String),
                //todo: should we output quality or such, multiple tags?
                used_tags: vec![
                    inner
                        .in_label
                        .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                ],
                used_barcodes: inner.barcodes.as_ref().cloned().into_iter().collect(),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for AssignByHalves {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let engine = &self.engine;
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let mut output_tags: Vec<TagValue> = Vec::with_capacity(input_tags.len());

        for input_tag in input_tags {
            let hit = match input_tag {
                TagValue::String(s) => engine
                    .query(s.as_ref())
                    .map_err(|e| anyhow::anyhow!("AssignToProbe query failed: {e}"))?,
                TagValue::Location(hits) => engine
                    .query(BStr::new(&hits.joined_sequence(None)))
                    .map_err(|e| anyhow::anyhow!("AssignToProbe query failed: {e}"))?,
                TagValue::Missing => None,
                TagValue::Bool(_) | TagValue::Numeric(_) => unreachable!(), // cov:excl-line
            };

            let tag_value = match hit {
                Some(reference_id) => TagValue::String(reference_id.as_bytes().into()),
                None => TagValue::Missing,
            };

            output_tags.push(tag_value);
        }

        block.tags.insert(self.out_label.clone(), output_tags);
        Ok((block, true))
    }
}

#[derive(Debug)]
struct CellRangerProbeAssigner {
    seq_to_name: Arc<FxIndexMap<BString, String>>,
    name_to_seq: FxIndexMap<String, BString>,
    left_hand_resonator: HammingResonator,
    left_hand_seq_to_name: IndexMap<BString, String>,
    right_hand_resonator: HammingResonator,
    right_hand_seq_to_name: IndexMap<BString, String>,
    rescue_min_score: i32,
    name_split_char: Option<u8>,
}

impl CellRangerProbeAssigner {
    fn new(
        seq_to_name: Arc<FxIndexMap<BString, String>>,
        name_split_char: Option<u8>,
    ) -> Result<Self, ValidationFailure> {
        let max_hamming_distance_for_better_half = 1;
        let left_hand_resonator = init_hamming_resonator(
            &(seq_to_name
                .iter()
                .map(|(k, v)| (k[..k.len() / 2].into(), v.clone()))
                .collect::<IndexMap<BString, String>>()),
            max_hamming_distance_for_better_half,
        )?; // cov:excl-line
        let left_hand_seq_to_name = seq_to_name
            .iter()
            .map(|(k, v)| (k[..k.len() / 2].into(), v.clone()))
            .collect();

        let right_hand_resonator = init_hamming_resonator(
            &(seq_to_name
                .iter()
                .map(|(k, v)| (k[k.len() / 2..].into(), v.clone()))
                .collect::<IndexMap<BString, String>>()),
            max_hamming_distance_for_better_half,
        )?;
        let right_hand_seq_to_name = seq_to_name
            .iter()
            .map(|(k, v)| (k[k.len() / 2..].into(), v.clone()))
            .collect();

        let name_to_seq = seq_to_name
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        // if seq_to_name.len() != name_to_seq.len() { // why actuall?
        //     return Err(ValidationFailure::new(
        //         "Duplicate probe names found in barcodes section.",
        //         Some("For this assignment method, all barcodes must have unique names"),
        //     ));
        // }
        Ok(Self {
            seq_to_name,
            name_to_seq,
            left_hand_resonator,
            left_hand_seq_to_name,
            right_hand_resonator,
            right_hand_seq_to_name,
            rescue_min_score: 30,
            name_split_char,
        })
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "can't have loss at reasonable lengths"
    )]
    fn query(&self, query: &BStr) -> Result<Option<&str>> {
        if let Some(name) = self.seq_to_name.get(query) {
            Ok(Some(name))
        } else {
            //the 'rescue' algorithm is simple
            //either the left or the right hand half have to match within 1-hamming
            //to exactly one barcode.
            //If they do, look up the correct other half.
            //Calculate the hamming from that, and a score,
            //and if the score of both together is >= rescue_min_score
            //accept it.
            //
            //Scoring is matches - mismatches == len() - 2 * mismatches
            let left_hand = query[..query.len() / 2].into();
            let right_hand = query[query.len() / 2..].into();
            let mut left_hand_matches = self.left_hand_resonator.query(left_hand)?;
            let mut right_hand_matches = self.right_hand_resonator.query(right_hand)?;

            if left_hand_matches.len() > 1 {
                left_hand_matches = self.combine_name_identical_tied_matches(
                    &left_hand_matches,
                    &self.left_hand_seq_to_name,
                );
            }
            if right_hand_matches.len() > 1 {
                right_hand_matches = self.combine_name_identical_tied_matches(
                    &right_hand_matches,
                    &self.right_hand_seq_to_name,
                );
            }

            if left_hand_matches.len() == 1 {
                //attempt rescue
                let (left_hand_seq, left_hand_distance) = &left_hand_matches[0];
                let left_hand_name = self
                    .left_hand_seq_to_name
                    .get(left_hand_seq.as_bstr())
                    .expect("Internal inconsistency between resonator and map");
                let expected_right_hand_side = self
                    .name_to_seq
                    .get(left_hand_name)
                    .expect("Unexpected mismatch between maps?");
                let right_hand_distance = hamming_resonate::hamming_distance(
                    &expected_right_hand_side[expected_right_hand_side.len() / 2..],
                    &query[query.len() / 2..],
                );

                let score_left = ((query.len() / 2) as i32) - (2 * *left_hand_distance as i32);
                let score_right =
                    ((query.len() - query.len() / 2) as i32) - (2 * right_hand_distance as i32);
                if score_right > 0 && (score_left + score_right >= self.rescue_min_score) {
                    //if !self.right_hand_seq_to_name.contains_key(right_hand) {
                    if right_hand_matches.is_empty()
                        || right_hand_matches.iter().all(|(rhs, _)| {
                            self.right_hand_seq_to_name
                                .get(rhs.as_bstr())
                                .expect("Internal inconsistency between resonator and map")
                                == left_hand_name
                        })
                    {
                        return Ok(Some(left_hand_name.as_str()));
                    }
                }
            } else {
                //too many. Or none.
            }

            //left hand didn't work. Mirror approach the other way
            if right_hand_matches.len() == 1 {
                //attempt rescue
                let (right_hand_seq, right_hand_distance) = &right_hand_matches[0];
                let right_hand_name = self
                    .right_hand_seq_to_name
                    .get(right_hand_seq.as_bstr())
                    .expect("Internal inconsistency between resonator and map");
                let expected_left_hand_side = self
                    .name_to_seq
                    .get(right_hand_name)
                    .expect("Unexpected mismatch between maps?");
                let left_hand_distance = hamming_resonate::hamming_distance(
                    &expected_left_hand_side[..expected_left_hand_side.len() / 2],
                    &query[..query.len() / 2],
                );

                let score_left = ((query.len() / 2) as i32) - (2 * left_hand_distance as i32);
                let score_right =
                    ((query.len() - query.len() / 2) as i32) - (2 * *right_hand_distance as i32);
                if score_left > 0 && (score_left + score_right >= self.rescue_min_score) {
                    //if !self.left_hand_seq_to_name.contains_key(left_hand) {
                    //
                    if left_hand_matches.is_empty()
                        || left_hand_matches.iter().all(|(lhs, _)| {
                            self.left_hand_seq_to_name
                                .get(lhs.as_bstr())
                                .expect("Internal inconsistency between resonator and map")
                                == right_hand_name
                        })
                    {
                        return Ok(Some(right_hand_name.as_str()));
                    }
                }
            } else {
                //too many. can't correct. Or no hits.
            }
            Ok(None)
        }
    }

    fn combine_name_identical_tied_matches<'b>(
        &'b self,
        hits: &Vec<(&'b BStr, u32)>,
        half_seq_to_name: &IndexMap<BString, String>,
    ) -> Vec<(&'b BStr, u32)> {
        fn split_name(name: &str, name_split_char: Option<u8>) -> &str {
            match name_split_char {
                None => name,
                Some(split_char) => name
                    .split_once(split_char as char)
                    .map_or(name, |(first_part, _)| first_part),
            }
        }

        debug_assert!(!hits.is_empty(), "can not be called without hits");
        let all_distances_equal = hits.iter().all(|(_, dist)| *dist == hits[0].1);
        if all_distances_equal {
            let first_name = split_name(
                half_seq_to_name
                    .get(hits[0].0)
                    .expect("Internal inconsistency between resonator and map"),
                self.name_split_char,
            );
            let all_names_equal = hits.iter().all(|(seq, _)| {
                let name = split_name(
                    half_seq_to_name
                        .get(seq.as_bstr())
                        .expect("Internal inconsistency between resonator and map"),
                    self.name_split_char,
                );
                name == first_name
            });
            if all_names_equal {
                return vec![hits[0]];
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::CellRangerProbeAssigner;

    #[test]
    fn test_assigner() {
        let seq_to_name = Arc::new(
            [
                (
                    "AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into(),
                    //format!("{}CCCCCGGGGG{}", "A".repeat(20), "T".repeat(20)).into(),
                    "probe1".to_string(),
                ),
                (
                    //format!("{}GGGGGCCCCC{}", "T".repeat(20), "A".repeat(20)).into(),
                    "TTTTTTTTTTTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAAAA".into(),
                    "probe2".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let engine = CellRangerProbeAssigner::new(seq_to_name, None).unwrap();
        //perfect queries
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into())
                .unwrap(),
            Some("probe1")
        );
        assert_eq!(
            engine
                .query("TTTTTTTTTTTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAAAA".into())
                .unwrap(),
            Some("probe2")
        );

        //one mismatch in left half
        assert_eq!(
            engine
                .query("TAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into())
                .unwrap(),
            Some("probe1")
        );

        //lhs perfect, rhs has 2 mismatches (rescue)
        //
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTATTTATTTT".into())
                .unwrap(),
            Some("probe1")
        );
        //split mapped
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCCCCCCAAAAAAAAAAAAAAAAAAAA".into())
                .unwrap(),
            None
        );
        //perfect lhs, 7 mismatches in RHS
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTCCCCCCCC".into())
                .unwrap(),
            Some("probe1")
        );

        //perfect lhs, 6 mismatches in RHS
        assert_eq!(
            engine
                .query("TAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTCCCCCCCT".into())
                .unwrap(),
            Some("probe1")
        );
        //one lhs mismatch, 9 mismatches in RHS
        assert_eq!(
            engine
                .query("TAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTCCCCCCCCCT".into())
                // "AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into(),
                .unwrap(),
            Some("probe1")
        );
        //one lhs mismatch, 10 mismatches in RHS
        assert_eq!(
            engine
                .query("TAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTCCCCCCCCCC".into())
                // "AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into(),
                .unwrap(),
            None
        );

        //now let's change the right half
        assert_eq!(
            engine
                .query("TTTTTTTTTTTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAATA".into())
                .unwrap(),
            Some("probe2")
        );
        //now let's change the right half
        assert_eq!(
            engine
                .query("XXXXXXXXXTTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAATA".into())
                .unwrap(),
            Some("probe2")
        );
        assert_eq!(
            engine
                .query("XXXXXXXXXXTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAATA".into())
                .unwrap(),
            None
        );

        assert_eq!(
            engine
                .query("XXXXXXXXXXTTTTTTTTTTGGGGGCCCCCAAAAAAAAAAAAAAAAAAAA".into())
                .unwrap(),
            Some("probe2")
        );
        //split mapped -> none
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCCCCCCAAAAAAAAAAAAAAAAAAAA".into())
                .unwrap(),
            None
        );
        assert_eq!(
            engine
                .query("TTTTTTTTTTTTTTTTTTTTGGGGGGGGGGTTTTTTTTTTTTTTTTTTTT".into())
                .unwrap(),
            None
        );

        //split mapped, but would maptch first with mismatches
        let seq_to_name = Arc::new(
            [
                (
                    "AAAAAAAAAAAAAAAAAAAACCCCCGGGGGTTTTTTTTTTTTTTTTTTTT".into(),
                    //format!("{}CCCCCGGGGG{}", "A".repeat(20), "T".repeat(20)).into(),
                    "probe1".to_string(),
                ),
                (
                    //format!("{}GGGGGCCCCC{}", "T".repeat(20), "A".repeat(20)).into(),
                    "TTTTTTTTTTTTTTTTTTTTGGGGGCCCCCTTTTTTTTTTTTTTTTTTTT".into(),
                    "probe2".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let engine = CellRangerProbeAssigner::new(seq_to_name, None).unwrap();
        assert_eq!(
            engine
                .query("AAAAAAAAAAAAAAAAAAAACCCCCCCCCCTTTTTTTTTTTTTTTTTTTT".into())
                .unwrap(),
            None
        );

        //split mapped,
        //
        let seq_to_name = Arc::new(
            [
                (
                    "GGAATGTAGCTGGCTCCGGCTATGTCTTCTGGCTGTGGCTCTAACAGACT".into(),
                    //format!("{}CCCCCGGGGG{}", "A".repeat(20), "T".repeat(20)).into(),
                    "probe1".to_string(),
                ),
                (
                    //format!("{}GGGGGCCCCC{}", "T".repeat(20), "A".repeat(20)).into(),
                    "GGAAGGCAGCTTGCACTGGCAAGAATCCAGGGAGGTCTCGCAGGTAAACT".into(),
                    "probe2".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        );
        let engine = CellRangerProbeAssigner::new(seq_to_name, None).unwrap();
        assert_eq!(
            engine
                .query("GGAATGTAGCTGGCTCCGGCTATGTTCCAGGGAGGTCTCGCAGGTAAACT".into())
                .unwrap(),
            None
        );
    }
}
