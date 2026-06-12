use std::cell::RefCell;
use std::rc::Rc;

use super::RegexExtraction;
use super::extract_region_or_value_tags_using_tags;
use super::extract_string_tags_using_tags;
use crate::transformations::prelude::*;
use bstr::ByteSlice;
use fastqrab_config::{tpd_adapt_bstring, tpd_adapt_regex};

fn regex_replace_with_self() -> BString {
    BString::from("$0")
}

/// Whether the replacement is exactly `$0` / `${0}` — the whole match. This is
/// the one replacement whose value span equals the write-back anchor (the whole
/// match), so the tag can stay a live alias window. Every other replacement
/// (a sub-group like `$1`, a reordering, literals, `[[tag]]`) produces content
/// whose value differs from, or is anchored differently than, a single read
/// slice, so it becomes an owned row anchored at the whole match (group 0).
fn replacement_is_whole_match(replacement: &BString) -> bool {
    matches!(replacement.as_slice(), b"$0" | b"${0}")
}

/// Quality for owned (divergent) regex content: the average quality of the
/// matched span `[start, end)` — or `b'B'` when that span is empty — repeated to
/// `out_len`. Mirrors the historic write-back behavior, so a grown or replaced
/// region reads back with a sensible quality that the tag now carries itself.
fn synth_quality(read_qual: &[u8], start: usize, end: usize, out_len: usize) -> Vec<u8> {
    let avg = if end <= start {
        b'B'
    } else {
        let span = &read_qual[start..end];
        let sum: u32 = span.iter().map(|&q| u32::from(q)).sum();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "average quality rounds into a single phred byte"
        )]
        let avg = (f64::from(sum) / span.len() as f64).round() as u8;
        avg
    };
    vec![avg; out_len]
}

/// Region by regular expression
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Regex {
    #[tpd(with = "tpd_adapt_regex")]
    #[tpd(alias = "pattern")]
    #[tpd(alias = "query")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,

    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub replacement: BString,
    out_label: TagLabel,

    #[tpd(adapt_in_verify(String), alias = "segment")]
    #[schemars(with = "String")]
    source: ResolvedSourceNoAll,
}

impl VerifyIn<PartialConfig> for PartialRegex {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        self.replacement.or_with(regex_replace_with_self);
        self.replacement.verify(|replacement| {
            let group_hunting_regexp =
                regex::bytes::Regex::new("[$]\\d+_").expect("hardcoded regex must compile");
            if group_hunting_regexp.is_match(replacement) {
                Err(ValidationFailure::new(
                    "Replacement string contains a '$1_' style group reference",
                    Some(
                        "This is a footgun: '$1_' is interpreted as group name '1_', not '$1' followed by '_'. Use '${1}_' instead.",
                    ),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialRegex> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = Vec::new();
            if let Some(replacement) = inner.replacement.as_ref() {
                let replacement = replacement.clone();
                let re = regex::bytes::Regex::new(r"\[\[(?P<tag>[^\]]+)\]\]")
                    .expect("hardcoded regex must compile");
                let toml_source = Rc::new(RefCell::new((
                    &mut inner.replacement.state,
                    &mut inner.replacement.help,
                )));
                for hit in re.captures_iter(replacement.as_bytes()) {
                    let tag = hit
                        .name("tag")
                        .expect("Regex should always match")
                        .as_bytes();
                    let tag = TagLabel::Normal(
                        std::str::from_utf8(tag)
                            .expect("Tag was not utf8, but toml is always utf8?)")
                            .to_string(),
                    );
                    // we already chek these for 'being present', just like any other
                    // tag.
                    used_tags.push(Some(UsedTag {
                        name: tag,
                        accepted_tag_types: &[
                            TagValueType::String,
                            TagValueType::Location,
                            TagValueType::Numeric((None, None)),
                            TagValueType::Bool,
                        ],
                        toml_source: toml_source.clone(),
                        further_help: None,
                    }));
                }
            }
            let declared_tag = inner.out_label.to_declared_tag({
                if let Some(MustAdapt::PostVerify(source)) = inner.source.as_ref() {
                    match source {
                        ResolvedSourceNoAll::Segment(_segment_index) => TagValueType::Location,
                        ResolvedSourceNoAll::Tag(_tag_label) => TagValueType::String,
                        ResolvedSourceNoAll::Name { .. } => TagValueType::String,
                    }
                } else {
                    TagValueType::Location
                }
            });
            if let Some(MustAdapt::PostVerify(ResolvedSourceNoAll::Tag(tag_name))) =
                inner.source.as_ref()
            {
                used_tags.push(Some(UsedTag {
                    name: tag_name.clone(),
                    accepted_tag_types: &[TagValueType::String, TagValueType::Location],
                    toml_source: Rc::new(RefCell::new((
                        &mut inner.source.state,
                        &mut inner.source.help,
                    ))),
                    further_help: None,
                }));
            }

            Some(TagUsageInfo {
                declared_tag,
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Regex {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn apply_regexp(
            search: &regex::bytes::Regex,
            replacement: &BString,
            haystack: &[u8],
            read_no: usize,
            block_tags: &IndexMap<TagLabel, TagColumn>,
        ) -> Option<Vec<u8>> {
            let re_hit = search.captures(haystack);
            if let Some(hit) = re_hit {
                let mut out = Vec::new();
                //let g = hit.get(0).expect("Regex should always match");
                hit.expand(&replacement, &mut out);
                for (tag_name, tags) in block_tags {
                    // only those we listed in use_tags.
                    let query = format!("[[{tag_name}]]");
                    let value = tags.to_bstr(read_no, |float| float.to_string(), None);
                    out = out.replace(query, value.as_bytes());
                }
                Some(out)
            } else {
                None
            }
        }
        let source = &self.source;

        match source {
            ResolvedSourceNoAll::Tag(tag_name) => {
                extract_string_tags_using_tags(
                    &mut block,
                    SegmentIndex::first(),
                    &self.out_label,
                    |_read, read_no, block_tags| {
                        // Choose source based on whether it's name or sequence
                        let haystack = block_tags
                            .get(tag_name)
                            .expect("Tag not present?!")
                            .to_bstr(read_no, |float| float.to_string(), None);
                        apply_regexp(
                            &self.search,
                            &self.replacement,
                            &haystack,
                            read_no,
                            block_tags,
                        )
                        .map(|x| x.into())
                    },
                );
            }
            ResolvedSourceNoAll::Name { segment_index, .. } => {
                let block_tags = &block.tags;
                let read_no = block.first_read_sequential_number;
                let mut out = Vec::with_capacity(block.row_count());
                for (ii, read_name) in block
                    .member(segment_index.as_index())
                    .names
                    .iter()
                    .enumerate()
                {
                    // Choose source based on whether it's name or sequence
                    let haystack = read_name;
                    out.push(
                        apply_regexp(
                            &self.search,
                            &self.replacement,
                            haystack,
                            read_no + ii,
                            block_tags,
                        )
                        .map(|x| x.into()),
                    );
                }
                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::String(out));
            }
            ResolvedSourceNoAll::Segment(segment_index) => {
                extract_region_or_value_tags_using_tags(
                    &mut block,
                    *segment_index,
                    &self.out_label,
                    |read_seq, read_qual, read_no, block_tags| {
                        let Some(hit) = self.search.captures(read_seq) else {
                            return RegexExtraction::None;
                        };
                        let g0 = hit.get(0).expect("group 0 always present on a match");
                        let anchor = g0.start() as u32..g0.end() as u32;

                        // Fast path: `$0` (the whole match) is the one replacement
                        // whose value span equals the write-back anchor, so keep it
                        // a live alias window (coordinates follow later edits, real
                        // quality preserved). This is the default "locate a pattern"
                        // case.
                        if replacement_is_whole_match(&self.replacement) {
                            return RegexExtraction::Region(anchor);
                        }

                        // General path: a sub-group (`$1`), reordering, literals, or
                        // [[tag]] interpolation. The value isn't the whole match, so
                        // own the expanded bytes anchored at the whole match (group 0
                        // — the span a write-back replaces), carrying quality
                        // synthesized from that span (its average, or b'B' if empty).
                        let mut seq = Vec::new();
                        hit.expand(&self.replacement, &mut seq);
                        for (tag_name, tags) in block_tags {
                            // only those we listed in use_tags.
                            let query = format!("[[{tag_name}]]");
                            let value = tags.to_bstr(read_no, |float| float.to_string(), None);
                            seq = seq.replace(query, value.as_bytes());
                        }
                        let qual = synth_quality(read_qual, g0.start(), g0.end(), seq.len());
                        RegexExtraction::Owned { anchor, seq, qual }
                    },
                );
            }
        }
        Ok((block, true))
    }
}
