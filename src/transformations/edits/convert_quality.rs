#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::config::PhredEncoding;
use crate::demultiplex::Demultiplexed;
use crate::transformations::Transformation;
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConvertQuality {
    pub from: PhredEncoding,
    to: PhredEncoding,
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
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.from == self.to {
            anyhow::bail!(
                "ConvertQuality 'from' and 'to' encodings are the same, no conversion needed. Aborting"
            );
        }
        //since this happens before expansion, we can't enforce that there's a ValidateQuality
        //before us. Todo: consider doing this after expansion, or adding a
        //validate_after_expansion trait member?
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        fn apply_to_qual(
            lower: u8,
            upper: u8,
            block: &mut crate::io::FastQBlocksCombined,
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
                                u8::try_from(v).unwrap() // we checked the range
                            }
                        })
                        .collect();
                    read.replace_qual(new_qual);
                }
            });
        }

        fn to_solexa(offset: u8, lower: u8, upper: u8, block: &mut crate::io::FastQBlocksCombined) {
            apply_to_qual(lower, upper, block, |x| {
                phred_to_solexa(i16::from(x) - i16::from(offset)) + 64
            });
        }
        fn from_solexa(
            offset: u8,
            lower: u8,
            upper: u8,
            block: &mut crate::io::FastQBlocksCombined,
        ) {
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
