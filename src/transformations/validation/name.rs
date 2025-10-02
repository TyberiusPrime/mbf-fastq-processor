#![allow(clippy::unnecessary_wraps)] // eserde false positives
use super::Step;
use crate::config::deser::single_u8_from_string;
use crate::demultiplex::Demultiplexed;
use anyhow::{Result, anyhow, bail};
use bstr::BStr;
use memchr::memchr;
use std::cell::{Cell, RefCell};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateName {
    #[serde(
        default = "default_readname_end_char",
        deserialize_with = "single_u8_from_string"
    )]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

fn default_readname_end_char() -> Option<u8> {
    Some(super::super::default_name_separator())
}

impl ValidateName {
    fn canonical_prefix<'a>(&self, name: &'a [u8]) -> &'a [u8] {
        if let Some(separator) = self.readname_end_char {
            if let Some(position) = memchr(separator, name) {
                &name[..position]
            } else {
                name
            }
        } else {
            name
        }
    }
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

            let expected_prefix = self.canonical_prefix(reference_name);

            for (segment_idx, read) in reads.iter().enumerate().skip(1) {
                let candidate_name = read.name();
                if candidate_name.is_empty() {
                    *error.borrow_mut() = Some(anyhow!(
                        "Read name is empty for segment {segment_idx} at read index {current_index}"
                    ));
                    return;
                }

                let candidate_prefix = self.canonical_prefix(candidate_name);

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

#[cfg(test)]
mod tests {
    use super::ValidateName;

    fn step_with(chars: Option<&str>) -> ValidateName {
        let readname_end_char = chars.and_then(|value| match value.as_bytes() {
            [] => None,
            [byte] => Some(*byte),
            _ => panic!("expected at most one byte"),
        });
        ValidateName { readname_end_char }
    }

    #[test]
    fn canonical_prefix_stops_at_first_separator() {
        let step = step_with(Some("_"));
        assert_eq!(step.canonical_prefix(b"Sample_1"), b"Sample");
    }

    #[test]
    fn canonical_prefix_uses_full_name_when_separator_missing() {
        let step = step_with(Some("_"));
        assert_eq!(step.canonical_prefix(b"Sample"), b"Sample");
    }

    #[test]
    fn custom_separator_is_respected() {
        let step = step_with(Some("/"));
        assert_eq!(step.canonical_prefix(b"Run/42"), b"Run");
    }

    #[test]
    fn empty_separator_yields_exact_matching() {
        let step = step_with(Some(""));
        assert_eq!(step.canonical_prefix(b"Exact"), b"Exact");
    }

    #[test]
    fn missing_separator_configuration_defaults_to_exact_match() {
        let step = step_with(None);
        assert_eq!(step.canonical_prefix(b"Exact"), b"Exact");
    }
}
