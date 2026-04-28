use crate::transformations::{RegionAnchor, prelude::*};

/// Define a region by coordinates
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Region {
    pub start: isize,
    #[tpd(alias = "length")]
    pub len: usize,

    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "segment")]
    pub source: ResolvedSourceNoAll,

    /// Is the region from the `Start` or the `End` of the source?
    pub anchor: RegionAnchor,

    pub out_label: TagLabel,
}

impl VerifyIn<PartialConfig> for PartialRegion {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        Ok(())
    }
}

// cov:excl-start
impl TagUser for PartialTaggedVariant<PartialRegion> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        Some(TagUsageInfo::default())
    }
}
// cov:excl-stop

impl Step for Region {
    // cov:excl-start
    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}
// cov:excl-stop
