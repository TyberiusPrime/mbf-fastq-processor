#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::config::PhredEncoding;

/// Convert PHRED scores between encodings
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ConvertQuality {
    pub from: PhredEncoding,
    to: PhredEncoding,
}
impl VerifyIn<PartialConfig> for PartialConvertQuality {
    fn verify(&mut self, _parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(from) = self.from.as_ref()
            && let Some(to) = self.to.as_ref()
            && from == to
        {
            let spans = vec![
                (self.to.span.clone(), "Identical to from".to_string()),
                (self.from.span.clone(), "Identical to to".to_string()),
            ];
            self.to.state = TomlValueState::Custom { spans };
            self.to.help = Some(
                "Conversion unnecessary? Please specify different encodings or remove this step."
                    .to_string(),
            );
        }
        Ok(())
    }
}

#[allow(clippy::cast_possible_truncation)]
fn phred_to_solexa(q_phred: i16) -> i16 {
    let val = 10f64.powf(f64::from(q_phred) / 10.0) - 1.0;
    (10.0 * val.log10()).round() as i16
}

#[allow(clippy::cast_possible_truncation)]
fn solexa_to_phred(q_solexa: i16) -> i16 {
    (10.0 * ((10f64.powf(f64::from(q_solexa) / 10.0) + 1.0).log10())).round() as i16
}

impl Step for ConvertQuality {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn apply_to_qual(
            lower: u8,
            upper: u8,
            block: &mut FastQBlocksCombined,
            func: impl Fn(u8) -> i16,
        ) {
            block.apply_mut(|segments| {
                for read in segments {
                    let qual = read.qual();
                    let new_qual: Vec<_> = qual
                        .iter()
                        .map(|x| {
                            let v = func(*x);
                            if v < i16::from(lower) {
                                lower
                            } else if v > i16::from(upper) {
                                upper
                            } else {
                                u8::try_from(v).expect("value must be in u8 range after validation")
                            }
                        })
                        .collect();
                    read.replace_qual(&new_qual);
                }
            });
        }

        fn to_solexa(offset: u8, lower: u8, upper: u8, block: &mut FastQBlocksCombined) {
            apply_to_qual(lower, upper, block, |x| {
                phred_to_solexa(i16::from(x) - i16::from(offset)) + 64
            });
        }
        fn from_solexa(offset: u8, lower: u8, upper: u8, block: &mut FastQBlocksCombined) {
            apply_to_qual(lower, upper, block, |x| {
                solexa_to_phred(i16::from(x) - 64) + i16::from(offset)
            });
        }
        let (lower, upper) = self.to.limits();

        //we may assume they have been checked, for range, because Transformation::expand
        //has added a ValidatePhred step before this one.
        match (self.from, self.to) {
            (PhredEncoding::Sanger, PhredEncoding::Sanger)
            | (PhredEncoding::Illumina13, PhredEncoding::Illumina13)
            | (PhredEncoding::Solexa, PhredEncoding::Solexa) => unreachable!(),

            (PhredEncoding::Sanger, PhredEncoding::Illumina13) => {
                apply_to_qual(lower, upper, &mut block, |x: u8| i16::from(x) + (64 - 33));
            }
            (PhredEncoding::Illumina13, PhredEncoding::Sanger) => {
                apply_to_qual(lower, upper, &mut block, |x: u8| i16::from(x) + (33 - 64));
            }

            (PhredEncoding::Sanger, PhredEncoding::Solexa) => {
                to_solexa(33, lower, upper, &mut block);
            }
            (PhredEncoding::Illumina13, PhredEncoding::Solexa) => {
                to_solexa(64, lower, upper, &mut block);
            }
            (PhredEncoding::Solexa, PhredEncoding::Sanger) => {
                from_solexa(33, lower, upper, &mut block);
            }
            (PhredEncoding::Solexa, PhredEncoding::Illumina13) => {
                from_solexa(64, lower, upper, &mut block);
            }
        }

        Ok((block, true))
    }
}
