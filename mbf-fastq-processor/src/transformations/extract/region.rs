#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

/// Define a region by coordinates
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub start: isize,
    #[serde(alias = "length")]
    pub len: usize,

    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[serde(alias = "segment")]
    pub source: String,

    #[serde(default)]
    #[serde(skip)]
    pub resolved_source: Option<ResolvedSourceNoAll>,

    /// Is the region from the `Start` or the `End` of the source?
    pub anchor: super::super::RegionAnchor,

    pub out_label: String,
}

impl FromTomlTableNested for Region {
    fn from_toml_table(
        _table: &toml_edit::Table,
        mut helper: TableErrorHelper,
    ) -> TomlResult<Self> {
        let start = helper.get("start");
        let len = helper.get(&["len", "length"][..]);
        let anchor = helper.get("anchor");
        let out_label = helper.get_tag("out_label");
        let resolved_source = helper.get_source_no_all(&["source", "segment"][..], true);
        helper.deny_unknown()?;

        let (source, resolved_source) = resolved_source?;

        Ok(Region {
            start: start?,
            len: len?,
            source: source,
            resolved_source: Some(resolved_source), //todo: remove Option
            anchor: anchor?,
            out_label: out_label?,
        })
    }
}

impl Step for Region {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> anyhow::Result<()> {
        self.resolved_source = Some(ResolvedSourceNoAll::parse(&self.source, input_def)?);
        Ok(())
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}
