use std::collections::{BTreeMap, HashMap};

use anyhow::{Context, Result};
use bstr::BString;

/// what the other steps need to know about the demultiplexing
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DemultiplexInfo {
    names: Vec<String>,                    //these include all outputs
    barcode_to_tag: HashMap<BString, u16>, //tag is never 0 in this
    include_no_barcode: bool,              //only relevant for output
}

impl DemultiplexInfo {
    pub fn new(
        barcode_to_name: &BTreeMap<BString, String>,
        include_no_barcode: bool,
    ) -> Result<Self> {
        let mut names = Vec::new();
        let mut barcode_to_tag = HashMap::new();
        if include_no_barcode {
            names.push("no-barcode".to_string());
        }
        for (tag, (barcode, name)) in barcode_to_name.iter().enumerate() {
            // no longer true. We combine outputs from multiple barcodes if
            // the user wishes
            /* if names.contains(name) {
                bail!(
                    "Barcode output infixes must be distinct. Duplicated: '{}'",
                    name
                )
            } */
            names.push(name.clone());
            let tag = tag + 1;
            barcode_to_tag.insert(
                barcode.clone(),
                tag.try_into().context("too many barcodes")?,
            );
        }
        Ok(Self {
            names,
            barcode_to_tag,
            include_no_barcode,
        })
    }

    #[must_use]
    pub fn barcode_to_tag(&self, barcode: &[u8]) -> Option<u16> {
        if let Some(tag) = self.barcode_to_tag.get(barcode) {
            return Some(*tag);
        } else if !barcode.is_empty() {
            for (bc, tag) in &self.barcode_to_tag {
                if bc.len() == barcode.len() && crate::dna::iupac_hamming_distance(bc, barcode) == 0
                {
                    return Some(*tag);
                }
            }
        }
        None
    }

    /// Iterate (barcode, tag) tuples
    /// this never includes the no-barcode output
    pub fn iter_barcodes(&self) -> impl Iterator<Item = (&BString, u16)> {
        //self.barcode_to_tag.iter()
        self.barcode_to_tag
            .iter()
            .map(|(barcode, tag)| (barcode, *tag))
    }

    /// Iterate `(tag, output_name)` tuples.
    /// this includes the no-barcode output if it exists
    #[allow(clippy::cast_possible_truncation)]
    pub fn iter_outputs(&self) -> impl Iterator<Item = (u16, &str)> {
        self.names.iter().enumerate().map(|(tag, name)| {
            (
                (tag + usize::from(!self.include_no_barcode)) as u16,
                name.as_str(),
            )
        })
    }

    #[must_use]
    pub fn len_outputs(&self) -> usize {
        self.names.len()
    }

    /// Combine this DemultiplexInfo with another to support multiple demultiplex steps
    /// Returns a new DemultiplexInfo where outputs are the Cartesian product of both
    /// and infixes are chained with underscore
    pub fn combine_with(&self, other: &DemultiplexInfo) -> Result<Self> {
        let mut combined_names = Vec::new();
        let combined_barcode_to_tag = HashMap::new();

        // Generate all combinations of output names by chaining infixes
        for (_tag1, name1) in self.iter_outputs() {
            for (_tag2, name2) in other.iter_outputs() {
                let combined_name = if name1 == "no-barcode" {
                    name2.to_string()
                } else if name2 == "no-barcode" {
                    name1.to_string()
                } else {
                    format!("{}_{}", name1, name2)
                };
                combined_names.push(combined_name);
            }
        }

        // For combined demultiplex, we don't use barcode_to_tag lookup
        // Instead, tags are computed by combining the individual demultiplex tags
        Ok(Self {
            names: combined_names,
            barcode_to_tag: combined_barcode_to_tag,
            include_no_barcode: false, // Combined demultiplex doesn't have unmatched
        })
    }

    /// Get the number of outputs (excluding no-barcode if present)
    #[must_use]
    pub fn output_count(&self) -> usize {
        if self.include_no_barcode {
            self.names.len() - 1
        } else {
            self.names.len()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Demultiplexed {
    No,
    Yes(DemultiplexInfo),
}

impl Demultiplexed {
    #[allow(clippy::cast_possible_truncation)]
    pub fn iter_tags(&self) -> impl Iterator<Item = u16> {
        match self {
            Self::No => 0..1,
            Self::Yes(info) => {
                if info.include_no_barcode {
                    0..info.names.len() as u16
                } else {
                    1..info.names.len() as u16
                }
            }
        }
    }

    #[must_use]
    pub fn max_tag(&self) -> u16 {
        match self {
            Self::No => 0,
            Self::Yes(info) => {
                u16::try_from(info.names.len()).expect("Currently handling at most 2^16 barcodes")
            }
        }
    }

    #[must_use]
    pub fn unwrap(&self) -> &DemultiplexInfo {
        match self {
            Self::No => panic!("Demultiplexed::unwrap() called on Demultiplexed::No"),
            Self::Yes(info) => info,
        }
    }

    #[must_use]
    pub fn get_name(&self, tag: u16) -> Option<String> {
        match self {
            Self::No => None,
            Self::Yes(info) => Some(info.names[tag as usize].clone()),
        }
    }
}
