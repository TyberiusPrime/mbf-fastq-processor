#![allow(clippy::unnecessary_wraps)] // eserde false positive

use crate::config::deser::tpd_adapt_bstring_uppercase;
use crate::transformations::prelude::*;

use super::extract_numeric_tags_plus_all;


/// Quantify base occurrence rate or count
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct BaseContent {
    pub out_label: String,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,

    pub relative: bool,

    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring_uppercase")]
    pub bases_to_count: BString,

    #[tpd(default)]
    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring_uppercase")]
    pub bases_to_ignore: BString,

    #[tpd(skip)]
    #[schemars(skip)]
    bases_to_count_lookup: Vec<bool>,
    #[tpd(skip)]
    #[schemars(skip)]
    bases_to_ignore_lookup: Vec<bool>,
}

impl PartialBaseContent {
    fn build_lookup(bases: &BString) -> Vec<bool> {
        let mut lookup = vec![false; 256];

        for ch in bases.as_slice() {
            let idx = *ch as usize;
            lookup[idx] = true;
        }
        lookup
    }
}

impl VerifyIn<PartialConfig> for PartialBaseContent {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.relative.or(true);
        self.segment.validate_segment(parent);

        self.bases_to_count.verify(|v| {
            if v.is_empty() {
                return Err(ValidationFailure::new(
                    "Must contain at least one letter (base)",
                    None,
                ));
            }

            Ok(())
        });

        if let Some(bases_to_count) = self.bases_to_count.as_ref() {
            self.bases_to_count_lookup = Some(Self::build_lookup(bases_to_count));
        }
        if let Some(bases_to_ignore) = self.bases_to_ignore.as_ref() {
            self.bases_to_ignore_lookup = Some(Self::build_lookup(bases_to_ignore));
        }
        Ok(())
    }
}

impl BaseContent {
    fn sequence_totals(
        sequence: &[u8],
        bases_to_count: &[bool],
        bases_to_ignore: &[bool],
    ) -> (usize, usize) {
        let mut considered = 0usize;
        let mut counted = 0usize;

        for &base in sequence {
            let idx = base.to_ascii_uppercase() as usize;
            if bases_to_ignore[idx] {
                continue;
            }
            considered += 1;
            if bases_to_count[idx] {
                counted += 1;
            }
        }

        (considered, counted)
    }

    fn percentage(counted: usize, considered: usize) -> f64 {
        if considered == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                (counted as f64 / considered as f64) * 100.0
            }
        }
    }

    pub(crate) fn for_gc_replacement(out_label: String, segment: SegmentIndexOrAll) -> Self {
        Self {
            out_label,
            segment,
            relative: true,
            bases_to_count: BString::from("GC"),
            bases_to_ignore: BString::from("N"),
            bases_to_count_lookup: Vec::new(),
            bases_to_ignore_lookup: Vec::new(),
        }
    }

    pub(crate) fn for_n_count(out_label: String, segment: SegmentIndexOrAll) -> Self {
        Self {
            out_label,
            segment,
            relative: false,
            bases_to_count: BString::from("N"),
            bases_to_ignore: BString::default(),
            bases_to_count_lookup: Vec::new(),
            bases_to_ignore_lookup: Vec::new(),
        }
    }
}

impl Step for BaseContent {
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.out_label.clone(), TagValueType::Numeric))
    }

    #[allow(clippy::cast_precision_loss)]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment = self.segment;
        let bases_to_count_single = self.bases_to_count_lookup.clone();
        let bases_to_ignore_single = self.bases_to_ignore_lookup.clone();
        let bases_to_count_all = self.bases_to_count_lookup.clone();
        let bases_to_ignore_all = self.bases_to_ignore_lookup.clone();
        let relative = self.relative;

        extract_numeric_tags_plus_all(
            segment,
            &self.out_label,
            move |read| {
                let sequence = read.seq();
                let (considered, counted) = Self::sequence_totals(
                    sequence,
                    &bases_to_count_single,
                    &bases_to_ignore_single,
                );
                if relative {
                    Self::percentage(counted, considered)
                } else {
                    counted as f64
                }
            },
            move |reads| {
                let mut total_considered = 0usize;
                let mut total_counted = 0usize;

                for read in reads {
                    let (considered, counted) = Self::sequence_totals(
                        read.seq(),
                        &bases_to_count_all,
                        &bases_to_ignore_all,
                    );
                    total_considered += considered;
                    total_counted += counted;
                }

                if relative {
                    Self::percentage(total_counted, total_considered)
                } else {
                    total_counted as f64
                }
            },
            &mut block,
        );

        Ok((block, true))
    }
}
