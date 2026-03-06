use anyhow::{Result, bail};
use bstr::BString;
use regex::bytes::Regex;
use schemars::JsonSchema;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use toml_pretty_deser::{TomlValue, TomlValueState, ValidationFailure};

pub mod dna;
pub mod segments;
//
// Default functions for common values
pub fn default_region_separator() -> bstr::BString {
    b"_".into()
}

pub fn default_segment_all() -> segments::SegmentIndexOrAll {
    segments::SegmentIndexOrAll::All
}

pub fn default_comment_separator() -> u8 {
    b'|'
}
pub fn default_comment_insert_char() -> u8 {
    b' '
}


// Schema helper for string or list of strings
#[derive(JsonSchema)]
#[allow(dead_code)]
pub enum StringOrVecString {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TagValueType {
    //Todo: should this be a struct with 4 bools?
    Location, // string + in-sequence-location
    String,   // just a piece of text
    Numeric,
    Bool,
}

#[derive(Debug)]
pub struct UsedTag<'a> {
    pub name: TagLabel,
    pub accepted_tag_types: &'a [TagValueType],
    pub toml_source: Rc<RefCell<(&'a mut TomlValueState, &'a mut Option<String>)>>,
    pub further_help: Option<String>,
}

impl UsedTag<'_> {
    pub fn add_help(mut self, line: impl AsRef<str>) -> Self {
        self.further_help = match self.further_help.take() {
            Some(existing) => Some(format!("{}\n{}", existing, line.as_ref())),
            None => Some(line.as_ref().to_string()),
        };
        self
    }
}

pub trait ToUsedTag {
    fn to_used_tag<'a>(&'a mut self, accepted_tag_types: &'a [TagValueType])
    -> Option<UsedTag<'a>>;
}

pub trait ToUsedTags {
    fn to_used_tags(&mut self) -> Vec<Option<UsedTag<'_>>>;
}

#[derive(Debug)]
pub struct DeclaredTag<'a> {
    pub name: TagLabel,
    pub tag_type: TagValueType,
    pub toml_source_state: &'a mut TomlValueState,
    pub toml_source_help: &'a mut Option<String>,
    pub toml_source_span: std::ops::Range<usize>,
}

pub trait ToDeclaredTag {
    fn to_declared_tag(&mut self, tag_type: TagValueType) -> Option<DeclaredTag<'_>>;
}
//see deser for impl

#[derive(Default, Debug)]
pub enum RemovedTags<'a> {
    #[default]
    None,
    All,
    Some(Vec<(TagLabel, &'a mut TomlValue<TagLabel>)>),
}
impl TagValueType {
    pub fn compatible(self, other: TagValueType) -> bool {
        self == other
    }
}

impl std::fmt::Display for TagValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValueType::Location => write!(f, "Location"),
            TagValueType::String => write!(f, "String"),
            TagValueType::Numeric => write!(f, "Numeric"),
            TagValueType::Bool => write!(f, "Boolean"),
        }
    }
}

#[must_use]
pub fn tpd_adapt_bstring(input: TomlValue<String>) -> TomlValue<BString> {
    input.map(|s| BString::from(s.as_bytes()))
}

#[must_use]
pub fn tpd_adapt_bstring_uppercase(input: TomlValue<String>) -> TomlValue<BString> {
    input.map(|s| BString::from(s.as_bytes().to_ascii_uppercase()))
}

#[must_use]
pub fn tpd_adapt_dna_bstring(mut input: TomlValue<String>) -> TomlValue<BString> {
    input.try_map(|s| {
        let res = BString::from(s.as_bytes());
        for c in res.iter() {
            let c = c.to_ascii_uppercase();
            if !matches!(c, b'A' | b'C' | b'G' | b'T') {
                return Err(ValidationFailure::new(
                    format!("Invalid DNA base: '{c}'."),
                    None,
                ));
            }
        }
        Ok(res)
    })
}

#[must_use]
pub fn tpd_adapt_dna_bstring_plus_n(mut input: TomlValue<String>) -> TomlValue<BString> {
    input.try_map(|s| {
        let res = BString::from(s.as_bytes());
        for c in res.iter() {
            let c = c.to_ascii_uppercase();
            if !matches!(c, b'A' | b'C' | b'G' | b'T' | b'N') {
                return Err(ValidationFailure::new(
                    format!("Invalid DNA base: '{c}'."),
                    None,
                ));
            }
        }
        Ok(res)
    })
}

#[must_use]
pub fn tpd_adapt_iupac_bstring(mut input: TomlValue<String>) -> TomlValue<BString> {
    input.try_map(|s| {
        let res = BString::from(s.as_bytes());
        if !dna::all_iupac(res.as_ref()) {
            return Err(ValidationFailure::new(
                format!("Invalid IUPAC base in '{res}'."),
                Some("Allowed letters are AGTC I R Y S W K M B D H V N ".to_string()),
            ));
        }
        Ok(res)
    })
}

#[must_use]
pub fn tpd_adapt_regex(mut input: TomlValue<String>) -> TomlValue<regex::bytes::Regex> {
    input.try_map(|s| match Regex::new(s) {
        Ok(r) => Ok(r),
        Err(e) => Err(ValidationFailure::new(
            "Invalid regex".to_string(),
            Some(format!("Regex engine error: {e}")),
        )),
    })
}

#[must_use]
pub fn tpd_adapt_u8_from_byte_or_char(mut input: TomlValue<toml_edit::Item>) -> TomlValue<u8> {
    let help =
        "Provide either a number (0..255), or a single letter string (with an ascii character)";
    input.try_map(|input| match input {
        toml_edit::Item::Value(toml_edit::Value::Integer(v)) => {
            if let Ok(b) = TryInto::<u8>::try_into(*v.value()) {
                Ok(b)
            } else {
                Err(ValidationFailure::new(
                    format!("Integer value {v} is out of range for a byte (0..255)"),
                    Some(help.to_string()),
                ))
            }
        }
        toml_edit::Item::Value(toml_edit::Value::String(v)) => {
            let mut chars = v.value().chars();
            let first = chars.next();
            let second = chars.next();
            if let Some(first) = first
                && let None = second
                && let Ok(char_first) = TryInto::<u8>::try_into(first)
            {
                Ok(char_first)
            } else {
                Err(ValidationFailure::new("Invalid value", Some(help)))
            }
        }
        _ => Err(ValidationFailure::new(
            "Wrong type, expected a byte",
            Some(help),
        )),
    })
}

#[must_use]
pub fn tpd_adapt_extract_base_or_dot(mut input: TomlValue<String>) -> TomlValue<u8> {
    fn err() -> Result<u8, ValidationFailure> {
        Err(ValidationFailure::new(
            "Invalid DNA base",
            Some("Must be a single character: A, C, G, T, N or '.'"),
        ))
    }
    input.try_map(|input| {
        if input.len() == 1 {
            let s = input.as_bytes()[0].to_ascii_uppercase();
            if matches!(s, b'A' | b'C' | b'G' | b'T' | b'N' | b'.') {
                Ok(s)
            } else {
                err()
            }
        } else {
            err()
        }
    })
}

#[derive(Clone, Debug, JsonSchema, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[schemars(with = "String")]
pub struct TagLabel(pub String);

impl TagLabel {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for TagLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::borrow::Borrow<str> for TagLabel {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TagLabel {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToDeclaredTag for TomlValue<TagLabel> {
    fn to_declared_tag(&mut self, tag_type: TagValueType) -> Option<DeclaredTag<'_>> {
        if self.as_ref().is_some() {
            let name = self.as_ref().expect("just checked").clone();
            let span = self.span();
            Some(DeclaredTag {
                name,
                tag_type,
                toml_source_state: &mut self.state,
                toml_source_help: &mut self.help,
                toml_source_span: span,
            })
        } else {
            None
        }
    }
}
impl ToDeclaredTag for TomlValue<Option<TagLabel>> {
    fn to_declared_tag(&mut self, tag_type: TagValueType) -> Option<DeclaredTag<'_>> {
        if let Some(name) = self.as_ref().and_then(|x| x.as_ref()) {
            let span = self.span();
            Some(DeclaredTag {
                name: name.clone(),
                tag_type,
                toml_source_state: &mut self.state,
                toml_source_help: &mut self.help,
                toml_source_span: span,
            })
        } else {
            None
        }
    }
}

impl ToUsedTag for TomlValue<TagLabel> {
    fn to_used_tag<'a>(
        &'a mut self,
        accepted_tag_types: &'a [TagValueType],
    ) -> Option<UsedTag<'a>> {
        Some(UsedTag {
            name: self.as_ref().expect("parent was ok?").clone(),
            accepted_tag_types,
            toml_source: Rc::new(RefCell::new((&mut self.state, &mut self.help))),
            further_help: None,
        })
    }
}

impl ToUsedTag for TomlValue<Option<TagLabel>> {
    fn to_used_tag<'a>(
        &'a mut self,
        accepted_tag_types: &'a [TagValueType],
    ) -> Option<UsedTag<'a>> {
        let name = self.as_ref().expect("parent was ok?").as_ref();
        if let Some(name) = name {
            Some(UsedTag {
                name: name.clone(),
                accepted_tag_types,
                toml_source: Rc::new(RefCell::new((&mut self.state, &mut self.help))),
                further_help: None,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[schemars(with = "String")]
pub struct ConditionalTagLabel {
    pub tag: TagLabel,
    pub invert: bool,
}
impl TryFrom<&str> for ConditionalTagLabel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(tag) = value.strip_prefix('!') {
            Ok(ConditionalTagLabel {
                tag: TagLabel::try_from(tag)?,
                invert: true,
            })
        } else {
            Ok(ConditionalTagLabel {
                tag: TagLabel::try_from(value)?,
                invert: false,
            })
        }
    }
}
impl ToUsedTag for TomlValue<Option<ConditionalTagLabel>> {
    #[track_caller]
    fn to_used_tag<'a>(
        &'a mut self,
        accepted_tag_types: &'a [TagValueType],
    ) -> Option<UsedTag<'a>> {
        assert!(
            accepted_tag_types.is_empty(),
            "accepted_tag_types not used for ConditionalTagLabel"
        );
        let ct = self.as_ref().expect("parent was ok?").as_ref();
        if let Some(ct) = ct {
            Some(UsedTag {
                name: ct.tag.clone(),
                accepted_tag_types: &[
                    TagValueType::Bool,
                    TagValueType::Location,
                    TagValueType::String,
                ],
                toml_source: Rc::new(RefCell::new((&mut self.state, &mut self.help))),
                further_help: None,
            })
        } else {
            None
        }
    }
}

pub fn offer_alternatives<T: AsRef<str>>(current: &str, available: &[T]) -> String {
    let available: Vec<_> = available
        .iter()
        .map(|s| format!("'{}'", s.as_ref()))
        .collect();
    toml_pretty_deser::suggest_alternatives(current, &available)
}

/// Validates that a tag name conforms to the pattern [a-zA-Z_][a-zA-Z0-9_]*
/// (starts with a letter or underscore, followed by zero or more alphanumeric characters or underscores)
pub fn validate_tag_name(tag_name: &str) -> Result<()> {
    if tag_name.is_empty() {
        bail!(
            "Cannot be empty. Please provide a non-empty tag name that starts with a letter or underscore."
        );
    }

    let mut chars = tag_name.chars();
    let first_char = chars
        .next()
        .expect("tag_name is not empty so must have at least one char");

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        bail!("Must start with a letter or underscore (a-zA-Z_), got '{first_char}'",);
    }

    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Must contain only letters, numbers, and underscores (a-zA-Z0-9_), found '{ch}'.",
            );
        }
    }

    for (forbidden, reason) in &[
        ("ReadName", "the index column in StoreTagsInTable"),
        ("read_no", "read numbering in EvalExpression"),
    ] {
        if tag_name == *forbidden {
            // because that's what we store in the output tables as
            // column 0
            bail!(
                "Reserved value '{forbidden}' cannot be used here. This name is reserved for {reason}. Please choose a different value."
            );
        }
    }
    if tag_name.starts_with("len_") {
        bail!(
            "Cannot start with reserved prefix 'len_'. This prefix is reserved for length-related internal tags. Please choose a different value that doesn't start with 'len_'."
        );
    }
    Ok(())
}
impl TryFrom<&str> for TagLabel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match validate_tag_name(value) {
            Ok(()) => Ok(TagLabel(value.to_string())),
            Err(e) => Err(e.to_string()),
        }
    }
}

toml_pretty_deser::impl_visitor_for_try_from_str!(TagLabel, "Invalid label");
toml_pretty_deser::impl_visitor_for_try_from_str!(
    ConditionalTagLabel,
    "Invalid (conditional) label"
);

#[derive(Debug, Clone)]
pub struct NonAmbigousDNA(pub BString);

impl TryFrom<&str> for NonAmbigousDNA {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("DNA sequence cannot be empty".to_string());
        }
        let s = value.to_uppercase();
        for c in s.chars() {
            if !matches!(c, 'A' | 'C' | 'G' | 'T') {
                return Err(format!("Invalid DNA base: {c}"));
            }
        }
        Ok(NonAmbigousDNA(s.as_bytes().into()))
    }
}

toml_pretty_deser::impl_visitor_for_try_from_str!(NonAmbigousDNA, "Invalid DNA sequence");
