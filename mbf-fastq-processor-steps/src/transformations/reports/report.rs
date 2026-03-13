use mbf_fastq_processor_config::NonAmbigousDNA;

use crate::transformations::prelude::*;

/// Include a report at this position
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Report {
    pub name: String,
    pub count: bool,
    #[tpd(default)]
    pub base_statistics: bool,
    #[tpd(default)]
    pub length_distribution: bool,
    #[tpd(default)]
    pub duplicate_count_per_read: bool,
    #[tpd(default)]
    pub duplicate_count_per_fragment: bool,

    #[schemars(skip)]
    #[tpd(default)]
    pub debug_reproducibility: bool,

    #[schemars(with = "Option<Vec<String>>")]
    pub count_oligos: Option<Vec<NonAmbigousDNA>>,

    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    pub count_oligos_segment: SegmentIndexOrAll,

    /// Generate histograms for specified tags
    #[tpd(alias = "tag_histogram")]
    pub tag_histograms: Option<Vec<TagLabel>>,
}

impl VerifyIn<PartialConfig> for PartialReport {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.name.verify(|name: &String| {
            if name.is_empty() {
                Err(ValidationFailure::new("Name must not be empty", None))
            } else {
                Ok(())
            }
        });
        self.count.or(true);
        self.count_oligos_segment.or(SegmentIndexOrAll::All);
        self.count_oligos_segment.validate_segment(parent);

        Ok(())
    }
}

impl Default for Report {
    fn default() -> Self {
        panic!();
        Self {
            name: "report".to_string(),
            count: true,
            base_statistics: false,
            length_distribution: false,
            duplicate_count_per_read: false,
            duplicate_count_per_fragment: false,
            debug_reproducibility: false,
            count_oligos: None,
            count_oligos_segment: SegmentIndexOrAll::All,
            tag_histograms: None,
        }
    }
}

impl TagUser for PartialTaggedVariant<PartialReport> {
    #[mutants::skip]
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        // cov:excl-start
        unreachable!(
            "Report should not be used as a tagged variant - should be expanded into individual parts before"
        );
        // cov:excl-stop
    }

    //report name dupliaction is being done in config verify_reports
}

impl Step for Report {
    #[mutants::skip]
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // cov:excl-start
        unreachable!("Should not be reached - should be expanded into individual parts before");
        // cov:excl-stop
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // cov:excl-start
        panic!("Should not be reached - should be expanded into individual parts before");
        // cov:excl-stop
    }
}
