use schemars::JsonSchema;

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema, Hash)]
#[schemars(with = "usize")]
pub struct SegmentIndex(pub usize);

//in DNA because it's used by DNA types. rest of the segment varations are in config
impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }

    pub fn first() -> SegmentIndex {
        SegmentIndex(0)
    }
}
