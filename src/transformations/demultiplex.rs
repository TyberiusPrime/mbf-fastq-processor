use std::collections::BTreeMap;

use anyhow::Result;

use super::{extract_regions, RegionDefinition};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use serde_valid::Validate;
use crate::config::deser::btreemap_dna_string_from_string;



#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformDemultiplex {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,
    pub max_hamming_distance: u8,
    pub output_unmatched: bool,
    #[serde(deserialize_with = "btreemap_dna_string_from_string")]
    pub barcodes: BTreeMap<Vec<u8>, String>,
}

impl ConfigTransformDemultiplex {
    pub fn init(&mut self) -> Result<DemultiplexInfo> {
        DemultiplexInfo::new(&self.barcodes, self.output_unmatched)
    }
}

pub fn transform_demultiplex(
    config: &mut ConfigTransformDemultiplex,
    mut block: crate::io::FastQBlocksCombined,
    demultiplex_info: &Demultiplexed,
) -> (crate::io::FastQBlocksCombined, bool) {
    let mut tags: Vec<u16> = vec![0; block.len()];
    let demultiplex_info = demultiplex_info.unwrap();
    for ii in 0..block.read1.len() {
        let key = extract_regions(ii, &block, &config.regions, b"_");
        let entry = demultiplex_info.barcode_to_tag(&key);
        match entry {
            Some(tag) => {
                tags[ii] = tag;
            }
            None => {
                if config.max_hamming_distance > 0 {
                    for (barcode, tag) in demultiplex_info.iter_barcodes() {
                        let distance = bio::alignment::distance::hamming(&key, barcode);
                        if distance.try_into().unwrap_or(255u8) <= config.max_hamming_distance {
                            tags[ii] = tag;
                            break;
                        }
                    }
                }
                //tag[ii] = 0 -> not found
                //todo: hamming distance trial
            }
        }
    }
    block.output_tags = Some(tags);
    (block, true)
}
