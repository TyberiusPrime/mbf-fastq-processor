use schemars::JsonSchema;
use std::{cell::RefCell, rc::Rc};
use toml_pretty_deser::prelude::*;

use crate::{TagLabel, TagValueType, ToUsedTags, UsedTag};
pub use fastqrab_dna::segments::SegmentIndex;

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema, Hash)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(SegmentIndex),
}

impl TryInto<SegmentIndex> for SegmentIndexOrAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<SegmentIndex, Self::Error> {
        match self {
            SegmentIndexOrAll::Indexed(idx) => Ok(idx),
            SegmentIndexOrAll::All => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentOrNameIndex {
    Sequence(SegmentIndex),
    Name(SegmentIndex),
}

impl SegmentOrNameIndex {
    // #[must_use]
    // pub fn get_segment_index(&self) -> SegmentIndex {
    //     match self {
    //         SegmentOrNameIndex::Sequence(idx) | SegmentOrNameIndex::Name(idx) => *idx,
    //     }
    // }
    //
    // #[must_use]
    // pub fn is_name(&self) -> bool {
    //     matches!(self, SegmentOrNameIndex::Name(_))
    // }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceNoAll {
    Segment(SegmentIndex),
    Tag(TagLabel),
    Name {
        segment_index: SegmentIndex,
        split_character: u8,
    },
}

impl ResolvedSourceNoAll {
    //that's the ones we're going to use
    #[must_use]
    pub fn get_tags(&self) -> Option<Vec<(TagLabel, &'static [TagValueType])>> {
        match &self {
            ResolvedSourceNoAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[TagValueType::String, TagValueType::Location],
            )]),
            _ => None,
        }
    }
}

impl ToUsedTags for TomlValue<MustAdapt<String, ResolvedSourceNoAll>> {
    fn to_used_tags(&mut self) -> Vec<Option<UsedTag<'_>>> {
        if let Some(resolved) = self.as_ref().and_then(|x| x.as_ref_post()) {
            let mut res = Vec::new();
            if let Some(tags) = resolved.get_tags() {
                let toml_source = Rc::new(RefCell::new((&mut self.state, &mut self.help)));
                for (tag_name, accepted_tag_types) in tags {
                    res.push(Some(UsedTag {
                        name: tag_name,
                        accepted_tag_types,
                        toml_source: toml_source.clone(),
                        further_help: None,
                    }));
                }
            }
            res
        } else {
            vec![None]
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceAll {
    Segment(SegmentIndexOrAll),
    Tag(TagLabel),
    Name {
        segment_index_or_all: SegmentIndexOrAll,
        split_character: u8,
    },
}

impl ResolvedSourceAll {
    #[must_use]
    /// # Panics
    /// when segment index is out of bounds
    pub fn get_name(&self, segment_order: &[String]) -> String {
        match self {
            ResolvedSourceAll::Segment(SegmentIndexOrAll::Indexed(idx)) => {
                segment_order.get(idx.as_index()).cloned().unwrap_or_else(|| {
                    // cov:excl-start
                    panic!(
                        "Segment index {idx} out of bounds for segment order: [{segment_order:?}]"
                    )
                    // cov:excl-stop
                })
            }
            ResolvedSourceAll::Segment(SegmentIndexOrAll::All) => "all".to_string(),
            ResolvedSourceAll::Tag(name) => format!("tag:{name}"),
            ResolvedSourceAll::Name {
                segment_index_or_all,
                ..
            } => format!(
                "name:{}",
                match segment_index_or_all {
                    SegmentIndexOrAll::Indexed(idx) => {
                        segment_order.get(idx.as_index()).cloned().unwrap_or_else(|| {
                        // cov:excl-start
                        panic!("Segment index {idx} out of bounds for segment order: [{segment_order:?}]")
                        // cov:excl-stop
                    })
                    }
                    SegmentIndexOrAll::All => "all".to_string(),
                }
            ),
        }
    }

    //that's the ones we're going to use
    #[must_use]
    pub fn get_tags(&self) -> Option<Vec<(TagLabel, &'static [TagValueType])>> {
        match &self {
            ResolvedSourceAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[TagValueType::String, TagValueType::Location],
            )]),
            _ => None,
        }
    }
}

impl ToUsedTags for TomlValue<MustAdapt<String, ResolvedSourceAll>> {
    fn to_used_tags(&mut self) -> Vec<Option<UsedTag<'_>>> {
        if let Some(resolved) = self.as_ref().and_then(|x| x.as_ref_post()) {
            let mut res = Vec::new();
            if let Some(tags) = resolved.get_tags() {
                let toml_source = Rc::new(RefCell::new((&mut self.state, &mut self.help)));
                for (tag_name, accepted_tag_types) in tags {
                    res.push(Some(UsedTag {
                        name: tag_name,
                        accepted_tag_types,
                        toml_source: toml_source.clone(),
                        further_help: None,
                    }));
                }
            }
            res
        } else {
            vec![None]
        }
    }
}

// No-op alias-tree impls for the segment scalar types (no aliases of their own).
toml_pretty_deser::tpd_alias_leaf!(SegmentIndexOrAll, ResolvedSourceNoAll, ResolvedSourceAll);
