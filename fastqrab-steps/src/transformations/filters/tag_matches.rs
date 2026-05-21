use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashSet;

use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct TagMatches {
    #[tpd(adapt_in_verify(String), alias = "segment")]
    #[schemars(with = "String")]
    source: ResolvedSourceNoAll,

    out_label: TagLabel,
    #[allow(dead_code)] //translated in verify
    accepted: Vec<String>,

    #[tpd(skip)]
    #[schemars(skip)]
    accepted_set: FxHashSet<BString>,
}

impl VerifyIn<PartialConfig> for PartialTagMatches {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        if let Some(accepted) = self.accepted.as_ref() {
            self.accepted_set = Some(
                accepted
                    .iter()
                    .map(|x| x.as_ref().expect("parent was ok"))
                    .map(|s| BString::new(s.as_bytes().to_vec()))
                    .collect(),
            );
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialTagMatches> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        let inner = self
            .toml_value
            .value
            .as_mut()
            .expect("get_tag_usage called on failed verify");
        let mut used_tags = Vec::new();
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
            declared_tag: inner.out_label.to_declared_tag(TagValueType::Bool),
            used_tags,
            must_see_all_tags: true, // for filtering them down
            ..Default::default()
        })
    }
}

impl Step for TagMatches {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let accepted_set = &self.accepted_set;
        let found: Vec<bool> = match &self.source {
            ResolvedSourceNoAll::Segment(segment_index) => {
                block.segments[segment_index.0].apply(|read| accepted_set.contains(read.seq()))
            }
            ResolvedSourceNoAll::Name { segment_index, .. } => {
                block.segments[segment_index.0].apply(|read| accepted_set.contains(read.name()))
            }
            ResolvedSourceNoAll::Tag(tag_label) => {
                let tag_values = block
                    .tags
                    .get(tag_label)
                    .expect("Tag not set? Should have been caught earlier in validation.");
                tag_values
                    .iter_stringified()
                    .map(|opt_tag_value| {
                        opt_tag_value
                            .map(|tag_value| accepted_set.contains(tag_value.as_ref()))
                            .unwrap_or(false)
                    })
                    .collect()
            }
        };
        block
            .tags
            .insert(self.out_label.clone(), TagColumn::Bool(found));

        Ok((block, true))
    }
}
