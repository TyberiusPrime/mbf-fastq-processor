use crate::transformations::prelude::*;

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///Does work with `ExtractRegions` and multiple regions.
///
#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct StoreTagBackInSequence {
    in_label: TagLabel,
    #[tpd(default)]
    ignore_missing: bool,
}

impl TagUser for PartialTaggedVariant<PartialStoreTagBackInSequence> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location])],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for StoreTagBackInSequence {
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Avg quality calculation won't be affected"
    )]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        //#[derive(Eq, PartialEq, Debug)]
        // enum WhatHappend {
        //     SameSize,
        //     Smaller,
        //     Larger,
        // }

        let mut what_happend = Vec::with_capacity(block.len());
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);

        block.apply_mut_with_location_tag(&self.in_label, |reads, hit, col| {
            if let Some(hit) = hit {
                let mut kept_size  = true;
                for &region in hit.iter() {
                    let location = col.hit_location(region);
                    match location {
                        None => {
                            if self.ignore_missing {
                                //if we ignore missing locations, we just skip this region
                            } else {
                                *error_encountered.borrow_mut() = Some(format!(
                                    "StoreTagBackInSequence only works on regions with location data.\n\nSuggestion: Set ignore_missing=true to skip regions without location data, or check if location data was lost in previous transformations"
                                ));
                                return;
                            }
                        }
                        Some(location) => {

                        let region_seq = col.hit_bytes(region).to_vec();
                        let read = &mut reads[location.segment_index.as_index()];
                        let seq = read.seq();
                        let mut new_seq: Vec<u8> = Vec::new();
                        new_seq.extend_from_slice(&seq[..location.start]);
                        new_seq.extend_from_slice(&region_seq);
                        new_seq.extend_from_slice(&seq[location.start + location.len..]);

                        let mut new_qual: Vec<u8> = Vec::new();
                        new_qual.extend_from_slice(&read.qual()[..location.start]);
                        if region_seq.len() == location.len {
                            //if the sequence is the same length as the location excised, we can just copy the quality
                            new_qual.extend_from_slice(
                                &read.qual()[location.start..location.start + location.len],
                            );
                            //size was kept
                        } else {
                            //otherwise, we need replace it with the average quality, repeated
                            let avg_qual = if location.len == 0 {
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
                            new_qual.extend_from_slice(&vec![avg_qual; region_seq.len()]);
                            kept_size = region_seq.len() <= location.len;
                        }
                        new_qual.extend_from_slice(&read.qual()[location.start + location.len..]);

                        read.replace_seq(&new_seq, &new_qual);
                        }
                    }
                }
                what_happend.push(kept_size);
            } else {
                what_happend.push(true);
            }
        });

        // Check if any error was encountered during processing
        if let Some(error_msg) = error_encountered.borrow().as_ref() {
            return Err(anyhow::anyhow!("{error_msg}"));
        }

        block.filter_tag_locations_all_targets(|_location: HitRegion, pos: usize| -> NewLocation {
            match &what_happend[pos] {
                true => NewLocation::Keep,
                false => {
                    //now the fun part. TODO
                    //Also todo: test cases
                    //for now, I'll just filter them
                    NewLocation::Remove
                }
            }
        });

        Ok((block, true))
    }
}
