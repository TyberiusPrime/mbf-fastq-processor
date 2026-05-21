use bstr::{BStr, BString};
use indexmap::IndexMap;
use std::collections::{BTreeMap, HashMap};

use fastqrab_io::io::output::chunked_writer::{ChunkedRecordWriter, TextRecordSink};

pub type Tag = u64;

#[derive(Debug)]
pub struct DemultiplexedData<T>(BTreeMap<Tag, T>);

// explicitly not DemultiplexedData, for that is uncloneable at runtime
// since we use it in the unclonable needs_serial stages
pub type DemultiplexTagToName = BTreeMap<Tag, Option<String>>;

#[derive(Default, Clone)]
pub struct DemultiplexedOutputFiles(pub DemultiplexedData<Option<Box<TextRecordSink>>>);

// cov:excl-start
impl std::fmt::Debug for DemultiplexedOutputFiles {
    #[mutants::skip] // never used, but it' s useful when you need to debug
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DemultiplexedOutputFiles")
            .field("outputs", &format!("n={:?}", self.0.len()))
            .finish()
    }
}
// cov:excl-stop

/// Per-step output writers passed to [`Step::init`].
///
/// Keyed by the `id` from each [`OutputDeclaration`]; the value is one
/// [`ChunkedRecordWriter`] per active demultiplex tag (or just tag `0` when
/// there is no demultiplexing).
pub struct StepOutputFiles(pub HashMap<String, DemultiplexedData<ChunkedRecordWriter>>);

impl StepOutputFiles {
    #[must_use]
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Take all writers for a declared output id. Panics if the id is unknown.
    #[must_use]
    pub fn take(&mut self, id: &str) -> DemultiplexedData<ChunkedRecordWriter> {
        self.0
            .remove(id)
            .unwrap_or_else(|| panic!("StepOutputFiles: unknown output id '{id}'"))
    }

    /// Insert writers for a declared output id.
    pub fn insert(&mut self, id: String, data: DemultiplexedData<ChunkedRecordWriter>) {
        self.0.insert(id, data);
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

    #[expect(clippy::len_without_is_empty, reason = "Never queried for is_empty")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // pub fn iter(&self) -> impl Iterator<Item = (Tag, &T)> {
    //     self.0.iter().map(|(tag, data)| (*tag, data))
    // }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Tag, &mut T)> {
        self.0.iter_mut().map(|(tag, data)| (*tag, data))
    }

    pub fn entry(&mut self, tag: Tag) -> std::collections::btree_map::Entry<'_, Tag, T> {
        self.0.entry(tag)
    }

    // pub fn keys(&self) -> impl Iterator<Item = Tag> + '_ {
    //     self.0.keys().copied()
    // }

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

    #[must_use]
    pub fn get_mut(&mut self, tag: &Tag) -> Option<&mut T> {
        self.0.get_mut(tag)
    }

    #[must_use]
    pub fn remove(&mut self, tag: &Tag) -> Option<T> {
        self.0.remove(tag)
    }

    #[must_use]
    pub fn replace(&mut self, other: DemultiplexedData<T>) -> DemultiplexedData<T> {
        let old = std::mem::replace(&mut self.0, other.0);
        DemultiplexedData(old)
    }
}

impl<T> IntoIterator for DemultiplexedData<T> {
    type Item = (Tag, T);
    type IntoIter = std::collections::btree_map::IntoIter<u64, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
#[expect(
    clippy::module_name_repetitions,
    reason = "Info by itself is not informative"
)]
pub struct DemultiplexInfo {
    //step specific, what we need during the runtime.
    //These are full qualified demultiplex1.demultiplex2 -> tag hashes.
    //up to the current step (demultiplex2)
    pub name_to_tag: BTreeMap<BString, Tag>,
    pub tag_to_name: DemultiplexTagToName,

    pub local_barcode_to_tag: BTreeMap<BString, Tag>, //And that's the values for this specific step,
    //which we then or together to get the full qualified tag.
    pub local_name_to_tag: BTreeMap<String, Tag>, //for the 'non lookup' demultiplexes
}

impl DemultiplexInfo {
    #[must_use]
    pub fn new(
        tag_to_name: DemultiplexTagToName,
        local_barcode_to_tag: BTreeMap<BString, Tag>,
        local_name_to_tag: BTreeMap<String, Tag>,
    ) -> Self {
        let mut name_to_tag = BTreeMap::new();
        for (tag, name_opt) in &tag_to_name {
            if let Some(name) = name_opt {
                name_to_tag.insert(BString::from(name.as_str()), *tag);
            }
        }
        Self {
            name_to_tag,
            tag_to_name,
            local_barcode_to_tag,
            local_name_to_tag,
        }
    }

    #[must_use]
    pub fn barcode_to_tag(&self, barcode: &[u8]) -> Option<Tag> {
        if let Some(tag) = self.local_barcode_to_tag.get(barcode) {
            return Some(*tag);
        } else if let Some(tag) = self
            .local_barcode_to_tag
            .get(BStr::new(&barcode.to_ascii_uppercase()))
        {
            return Some(*tag);
        }
        None
    }

    #[must_use]
    pub fn name_to_tag(&self, name: &str) -> Option<Tag> {
        self.local_name_to_tag.get(name).copied()
    }
}

#[expect(
    clippy::module_name_repetitions,
    reason = "Info by itself is not informative"
)]
pub struct DemultiplexBarcodes {
    pub barcode_to_name: IndexMap<BString, String>,
    pub include_no_barcode: bool,
}

// so we can abstract over whether demultiplexing is enabled or not
#[derive(Debug, Clone)]
pub enum OptDemultiplex {
    Yes(DemultiplexInfo),
    No,
}

impl OptDemultiplex {
    #[expect(clippy::len_without_is_empty, reason = "Never queried for is_empty")]
    #[must_use]
    #[mutants::skip] // only used by initial filter capacity calculation
    pub fn len(&self) -> usize {
        match self {
            Self::No => 1,
            Self::Yes(info) => info.tag_to_name.len(),
        }
    }

    /// # Panics
    /// when called on a `OptDemultiplex::No` - as the name suggests
    #[must_use]
    pub fn expect(&self, msg: &str) -> &DemultiplexInfo {
        match self {
            Self::No => {
                // cov:excl-start
                panic!("OptDemultiplex::expect() called on OptDemultiplex::No. Message was {msg}")
                // cov:excl-stop
            }
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
}

#[cfg(test)]
mod test {
    #[test]
    #[should_panic(expected = "Must not clone needs_serial stages")]
    fn cant_clone_demultiplexed_data() {
        let data: super::DemultiplexedData<u32> = super::DemultiplexedData::new();
        let _cloned = data.clone();
    }
}
