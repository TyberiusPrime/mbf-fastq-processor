use schemars::JsonSchema;

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema, Hash)]
#[schemars(with = "u8")]
pub struct SegmentIndex(pub u8);

//in DNA because it's used by DNA types. rest of the segment varations are in config
impl SegmentIndex {
    /// Create a new `SegmentIndex` between 0..256
    /// # Panics
    /// On values outside that range.
    #[must_use]
    pub fn new(idx: usize) -> Self {
        SegmentIndex(u8::try_from(idx).expect("SegementIndex must be between 0 and 255"))
    }
    #[must_use]
    pub fn get_index(&self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn as_index(&self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub fn first() -> SegmentIndex {
        SegmentIndex(0)
    }
}

// impl std::fmt::Display for SegmentIndex {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "SegmentIndex({})", self.0)
//     }
// }

impl From<u32> for SegmentIndex {
    fn from(value: u32) -> Self {
        SegmentIndex::new(value as usize)
    }
}

// `SegmentIndex` is a scalar wrapper with no aliases; let TpdAliasTree recurse past it.
toml_pretty_deser::tpd_alias_leaf!(SegmentIndex);
