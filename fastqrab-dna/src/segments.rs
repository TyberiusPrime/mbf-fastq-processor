use schemars::JsonSchema;

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema, Hash)]
#[schemars(with = "u16")]
pub struct SegmentIndex(pub u16);

//in DNA because it's used by DNA types. rest of the segment varations are in config
impl SegmentIndex {
    pub fn new(idx: usize) -> Self {
        SegmentIndex(u16::try_from(idx).expect("SegementIndex must be between 0 and 65535"))
    }
    #[must_use]
    pub fn get_index(&self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn as_index(&self) -> usize {
        self.0 as usize
    }

    pub fn first() -> SegmentIndex {
        SegmentIndex(0)
    }
}

impl std::fmt::Display for SegmentIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SegmentIndex({})", self.0)
    }
}
