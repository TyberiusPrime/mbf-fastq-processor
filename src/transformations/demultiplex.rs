use std::collections::BTreeMap;

use anyhow::Result;

use super::{extract_regions, RegionDefinition};
use crate::config::deser::btreemap_dna_string_from_string;
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use serde_valid::Validate;

#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformDemultiplex {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,
    pub max_hamming_distance: u8,
    pub output_unmatched: bool,
    // a mapping barcode -> output infix
    #[serde(deserialize_with = "btreemap_dna_string_from_string")]
    pub barcodes: BTreeMap<Vec<u8>, String>,
    #[serde(skip)]
    pub had_iupac: bool,
}

impl ConfigTransformDemultiplex {
    pub fn init(&mut self) -> Result<DemultiplexInfo> {
        self.had_iupac = self
            .barcodes
            .keys()
            .map(|x| crate::dna::contains_iupac_ambigous(x))
            .any(|x| x);
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
    for (ii, target_tag) in tags.iter_mut().enumerate() {
        let key = extract_regions(ii, &block, &config.regions, b"_");
        let entry = demultiplex_info.barcode_to_tag(&key);
        match entry {
            Some(tag) => {
                *target_tag = tag;
            }
            None => {
                if config.had_iupac {
                    for (barcode, tag) in demultiplex_info.iter_barcodes() {
                        let distance = crate::dna::iupac_hamming_distance(barcode, &key);
                        if distance.try_into().unwrap_or(255u8) <= config.max_hamming_distance {
                            *target_tag = tag;
                            break;
                        }
                    }
                }
                if config.max_hamming_distance > 0 {
                    for (barcode, tag) in demultiplex_info.iter_barcodes() {
                        //barcodes typically are below teh distance where we would consider
                        //SIMD to be helpful. Could benchmark though
                        let distance = bio::alignment::distance::hamming(barcode, &key);
                        if distance.try_into().unwrap_or(255u8) <= config.max_hamming_distance {
                            *target_tag = tag;
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
