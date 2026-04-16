use crate::transformations::prelude::*;

/// Validate that read names conform to the SAM/BAM specification.
///
/// Valid read names match `[!-?A-~]{1,254}` — printable ASCII excluding `@` and
/// space, with a maximum length of 254 characters.
#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ValidateReadNamesPrintable {
    #[schemars(skip)]
    ignored: Option<u8>, // tpd dislikes empty structs
}

impl TagUser for PartialTaggedVariant<PartialValidateReadNamesPrintable> {
    //default is ok, no tags
}

/// Returns `true` for bytes in `[!-?A-~]` (SAM QNAME character set).
fn is_valid_qname_byte(b: u8) -> bool {
    // '!' (0x21) .. '?' (0x3F)  AND  'A' (0x41) .. '~' (0x7E)
    // '@' (0x40) is explicitly excluded by the SAM spec gap between the two ranges.
    (0x21..=0x3F).contains(&b) || (0x41..=0x7E).contains(&b)
}

const MAX_QNAME_LEN: usize = 254;

impl Step for ValidateReadNamesPrintable {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let reads_in_block = block.segments[0].entries.len();
        for read_idx in 0..reads_in_block {
            let read = block.segments[0].get(read_idx);
            let name = read.name();

            if name.len() > MAX_QNAME_LEN {
                bail!(
                    "ValidateReadNamesPrintable: Read name is {} characters long, exceeding the SAM/BAM limit of {MAX_QNAME_LEN}.\n\
                     Read name (first 80 bytes): '{}'",
                    name.len(),
                    BString::from(&name[..name.len().min(80)]),
                );
            }

            if let Some(&bad_byte) = name.iter().find(|&&b| !is_valid_qname_byte(b)) {
                bail!(
                    "ValidateReadNamesPrintable: Read name '{}' contains a character not allowed by the SAM/BAM spec (byte 0x{bad_byte:02X}).\n\
                     Allowed characters: [!-?A-~] (printable ASCII, excluding '@' and space).\n\
                     Bytes: {:?}",
                    BString::from(name),
                    name,
                );
            }
        }
        Ok((block, true))
    }
}
