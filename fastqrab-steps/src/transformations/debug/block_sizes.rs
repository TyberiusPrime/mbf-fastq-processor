use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
#[expect(dead_code, reason = "Must have an internal field for TDP")]
pub struct BlockSizes {
    ignored: Option<u8>, //tpd does not like empty structs
    //
    #[tpd(skip, default)]
    #[schemars(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}
impl TagUser for PartialTaggedVariant<PartialBlockSizes> {}

impl Step for BlockSizes {
    fn store_progress_output(&mut self, progress: &crate::transformations::reports::Progress) {
        self.progress_output = Some(progress.clone());
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        self.progress_output.as_ref().map(|progress| {
            let _ = progress.output(&format!(
                "Block_no: {}, Blocksize: {}",
                block.block_no(),
                block.segments[0].len()
            ));
        });
        Ok((block, true))
    }
}
