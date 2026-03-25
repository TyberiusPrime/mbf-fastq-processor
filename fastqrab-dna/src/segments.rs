#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

//in DNA because it's used by DNA types. rest of the segment varations are in config
impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }
}
