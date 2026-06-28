use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct InducedPanic {
    after: usize,
}

impl TagUser for PartialTaggedVariant<PartialInducedPanic> {}

impl Step for InducedPanic {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        if block.first_read_sequential_number >= self.after {
            panic!("induced test panic");
        }
        Ok((block, true))
    }
}
