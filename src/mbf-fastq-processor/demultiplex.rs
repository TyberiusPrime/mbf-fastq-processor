use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::io::compressed_output::HashedAndCompressedWriter;
use crate::{config::CompressionFormat, join_nonempty};
use anyhow::{Context, Result};
use bstr::BString;

pub type Tag = u64;

#[derive(Debug)]
pub struct DemultiplexedData<T>(BTreeMap<Tag, T>);

// explicitly not DemultiplexedData, for that is uncloneable at runtime
// since we use it in the unclonable needs_serial stages
pub type DemultiplexTagToName = BTreeMap<Tag, Option<String>>;

pub type OutputWriter = HashedAndCompressedWriter<'static, ex::fs::File>;

impl std::fmt::Debug for OutputWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputWriter").finish_non_exhaustive()
    }
}

#[derive(Default, Clone)]
pub struct DemultiplexedOutputFiles(pub DemultiplexedData<Option<Box<OutputWriter>>>);

impl std::fmt::Debug for DemultiplexedOutputFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DemultiplexedOutputFiles")
            .field("outputs", &format!("n={:?}", self.0.len()))
            .finish()
    }
}

impl<T> Default for DemultiplexedData<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DemultiplexedData<T> {
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Tag, &T)> {
        self.0.iter().map(|(tag, data)| (*tag, data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Tag, &mut T)> {
        self.0.iter_mut().map(|(tag, data)| (*tag, data))
    }

    pub fn entry(&mut self, tag: Tag) -> std::collections::btree_map::Entry<'_, Tag, T> {
        self.0.entry(tag)
    }

    pub fn keys(&self) -> impl Iterator<Item = Tag> + '_ {
        self.0.keys().copied()
    }

    pub fn values(&self) -> impl Iterator<Item = &T> + '_ {
        self.0.values()
    }

    pub fn insert(&mut self, tag: Tag, data: T) {
        self.0.insert(tag, data);
    }

    #[must_use]
    pub fn get(&self, tag: &Tag) -> Option<&T> {
        self.0.get(tag)
    }
    pub fn get_mut(&mut self, tag: &Tag) -> Option<&mut T> {
        self.0.get_mut(tag)
    }
}

impl<T> IntoIterator for DemultiplexedData<T> {
    type Item = (Tag, T);
    type IntoIter =
        std::iter::Map<std::collections::btree_map::IntoIter<Tag, T>, fn((Tag, T)) -> (Tag, T)>;

    #[allow(clippy::map_identity)] // you can probably say this much better.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(|(tag, data)| (tag, data))
    }
}

impl<'a, T> IntoIterator for &'a DemultiplexedData<T> {
    type Item = (Tag, &'a T);
    type IntoIter = std::iter::Map<
        std::collections::btree_map::Iter<'a, Tag, T>,
        fn((&'a Tag, &'a T)) -> (Tag, &'a T),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(tag, data)| (*tag, data))
    }
}

impl<'a, T> IntoIterator for &'a mut DemultiplexedData<T> {
    type Item = (Tag, &'a mut T);
    type IntoIter = std::iter::Map<
        std::collections::btree_map::IterMut<'a, Tag, T>,
        fn((&'a Tag, &'a mut T)) -> (Tag, &'a mut T),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut().map(|(tag, data)| (*tag, data))
    }
}

impl<T> FromIterator<(Tag, T)> for DemultiplexedData<T> {
    fn from_iter<I: IntoIterator<Item = (Tag, T)>>(iter: I) -> Self {
        let mut map = BTreeMap::new();
        for (tag, data) in iter {
            map.insert(tag, data);
        }
        Self(map)
    }
}

impl<T> Clone for DemultiplexedData<T> {
    /// I can't ensure that only !`needs_serial steps` are cloned with the type system
    /// but I can make it fail at runtime which hopefully the tests will catch
    fn clone(&self) -> Self {
        panic!("Must not clone needs_serial stages")
    }
}

/// what the other steps need to know about the demultiplexing
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DemultiplexInfo {
    //step specific, what we need during the runtime.
    //These are full qualified demultiplex1.demultiplex2 -> tag hashes.
    //up to the current step (demultiplex2)
    pub name_to_tag: BTreeMap<BString, Tag>,
    pub tag_to_name: DemultiplexTagToName,

    pub local_barcode_to_tag: BTreeMap<BString, Tag>, //And that's the values for this specific step,
                                                      //which we then or together to get the full qualified tag.
}

impl DemultiplexInfo {
    #[must_use]
    pub fn new(tag_to_name: DemultiplexTagToName, barcode_to_tag: BTreeMap<BString, Tag>) -> Self {
        let mut name_to_tag = BTreeMap::new();
        for (tag, name_opt) in &tag_to_name {
            if let Some(name) = name_opt {
                name_to_tag.insert(BString::from(name.as_str()), *tag);
            }
        }
        Self {
            name_to_tag,
            tag_to_name,
            local_barcode_to_tag: barcode_to_tag,
        }
    }

    #[must_use]
    pub fn barcode_to_tag(&self, barcode: &[u8]) -> Option<Tag> {
        if let Some(tag) = self.local_barcode_to_tag.get(barcode) {
            return Some(*tag);
        } else if !barcode.is_empty() {
            for (bc, tag) in &self.local_barcode_to_tag {
                if bc.len() == barcode.len() && crate::dna::iupac_hamming_distance(bc, barcode) == 0
                {
                    return Some(*tag);
                }
            }
        }
        None
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct DemultiplexBarcodes {
    pub barcode_to_name: BTreeMap<BString, String>,
    pub include_no_barcode: bool,
}

// so we can abstract over whether demultiplexing is enabled or not
#[derive(Debug, Clone)]
pub enum OptDemultiplex {
    Yes(DemultiplexInfo),
    No,
}

impl OptDemultiplex {
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::No => 1,
            Self::Yes(info) => info.tag_to_name.len(),
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn unwrap(&self) -> &DemultiplexInfo {
        match self {
            Self::No => panic!("OptDemultiplex::unwrap() called on OptDemultiplex::No"),
            Self::Yes(info) => info,
        }
    }

    #[must_use]
    pub fn iter_tags(&self) -> Vec<Tag> {
        match self {
            Self::No => vec![0],
            Self::Yes(info) => info.tag_to_name.keys().copied().collect(),
        }
    }

    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<DemultiplexedOutputFiles> {
        let filenames_in_order: DemultiplexedData<Option<PathBuf>> = match self {
            Self::No => {
                let basename = join_nonempty(vec![filename_prefix, filename_suffix], ix_separator);
                let with_suffix = format!("{basename}.{filename_extension}");
                [(
                    0,
                    Some(compression_format.apply_suffix(&with_suffix).into()),
                )]
                .into_iter()
                .collect()
            }
            Self::Yes(info) => {
                let mut filenames = DemultiplexedData::new();
                /* if !info.include_no_barcode {
                    filenames.push(None)
                }
                for _ in 0..info.len_outputs() {
                    filenames.push(None)
                } */
                for (tag, name) in &info.tag_to_name {
                    filenames.insert(
                        *tag,
                        name.as_ref().map(|name| {
                            let basename = join_nonempty(
                                vec![filename_prefix, filename_suffix, name],
                                ix_separator,
                            );
                            let with_suffix = format!("{basename}.{filename_extension}");
                            let filename = compression_format.apply_suffix(&with_suffix);
                            filename.into()
                        }),
                    );
                }
                filenames
            }
        };
        let mut streams = DemultiplexedData::new();

        for (tag, opt_filename) in filenames_in_order {
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
                streams.insert(tag, Some(Box::new(buffered_writer)));
            } else {
                streams.insert(tag, None);
            }
        }
        Ok(DemultiplexedOutputFiles(streams))
    }
}
