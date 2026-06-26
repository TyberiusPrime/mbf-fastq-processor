use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_dna::dna::{self, IupacExpander, iupac_overlapping};

use anyhow::{Context, Result, bail};
use bstr::BString;
use fastqrab_io::io::{apply_to_read_names_and_sequences, input::open_text_file};
use indexmap::IndexMap;
use schemars::JsonSchema;
use std::{
    collections::{BTreeMap, HashSet},
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};
use toml_pretty_deser::prelude::*;

use crate::{config::PartialConfig, no_barcode_infix};

#[derive(Clone, Debug, JsonSchema)]
#[tpd]
pub struct Barcodes {
    #[tpd(nested)]
    pub from_file: Option<BarcodesFromFile>,

    //this configuration value is empty after parsing
    //(saves turning barcodes from files into TomlValues and back).
    //
    //Values are in dna_to_name
    #[schemars(with = "BTreeMap<String, String>")]
    #[tpd(absorb_remaining)]
    pub barcode_to_name: IndexMap<BString, String>,

    // built during init.
    #[tpd(skip, default)]
    #[schemars(skip)]
    pub seq_to_name: Arc<IndexMap<BString, String>>,
}

#[tpd(no_verify)]
#[derive(Clone, Debug, JsonSchema)]
pub struct BarcodesFromFile {
    #[expect(
        dead_code,
        reason = "only used in PartialBarcodesFromFile. But necessary for logging"
    )]
    filename: String,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "read_comment_char")]
    pub read_comment_character: Option<u8>,
}

fn read_barcodes_from_txt(filename: &Path, entries: &mut IndexMap<BString, String>) -> Result<()> {
    use bstr::io::BufReadExt;
    let file = open_text_file(filename)?;
    let buffered = BufReader::new(file);
    let mut target_len = None;
    for line in buffered.byte_lines() {
        let line: BString = line?.into();
        match target_len {
            None => {
                target_len = Some(line.len());
            }
            Some(old_len) => {
                if old_len != line.len() {
                    bail!("Unequal line lengths detected in {}", filename.display());
                }
            }
        }
        let str_line: String = String::from_utf8(line.to_vec())
            .context("Expected DNA, found something non-utf8-encoded?")?;
        entries.insert(line, str_line);
    }
    Ok(())
}

impl PartialBarcodesFromFile {
    fn load_from_file(&self) -> Result<IndexMap<BString, String>, ValidationFailure> {
        let mut entries: IndexMap<BString, String> = IndexMap::new();
        let filename = self.filename.as_ref().expect("Should be checked before");
        let comment_char = self
            .read_comment_character
            .as_ref()
            .and_then(|x| x.as_ref());

        if let Err(err) =
            apply_to_read_names_and_sequences(filename, &mut |name: &[u8], seq: &[u8]| {
                let name_str = if let Some(comment_char) = comment_char
                    && let Some(pos) = name.iter().position(|&c| c == *comment_char)
                {
                    let name_without_comment = &name[..pos];
                    String::from_utf8_lossy(name_without_comment).into_owned()
                } else {
                    String::from_utf8_lossy(name).into_owned()
                };

                // duplicates are not ok, it's 1:m for seq:output_key
                if entries.insert(BString::from(seq), name_str).is_some() {
                    bail!("Duplicate sequence in {filename}: '{}'", BString::from(seq));
                }
                Ok(())
            })
        {
            if entries.is_empty() && filename.contains(".txt") {
                if let Err(second_err) =
                    read_barcodes_from_txt(&PathBuf::from(filename), &mut entries)
                {
                    return Err(ValidationFailure::new(
                        format!("Error reading from {filename}"),
                        Some(format!(
                            "Tried as sequence file:\n\t{err:?}\n\n\
                            Tried as text file of one barcode-per-line:\n\t{second_err:?}"
                        )),
                    ));
                }
            } else {
                return Err(ValidationFailure::new(
                    format!("Error reading from {filename}"),
                    Some(format!("{err:?}")),
                ));
            }
        }
        if entries.is_empty() {
            // cov:excl-start
            // empty file would lead to 'can't detect file format' before we reach this
            Err(ValidationFailure::new(
                format!("Barcodes: reference file '{filename}' contains no sequences.",),
                Some("Check your input file.".to_string()),
            ))
            //cov:excl-end
        } else {
            Ok(entries)
        }
    }
}

impl VerifyIn<PartialConfig> for PartialBarcodes {
    #[expect(clippy::too_many_lines, reason = "needed")]
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.barcode_to_name.verify_keys(|key| {
            if let Some(None) = self.from_file.as_ref() {
                if key == "filename" {
                    return Err(ValidationFailure::new(
                        "Missing from_file.",
                        Some("Use from_file = {filename = '...'} to read barcodes from a file"))); 
                } else if key == "read_comment_character" {
                    return Err(ValidationFailure::new(
                        "Missing from_file.",
                        Some("Use from_file = {filename = '...', read_comment_character = 'x'} to read barcodes from a file"))); }
                if dna::all_iupac_or_underscore(key.as_bytes()) {
                    Ok(())
                } else {
                    Err(ValidationFailure::new(
                            "Invalid IUPAC (uppercase only)",
                            Some("See https://en.wikipedia.org/wiki/International_Union_of_Pure_and_Applied_Chemistry#Amino_acid_and_nucleotide_base_codes")
                    ))}
            } else {
                Ok(())
            }
        });
        if let Some(values) = self.barcode_to_name.as_mut() {
            if let Some(bad_value) = values
                .map
                .values_mut()
                .find(|v| v.as_ref().map(String::as_str) == Some(no_barcode_infix()))
            {
                bad_value.state = TomlValueState::new_validation_failed(format!(
                    "Must not be '{}'",
                    no_barcode_infix()
                ));
                bad_value.help = Some(
                    "Choose a different name for your barcode, \
                this one is reserved for not-matched reads."
                        .to_string(),
                );
                return Ok(());
            }
            if values.map.is_empty()
                && (self.from_file.as_ref().is_none()
                    || matches!(self.from_file.as_ref(), Some(None)))
            {
                return Err(ValidationFailure::new(
                    "Empty barcode section",
                    Some(
                        "Add at least one barcode mapping (IUPAC='name') under this section,\n\
                        or read them from a file using from_file={filename='...'}",
                    ),
                ));
            }
        }

        let iupac_to_name: Option<IndexMap<BString, String>> =
            if let Some(Some(from_file)) = self.from_file.as_ref() {
                if let Some(barcode_to_name) = self.barcode_to_name.as_ref()
                    && !barcode_to_name.map.is_empty()
                {
                    let spans = vec![
                        (
                            self.from_file.span.clone(),
                            "Conflicts with inline barcodes".to_string(),
                        ),
                        (
                            self.barcode_to_name.span.clone(),
                            "Conflicts with from_file".to_string(),
                        ),
                    ];
                    self.from_file.state = TomlValueState::Custom { spans };
                    self.from_file.help = Some(
                        "Remove either the manual barcode definition or the from_file section."
                            .to_string(),
                    );
                    None
                } else {
                    //we are good to read the barcodes
                    let barcode_to_name = from_file.load_from_file()?;
                    Some(barcode_to_name)
                }
            } else if let Some(barcode_to_name) = self.barcode_to_name.as_mut()
                && matches!(self.from_file.as_ref(), Some(None))
            {
                // we implicitly check disjointness when expanding iupac below,
                // but this allows pinpointing the offending barcodes in the config
                //  (which we don't offer for the file based barcode config
                if validate_barcode_disjointness(barcode_to_name) {
                    Some(
                        barcode_to_name
                            .map
                            .iter()
                            .map(|(k, v)| (k.clone(), v.value.clone().expect("parent was ok")))
                            .collect(),
                    )
                } else {
                    None
                }
            } else {
                None
            };

        //now, no matter where the barcodes came from...

        if let Some(iupac_to_name) = iupac_to_name {
            let mut dna_to_name: IndexMap<BString, String> = IndexMap::new();

            if !iupac_to_name.is_empty()
                && let first_len = iupac_to_name
                    .keys()
                    .next()
                    .map(|k| k.len())
                    .expect("just checked")
                && iupac_to_name
                    .iter()
                    .any(|(iupac, _)| iupac.len() != first_len)
            {
                let lengths: HashSet<_> = iupac_to_name.keys().map(|k| k.len()).collect();
                let mut lengths: Vec<_> = lengths.into_iter().collect();
                lengths.sort_unstable();
                return Err(ValidationFailure::new(
                    "Barcodes of different lengths".to_string(),
                    Some(format!(
                        "All barcodes in one section must have the same length. Observed lengths: {lengths:?}"
                    )),
                ));
            }

            for (iupac, name) in &iupac_to_name {
                //this isn't free (about 100ms for 1.5 million barcodes
                //we could hide it behind an option, but I don' think it's worth it right now.
                for dna in IupacExpander::new(iupac.as_ref()) {
                    if let Some(old) = dna_to_name.insert(dna.clone(), name.clone())
                        && old != *name
                    {
                        // it's ok if we essentially replaced them with themselves
                        return Err(ValidationFailure::new(
                            format!(
                                "Overlapping sequences after iupac expansion: {dna} {name} -> {old}"
                            ),
                            Some("Verify your input barcodes are disjoint.".to_string()),
                        ));
                    }
                }
            }

            self.barcode_to_name.value = Some(MapAndKeys {
                keys: vec![],
                map: IndexMap::new(),
            });
            self.barcode_to_name.state = TomlValueState::Ok;
            self.seq_to_name = Some(Arc::new(dna_to_name));
        }

        Ok(())
    }
}

/// Validate that IUPAC barcodes are disjoint (don't overlap in their accepted sequences)
#[mutants::skip] // yeah, modifying to for j in (i * 1) will still 'work', just perform more checks
fn validate_barcode_disjointness(barcodes: &mut MapAndKeys<BString, String>) -> bool {
    // First pass: collect all overlapping pairs without mutating anything.
    // We must not assign while iterating because one barcode can overlap multiple others
    // (e.g. NNNN overlaps both ATCG and RYRN); assigning in-loop would overwrite earlier results.
    struct OverlapPair {
        index_i: usize,
        index_j: usize,
        spans: Vec<(std::ops::Range<usize>, String)>,
    }
    let mut overlapping_pairs: Vec<OverlapPair> = Vec::new();

    for index_i in 0..barcodes.keys.len() {
        for index_j in (index_i + 1)..barcodes.keys.len() {
            if let Some(dna_a) = barcodes.keys[index_i].value.as_ref()
                && let Some(dna_b) = barcodes.keys[index_j].value.as_ref()
                && let Some(barcode_name_a) = barcodes
                    .map
                    .get(bstr::BStr::new(dna_a))
                    .and_then(|x| x.as_ref())
                && let Some(barcode_name_b) = barcodes
                    .map
                    .get(bstr::BStr::new(dna_b))
                    .and_then(|x| x.as_ref())
                && barcode_name_a != barcode_name_b
                && iupac_overlapping(dna_a.as_bytes(), dna_b.as_bytes())
            {
                let spans = vec![
                    (
                        barcodes.keys[index_i].span(),
                        format!("Overlaps with {dna_b}"),
                    ),
                    (
                        barcodes.keys[index_j].span(),
                        format!("Overlaps with {dna_a}"),
                    ),
                ];
                overlapping_pairs.push(OverlapPair {
                    index_i,
                    index_j,
                    spans,
                });
            }
        }
    }

    // Second pass: assign each pair's error to the first barcode in the pair that hasn't
    // already been used as an error anchor, so every overlap gets its own error entry.
    let mut assigned: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut err = false;
    for op in overlapping_pairs {
        let error_idx = if assigned.contains(&op.index_i) {
            op.index_j
        } else {
            op.index_i
        };
        barcodes.keys[error_idx].state = TomlValueState::Custom { spans: op.spans };
        barcodes.keys[error_idx].help =
            Some("IUPAC patterns overlap, but lead to different barcodes.".to_string());
        assigned.insert(error_idx);
        err = true;
    }
    !err
}
