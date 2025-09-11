#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{Demultiplexed, dna::HitRegion};

use super::super::{NewLocation, Step, filter_tag_locations_all_targets};

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///Does work with `ExtractRegions` and multiple regions.
///
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInSequence {
    label: String,
    #[serde(default)]
    ignore_missing: bool,
}

impl Step for StoreTagInSequence {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        #[derive(Eq, PartialEq, Debug)]
        enum WhatHappend {
            SameSize,
            Smaller,
            Larger,
        }

        let mut what_happend = Vec::new();

        block.apply_mut_with_tag(&self.label, |reads, tag_val| {
            if let Some(hit) = tag_val.as_sequence() {
                let mut what_happend_here = Vec::new();
                for region in &hit.0 {
                    let location = region
                        .location
                        .as_ref();
                    match location {
                        None => {
                            if self.ignore_missing {
                                //if we ignore missing locations, we just skip this region
                            } else {
                                panic!("StoreTagInSequence only works on regions with location data. Might have been lost on subsequent sequence editing transformations? Region: {region:?}. If you're ok with not storing those, set ignore_missing=true");
                            }
                        }

                        Some(location) => {

                        let read = &mut reads[location.segment_index.get_index()];
                        let seq = read.seq();
                        let mut new_seq: Vec<u8> = Vec::new();
                        new_seq.extend_from_slice(&seq[..location.start]);
                        new_seq.extend_from_slice(&region.sequence);
                        new_seq.extend_from_slice(&seq[location.start + location.len..]);

                        let mut new_qual: Vec<u8> = Vec::new();
                        new_qual.extend_from_slice(&read.qual()[..location.start]);
                        if region.sequence.len() == location.len {
                            //if the sequence is the same length as the location excised, we can just copy the quality
                            new_qual.extend_from_slice(
                                &read.qual()[location.start..location.start + location.len],
                            );
                            what_happend_here.push(WhatHappend::SameSize);
                        } else {
                            //otherwise, we need replace it with the average quality, repeated
                            let avg_qual = if location.is_empty() {
                                b'B'
                            } else {
                                let sum_qual = read.qual()
                                    [location.start..location.start + location.len]
                                    .iter()
                                    .map(|&x| u32::from(x))
                                    .sum::<u32>() ;
                                let avg_qual = f64::from(sum_qual) / location.len as f64;
                                avg_qual.round() as u8
                            };
                            new_qual.extend_from_slice(&vec![avg_qual; region.sequence.len()]);
                                if region.sequence.len() < location.len {
                                    what_happend_here.push(WhatHappend::Smaller);
                                } else {
                                    what_happend_here.push(WhatHappend::Larger);
                                }
                        }
                        new_qual.extend_from_slice(&read.qual()[location.start + location.len..]);

                        read.replace_seq(new_seq, new_qual);
                        }
                    }
                }
                what_happend.push(Some(what_happend_here));
            } else {
                what_happend.push(None);
            }
        });

        filter_tag_locations_all_targets(
            &mut block,
            |_location: &HitRegion, pos: usize| -> NewLocation {
                let what_happend_here = &what_happend[pos];
                match what_happend_here {
                    None => NewLocation::Keep,
                    Some(what_happend_here) => {
                        if what_happend_here
                            .iter()
                            .all(|x| *x == WhatHappend::SameSize)
                        {
                            NewLocation::Keep
                        } else {
                            //now the fun part. TODO
                            //Also todo: test cases
                            //for now, I'll just filter them
                            NewLocation::Remove
                        }
                    }
                }
            },
        );

        (block, true)
    }
}
