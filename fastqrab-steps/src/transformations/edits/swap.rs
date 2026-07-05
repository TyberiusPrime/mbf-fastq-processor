use crate::transformations::prelude::*;
use bstr::ByteSlice;
use fastqrab_io::blocks::FastQChunk;

/// Swap two segments.
///
/// With `if_tag` set, the swap is per-read (only reads where the tag is truthy
/// exchange their mates). A per-read swap moves a read to the *other* segment,
/// so any location tag captured against either swapped segment is **forgotten**
/// — a location column may alias only one segment, and a per-read swap would
/// split it across two. To carry such data across a conditional swap, snapshot
/// it to a plain String with `ConcatTags` beforehand. An unconditional swap
/// keeps location tags: their `source_id` is simply re-stamped to the segment
/// that now holds their bytes.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Swap {
    #[tpd(default)]
    #[tpd(alias = "if_label")]
    if_tag: Option<ConditionalTagLabel>,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment_a: SegmentIndex,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment_b: SegmentIndex,
}

impl VerifyIn<PartialConfig> for PartialSwap {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        let input_def = parent
            .input
            .as_ref()
            .expect("Input definition must be set before config for Swap step validation");
        let segment_order = input_def.get_segment_order();

        match (self.segment_a.is_missing(), self.segment_b.is_missing()) {
            (true, false) | (false, true) => {
                self.segment_a.state = TomlValueState::Nested;
                self.segment_b.state = TomlValueState::Nested;
                return Err(ValidationFailure::new(
                    "Insuffient swap definition",
                    Some(
                        "Please either specify both segment_a and segment_b, or omit both for auto-detection.",
                    ),
                ));
            }
            (true, true) => {
                if segment_order.len() == 2 {
                    self.segment_a =
                        TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(0)), 0..0);
                    self.segment_b =
                        TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(1)), 0..0);
                } else {
                    self.segment_a.state = TomlValueState::Nested;
                    self.segment_b.state = TomlValueState::Nested;
                    return Err(ValidationFailure::new(
                        "Insuffient swap definition",
                        Some(
                            "There were more (or fewer) than 2 segments, and you did not specify both segment_a and segment_b.",
                        ),
                    ));
                }
            }
            (false, false) => {
                if self.segment_a.is_needs_further_validation()
                //|| self.segment_b.is_needs_further_validation()
                {
                    self.segment_a.validate_segment(parent);
                    self.segment_b.validate_segment(parent);
                    if self.segment_a.is_ok()
                        && self.segment_b.is_ok()
                        && self
                            .segment_a
                            .as_ref()
                            .expect("just checked is._ok")
                            .as_ref_post()
                            == self
                                .segment_b
                                .as_ref()
                                .expect("just checked is._ok")
                                .as_ref_post()
                    {
                        let spans = vec![
                            (self.segment_a.span(), "Identical to segment_b".to_string()),
                            (self.segment_b.span(), "Identical to segment_a".to_string()),
                        ];
                        self.segment_a.state = TomlValueState::Custom { spans };
                        self.segment_a.help =
                            Some("Please specify two different segments to swap.".to_string());
                        self.segment_b.state = TomlValueState::Nested;
                    }
                } // cov:excl-line
            }
        }
        //all other errors we pass straight on
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialSwap> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            // A conditional (per-read) swap moves individual reads between the two
            // segments, which would split any location tag on those segments
            // across two segments — a representation the model forbids. So forget
            // those location tags here, at config-verify, by removing them from
            // the available set; a later reference then fails cleanly (instead of
            // panicking at runtime when the tag is materialized).
            let removed_tags = {
                let is_conditional = inner.if_tag.as_ref().and_then(|o| o.as_ref()).is_some();
                let seg_a = inner
                    .segment_a
                    .as_ref()
                    .and_then(|m| m.as_ref_post())
                    .map(SegmentIndex::as_index);
                let seg_b = inner
                    .segment_b
                    .as_ref()
                    .and_then(|m| m.as_ref_post())
                    .map(SegmentIndex::as_index);
                if let (true, Some(a), Some(b)) = (is_conditional, seg_a, seg_b) {
                    let removed: Vec<TagLabel> = tags_available
                        .iter()
                        .filter_map(|(label, meta)| {
                            meta.segment
                                .filter(|seg| seg.as_index() == a || seg.as_index() == b)
                                .map(|_| label.clone())
                        })
                        .collect();
                    if removed.is_empty() {
                        RemovedTags::None
                    } else {
                        RemovedTags::SomeOwned(removed)
                    }
                } else {
                    RemovedTags::None
                }
            };
            Some(TagUsageInfo {
                used_tags: vec![inner.if_tag.to_used_tag(&[])],
                removed_tags,
                must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Swap {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let (index_a, index_b) = {
            let a = self.segment_a.get_index();
            let b = self.segment_b.get_index();
            match a.cmp(&b) {
                std::cmp::Ordering::Less => (a, b),
                std::cmp::Ordering::Equal => {
                    panic!("Swap same segment. should be prevente by config?!") // cov:excl-line
                }
                std::cmp::Ordering::Greater => (b, a),
            }
        };

        // If no condition, do unconditional swap. The two segments move as
        // whole units, so any location tag captured against one of them is still
        // valid — it just lives at the other index now. Re-stamp its source_id so
        // liftover and write-back keep targeting the segment that holds its bytes.
        if self.if_tag.is_none() {
            block.swap_members(index_a as usize, index_b as usize);

            for col in block.tags.values_mut() {
                if let TagColumn::Location(loc) = col {
                    let sid = loc.source_id();
                    if sid == u32::from(index_a) {
                        loc.set_source_id(u32::from(index_b));
                    } else if sid == u32::from(index_b) {
                        loc.set_source_id(u32::from(index_a));
                    }
                } // cov:excl-line not swapping non string tags is fine, no need for a test
            }

            return Ok((block, true));
        }

        // Conditional swap: only the reads where `cond_tag` is true exchange
        // their segment_a/segment_b mates. A single read now moves to the *other*
        // segment, so a location tag captured against one of these segments would
        // need to track different segments for different rows — a cross-segment
        // column, which the design forbids (a Value aliases exactly one read pod).
        //
        // So we rebuild both segments as fresh pods (selecting each output row
        // from whichever source it should now carry) and *forget* every location
        // tag bound to either swapped segment. A user who wants to keep that data
        // across a conditional swap can snapshot it to a plain String first with
        // `ConcatTags` (which detaches it from the live segment).
        let cond_tag = self
            .if_tag
            .as_ref()
            .expect("if_tag must be set when conditional swap is used");
        let swap_these = get_bool_vec_from_tag(&block, cond_tag);

        let (a, b) = (index_a as usize, index_b as usize);
        let n = block.segments[a].len();

        let mut a_names = StringPodBuilder::with_capacity(0, n);
        let mut a_seq_quals = DualStringPodBuilder::with_capacity(0, n);
        let mut a_pluses = StringPodBuilder::with_capacity(0, n);
        let mut b_names = StringPodBuilder::with_capacity(0, n);
        let mut b_seq_quals = DualStringPodBuilder::with_capacity(0, n);
        let mut b_pluses = StringPodBuilder::with_capacity(0, n);

        {
            let seg_a = &block.segments[a];
            let seg_b = &block.segments[b];
            for (row, &swap) in swap_these.iter().enumerate() {
                // For a swapped row, new-a takes from old-b and new-b from old-a.
                let (src_a, src_b) = if swap { (seg_b, seg_a) } else { (seg_a, seg_b) };

                a_names.push(src_a.names.get(row).as_bytes());
                let (seq, qual) = src_a.seq_quals.pair(row);
                a_seq_quals.push(seq.as_bytes(), qual.as_bytes());
                a_pluses.push(src_a.pluses.get(row).as_bytes());

                b_names.push(src_b.names.get(row).as_bytes());
                let (seq, qual) = src_b.seq_quals.pair(row);
                b_seq_quals.push(seq.as_bytes(), qual.as_bytes());
                b_pluses.push(src_b.pluses.get(row).as_bytes());
            }
        }

        block.segments[a] = FastQChunk {
            names: a_names.finish(),
            seq_quals: a_seq_quals.finish(),
            pluses: a_pluses.finish(),
        };
        block.segments[b] = FastQChunk {
            names: b_names.finish(),
            seq_quals: b_seq_quals.finish(),
            pluses: b_pluses.finish(),
        };

        // The location tags anchored to either rebuilt segment are forgotten by
        // the pipeline: this stage declares them via `removed_tags`, and the
        // workpool drops them before `apply` runs — so there is nothing to do
        // here.
        Ok((block, true))
    }
}
