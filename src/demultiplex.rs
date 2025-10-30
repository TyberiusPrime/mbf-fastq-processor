use std::collections::{BTreeMap, HashMap};
use std::path::{PathBuf, Path};

use crate::io::compressed_output::HashedAndCompressedWriter;
use crate::{config::CompressionFormat, join_nonempty};
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
}

#[derive(Debug, Clone)]
pub struct Demultiplex {
    pub demultiplexed: Demultiplexed,
    pub ix_separator: String,
}

impl Demultiplex 
{

    pub fn new(demultiplex_info: Option<DemultiplexInfo>, ix_separator: String) -> Self {
        let demultiplexed = match demultiplex_info {
            Some(info) => Demultiplexed::Yes(info),
            None => Demultiplexed::No,
        };
        Self {
            demultiplexed,
            ix_separator,
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

    #[must_use]
    pub fn open_output_streams(
        &self,
        output_directory: &Path,
        filename_prefix: &str,
        filename_suffix: &str,
        filename_extension: &str,
        ix_separator: &str,
        compression_format: CompressionFormat,
        compression_level: Option<u8>,
        hash_compressed: bool,
        hash_uncompressed: bool,
        allow_overwrite: bool,
    ) -> Result<Vec<Option<Box<HashedAndCompressedWriter<'static, ex::fs::File>>>>> {
        let filenames_in_order: Vec<Option<PathBuf>> = match self {
            Self::No => {
                let basename = join_nonempty(vec![filename_prefix, filename_suffix], &ix_separator);
                let with_suffix = format!("{}.{}", basename, filename_extension);
                vec![Some(compression_format.apply_suffix(&with_suffix).into())]
            }
            Self::Yes(info) => {
                let mut filenames = Vec::new();
                if !info.include_no_barcode {
                    filenames.push(None)
                }
                for _ in 0..info.len_outputs() {
                    filenames.push(None)
                }
                for (tag, name) in info.iter_outputs() {
                    let basename =
                        join_nonempty(vec![filename_prefix, filename_suffix, name], &ix_separator);
                    let with_suffix = format!("{}.{}", basename, filename_extension);
                    let filename = compression_format.apply_suffix(&with_suffix);
                    filenames[tag as usize] = Some(filename.into());
                }
                filenames
            }
        };
        let mut streams = Vec::new();

        for opt_filename in filenames_in_order.into_iter() {
            if let Some(filename) = opt_filename {
                let filename = output_directory.join(filename);
                crate::output::ensure_output_destination_available(&filename, allow_overwrite)?;
                let file_handle = ex::fs::File::create(&filename).with_context(|| {
                    format!("Could not open output file: {}", filename.display())
                })?;
                let buffered_writer = HashedAndCompressedWriter::new(
                    file_handle,
                    compression_format,
                    hash_uncompressed,
                    hash_compressed,
                    compression_level, 
                    None,
                )?;
                streams.push(Some(Box::new(buffered_writer)))
            } else {
                streams.push(None)
            }
        }
        Ok(streams)
    }
}
