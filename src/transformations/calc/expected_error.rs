use anyhow::Result;
use bstr::BString;
use std::cell::RefCell;

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    demultiplex::Demultiplexed,
    io,
};

use super::super::{
    Step, TagValueType,
    reports::common::{PHRED33OFFSET, Q_LOOKUP},
};

use super::extract_numeric_tags_plus_all;

const PHRED33_MAX: u8 = 126;

#[derive(eserde::Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ExpectedErrorAggregate {
    Sum,
    Max,
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExpectedError {
    pub label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
    pub aggregate: ExpectedErrorAggregate,
}

impl Step for ExpectedError {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.label.clone(), TagValueType::Numeric))
    }

    fn apply(
        &mut self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        let error_state: RefCell<Option<anyhow::Error>> = RefCell::new(None);

        let aggregate = self.aggregate;

        extract_numeric_tags_plus_all(
            self.segment_index.expect("segment_index validated"),
            &self.label,
            |read| {
                if error_state.borrow().is_some() {
                    return 0.0;
                }
                match expected_error_for_read(read, aggregate) {
                    Ok(value) => value,
                    Err(err) => {
                        *error_state.borrow_mut() = Some(err);
                        0.0
                    }
                }
            },
            |reads| {
                if error_state.borrow().is_some() {
                    return 0.0;
                }
                let mut aggregated_value = 0.0;
                for read in reads {
                    match expected_error_for_read(read, aggregate) {
                        Ok(value) => match aggregate {
                            ExpectedErrorAggregate::Sum => {
                                aggregated_value += value;
                            }
                            ExpectedErrorAggregate::Max => {
                                aggregated_value = aggregated_value.max(value);
                            }
                        },
                        Err(err) => {
                            *error_state.borrow_mut() = Some(err);
                            return 0.0;
                        }
                    }
                }
                aggregated_value
            },
            &mut block,
        );

        match error_state.into_inner() {
            Some(err) => Err(err),
            None => Ok((block, true)),
        }
    }
}

fn expected_error_for_read(
    read: &io::WrappedFastQRead,
    aggregate: ExpectedErrorAggregate,
) -> anyhow::Result<f64> {
    let mut agg = 0.0;

    for &quality in read.qual() {
        if !(PHRED33OFFSET..=PHRED33_MAX).contains(&quality) {
            let quality_display = BString::from(vec![quality]);
            let read_name = BString::from(read.name().to_vec());
            anyhow::bail!(
                "CalcExpectedError requires PHRED+33 encoded qualities (ASCII 33..=126). Observed byte {quality} ('{}') in read '{}'. Consider running ConvertQuality before CalcExpectedError.",
                quality_display.escape_ascii(),
                read_name.escape_ascii()
            );
        }
        let expected_error = Q_LOOKUP[quality as usize];
        match aggregate {
            ExpectedErrorAggregate::Sum => {
                agg += expected_error;
            }
            ExpectedErrorAggregate::Max => {
                agg = expected_error.max(agg);
            }
        }
    }

    Ok(agg)
}
