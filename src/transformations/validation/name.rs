#![allow(clippy::unnecessary_wraps)] // eserde false positives
use super::Step;
use crate::config::deser::single_u8_from_string;
use crate::demultiplex::Demultiplexed;
use crate::transformations::read_name_canonical_prefix;
use anyhow::{anyhow, bail, Result};
use bstr::BStr;
use std::cell::{Cell, RefCell};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateName {
    #[serde(default, deserialize_with = "single_u8_from_string")]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl Step for ValidateName {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if input_def.segment_count() <= 1 {
            bail!("ValidateName requires at least two input segments");
        }
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let error = RefCell::new(None);
        let read_index = Cell::new(0usize);

        block.apply_mut(|reads| {
            if error.borrow().is_some() {
                return;
            }

            let current_index = read_index.get();

            if reads.is_empty() {
                read_index.set(current_index + 1);
                return;
            }

            let reference_name = reads[0].name();
            if reference_name.is_empty() {
                *error.borrow_mut() = Some(anyhow!(
                    "Read name is empty for segment 0 at read index {current_index}"
                ));
                return;
            }

            let expected_prefix = read_name_canonical_prefix(reference_name, self.readname_end_char);

            for (segment_idx, read) in reads.iter().enumerate().skip(1) {
                let candidate_name = read.name();
                if candidate_name.is_empty() {
                    *error.borrow_mut() = Some(anyhow!(
                        "Read name is empty for segment {segment_idx} at read index {current_index}"
                    ));
                    return;
                }

                let candidate_prefix = read_name_canonical_prefix(candidate_name, self.readname_end_char);

                if candidate_prefix != expected_prefix {
                    *error.borrow_mut() = Some(anyhow!(
                        "Read name mismatch at read no {current_index} (0 based count). Expected prefix {:?} from segment 0 name {:?}, segment {segment_idx} provided prefix {:?} from name {:?}",
                        BStr::new(expected_prefix),
                        BStr::new(reference_name),
                        BStr::new(candidate_prefix),
                        BStr::new(candidate_name)
                    ));
                    return;
                }
            }

            read_index.set(current_index + 1);
        });

        if let Some(error) = error.into_inner() {
            Err(error)
        } else {
            Ok((block, true))
        }
    }
}
