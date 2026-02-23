use crate::dna;
use bstr::BString;
use schemars::JsonSchema;
/// all our serde deserializers in one place.
///
use serde::{Deserialize, Deserializer, de};
use std::collections::BTreeMap;
use std::{fmt, marker::PhantomData};
use toml_pretty_deser::{TomlValue, ValidationFailure};

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
    use regex::bytes::Regex;
    input.try_map(|s| match Regex::new(s) {
        Ok(r) => Ok(r),
        Err(e) => Err(ValidationFailure::new(
            "Invalid regex".to_string(),
            Some(format!("Regex engine error: {e}")),
        )),
    })
}

#[must_use]
pub fn tpd_adapt_trim_string(mut input: TomlValue<String>) -> TomlValue<String> {
    input.try_map(|x| Ok(x.trim().to_string()))
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

pub(crate) fn default_comment_insert_char() -> u8 {
    b' '
}

pub fn deserialize_map_of_string_or_seq_string<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapStringOrVec(PhantomData<BTreeMap<String, Vec<String>>>);

    impl<'de> de::Visitor<'de> for MapStringOrVec {
        type Value = BTreeMap<String, Vec<String>>;

        #[mutants::skip] // I have no idea how to trigger this code path
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and string or list of strings values")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut result = Vec::new();

            while let Some(key) = map.next_key::<String>()? {
                let value = map.next_value_seed(StringOrVecSeed)?;
                result.push((key, value));
            }
            result.sort();
            let result = result.into_iter().collect();

            Ok(result)
        }
    }

    struct StringOrVecSeed;

    impl<'de> de::DeserializeSeed<'de> for StringOrVecSeed {
        type Value = Vec<String>;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct StringOrVec;

            impl<'de> de::Visitor<'de> for StringOrVec {
                type Value = Vec<String>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("string or list of strings")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(vec![value.to_owned()])
                }

                fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
                where
                    S: de::SeqAccess<'de>,
                {
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
                }
            }

            deserializer.deserialize_any(StringOrVec)
        }
    }

    deserializer.deserialize_map(MapStringOrVec(PhantomData))
}

/* pub fn string_or_seq_string_or_none<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Option<Vec<String>>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![value.to_owned()]))
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Ok(Some(Deserialize::deserialize(
                de::value::SeqAccessDeserializer::new(visitor),
            )?))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
} */

pub fn filename_or_filenames<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Option<Vec<String>>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("filename (string) or list of filenames")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}
/// also accepts '-'
pub fn btreemap_iupac_dna_string_from_string<'de, D>(
    deserializer: D,
) -> core::result::Result<BTreeMap<BString, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: BTreeMap<String, String> = Deserialize::deserialize(deserializer)?;
    //we store them without separators
    let s: Result<Vec<(BString, String)>, _> = s
        .into_iter()
        .map(|(k, v)| {
            let filtered_k: String = k
                .to_uppercase()
                .chars()
                .filter(|c| {
                    matches!(
                        c,
                        'A' | 'C'
                            | 'G'
                            | 'T'
                            | 'N'
                            | 'I'
                            | 'R'
                            | 'Y'
                            | 'S'
                            | 'W'
                            | 'K'
                            | 'M'
                            | 'B'
                            | 'D'
                            | 'H'
                            | 'V'
                            | '_'
                    )
                })
                .collect();
            if filtered_k.len() != k.chars().count() {
                return Err(serde::de::Error::custom(format!(
                    "Invalid DNA base in : '{k}'"
                )));
            }
            Ok((filtered_k.as_bytes().into(), v))
        })
        .collect();
    let mut s = s?;
    s.sort();
    Ok(s.into_iter().collect())
}

pub fn bstring_from_string<'de, D>(deserializer: D) -> core::result::Result<BString, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.as_bytes().into())
}

// pub fn option_bstring_from_string<'de, D>(
//     deserializer: D,
// ) -> core::result::Result<Option<BString>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let o: Option<String> = Deserialize::deserialize(deserializer)?;
//     Ok(o.map(|s| s.as_bytes().into()))
// }

pub fn u8_regex_from_string<'de, D>(
    deserializer: D,
) -> core::result::Result<regex::bytes::Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let re = regex::bytes::Regex::new(&s)
        .map_err(|e| serde::de::Error::custom(format!("Invalid regex: {e}")))?;
    Ok(re)
}

pub fn dna_from_string<'de, D>(deserializer: D) -> core::result::Result<BString, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    //check whether it's DNA bases...
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N') {
            return Err(serde::de::Error::custom(format!("Invalid DNA base: {c}")));
        }
    }
    Ok(s.as_bytes().into())
}

pub fn iupac_from_string<'de, D>(deserializer: D) -> core::result::Result<BString, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    if !dna::all_iupac(s.as_bytes()) {
        return Err(serde::de::Error::custom(
            format!("Invalid IUPAC base: {s}",),
        ));
    }
    Ok(s.as_bytes().into())
}

pub fn iupac_string_or_list<'de, D>(deserializer: D) -> core::result::Result<Vec<BString>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    let value = StringOrVec::deserialize(deserializer)?;

    let strings = match value {
        StringOrVec::String(s) => vec![s],
        StringOrVec::Vec(v) => {
            if v.is_empty() {
                return Err(Error::custom("search cannot be an empty list"));
            }
            v
        }
    };

    // Validate each string is valid IUPAC and uppercase it
    let mut result: Vec<BString> = Vec::new();
    for s in strings {
        let s = s.to_uppercase();
        if !dna::all_iupac(s.as_bytes()) {
            return Err(Error::custom(format!("Invalid IUPAC base: {s}")));
        }
        result.push(s.as_bytes().into());
    }

    /* // Check for overlapping patterns (distinctness check)
    * I don't think that's necessary or sufficient, since variable length entries might still
    * overlap. so we don't check for it.
    for i in 0..result.len() {
        for j in (i + 1)..result.len() {
            if dna::iupac_overlapping(&result[i], &result[j]) {
                return Err(Error::custom(format!(
                    "IUPAC patterns '{}' and '{}' can match the same sequence and are not distinct",
                    std::str::from_utf8(&result[i]).unwrap(),
                    std::str::from_utf8(&result[j]).unwrap()
                )));
            }
        }
    } */

    Ok(result)
}

pub fn base_or_dot<'de, D>(deserializer: D) -> core::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    if s.len() != 1 {
        return Err(serde::de::Error::custom(format!(
            "Single DNA base or '.' only): was '{s}'",
        )));
    }
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N' | '.') {
            return Err(serde::de::Error::custom(format!(
                "Invalid DNA base ('.' for 'any' is also allowed): {c}",
            )));
        }
    }
    let out = s.as_bytes()[0];
    Ok(out)
}

pub fn u8_from_char_or_number<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl serde::de::Visitor<'_> for Visitor {
        type Value = u8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("either a byte character or a number 0..255")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u8::try_from(v).map_err(|_| E::custom("Number must be between 0 and 255"))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u8::try_from(v).map_err(|_| E::custom("Number must be between 0 and 255"))
        }

        #[mutants::skip] // I think that never happens with TOML. Also trivially right.
        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        #[mutants::skip] // I think that never happens with TOML. Also trivially right.
        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            v.try_into()
                .map_err(|_| E::custom("Number must be between 0 and 255"))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v.len() {
                0 => Err(E::custom("empty string")),
                1 => Ok(v
                    .bytes()
                    .next()
                    .expect("single char string must have exactly one byte")),
                _ => Err(E::custom("string should be exactly one character long")),
            }
        }
    }

    deserializer.deserialize_any(Visitor)
}

pub fn opt_u8_from_char_or_number<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Option<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional byte character or a number 0..255")
        }

        #[mutants::skip] // I think that never happens with TOML (no Null value). Also trivially right.
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            u8_from_char_or_number(deserializer).map(Some)
        }
    }

    deserializer.deserialize_option(Visitor)
}

pub fn single_u8_from_string<'de, D>(deserializer: D) -> std::result::Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    match value {
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            let bytes = s.into_bytes();
            if bytes.len() == 1 {
                Ok(Some(bytes[0]))
            } else {
                Err(serde::de::Error::custom(
                    "readname_end_char must be exactly one byte",
                ))
            }
        }
        None => Ok(None),
    }
}

#[allow(clippy::type_complexity)]
pub fn arc_mutex_option_vec_string<'de, D>(
    deserializer: D,
) -> core::result::Result<std::sync::Arc<std::sync::Mutex<Option<Vec<String>>>>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<Vec<String>> = Deserialize::deserialize(deserializer)?;
    Ok(std::sync::Arc::new(std::sync::Mutex::new(o)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "u8_from_char_or_number")]
        value: u8,
    }

    #[derive(Deserialize)]
    struct TestStructOpt {
        #[serde(deserialize_with = "opt_u8_from_char_or_number")]
        value: Option<u8>,
    }

    fn test_deserialize(input: &str) -> Result<u8, String> {
        let result: Result<TestStruct, _> = serde_json::from_str(input);
        match result {
            Ok(s) => Ok(s.value),
            Err(e) => Err(e.to_string()),
        }
    }

    fn test_deserialize_opt(input: &str) -> Result<Option<u8>, String> {
        let result: Result<TestStructOpt, _> = serde_json::from_str(input);
        match result {
            Ok(s) => Ok(s.value),
            Err(e) => Err(e.to_string()),
        }
    }

    #[test]
    fn test_u8_from_char_or_number_valid_strings() {
        assert_eq!(
            test_deserialize(r#"{"value": "A"}"#).expect("test deserialization should succeed"),
            b'A'
        );
        assert_eq!(
            test_deserialize(r#"{"value": "!"}"#).expect("test deserialization should succeed"),
            b'!'
        );
        assert_eq!(
            test_deserialize(r#"{"value": " "}"#).expect("test deserialization should succeed"),
            b' '
        );
        assert_eq!(
            test_deserialize(r#"{"value": "0"}"#).expect("test deserialization should succeed"),
            b'0'
        );
        assert_eq!(
            test_deserialize(r#"{"value": "~"}"#).expect("test deserialization should succeed"),
            b'~'
        );
    }

    #[test]
    fn test_u8_from_char_or_number_empty_string() {
        let result = test_deserialize(r#"{"value": ""}"#);
        assert!(result.expect_err("Expected err").contains("empty string"));
    }

    #[test]
    fn test_u8_from_char_or_number_multi_character_string() {
        let result = test_deserialize(r#"{"value": "ab"}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("string should be exactly one character long")
        );

        let result = test_deserialize(r#"{"value": "123"}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("string should be exactly one character long")
        );
    }

    #[test]
    fn test_u8_from_char_or_number_valid_numbers() {
        assert_eq!(
            test_deserialize(r#"{"value": 0}"#).expect("test deserialization should succeed"),
            0
        );
        assert_eq!(
            test_deserialize(r#"{"value": 127}"#).expect("test deserialization should succeed"),
            127
        );
        assert_eq!(
            test_deserialize(r#"{"value": 255}"#).expect("test deserialization should succeed"),
            255
        );
        assert_eq!(
            test_deserialize(r#"{"value": 65}"#).expect("test deserialization should succeed"),
            65
        );
    }

    #[test]
    fn test_u8_from_char_or_number_negative_numbers() {
        let result = test_deserialize(r#"{"value": -1}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("Number must be between 0 and 255")
        );

        let result = test_deserialize(r#"{"value": -128}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("Number must be between 0 and 255")
        );
    }

    #[test]
    fn test_u8_from_char_or_number_out_of_range_numbers() {
        let result = test_deserialize(r#"{"value": 256}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("Number must be between 0 and 255")
        );

        let result = test_deserialize(r#"{"value": 1000}"#);
        assert!(
            result
                .expect_err("expected error")
                .contains("Number must be between 0 and 255")
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_some_string() {
        assert_eq!(
            test_deserialize_opt(r#"{"value": "A"}"#).expect("test deserialization should succeed"),
            Some(b'A')
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_some_number() {
        assert_eq!(
            test_deserialize_opt(r#"{"value": 42}"#).expect("test deserialization should succeed"),
            Some(42)
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_none() {
        assert_eq!(
            test_deserialize_opt(r#"{"value": null}"#)
                .expect("test deserialization should succeed"),
            None
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_invalid() {
        let result = test_deserialize_opt(r#"{"value": ""}"#);
        assert!(result.expect_err("expected error").contains("empty string"));
    }
}

#[derive(Clone, Debug, JsonSchema)]
#[schemars(with = "String")]
pub struct TagLabel(pub String);

/// Validates that a tag name conforms to the pattern [a-zA-Z_][a-zA-Z0-9_]*
/// (starts with a letter or underscore, followed by zero or more alphanumeric characters or underscores)
pub fn validate_tag_name(tag_name: &str) -> anyhow::Result<()> {
    use anyhow::bail;
    if tag_name.is_empty() {
        bail!(
            "Tag label cannot be empty. Please provide a non-empty tag name that starts with a letter or underscore."
        );
    }

    let mut chars = tag_name.chars();
    let first_char = chars
        .next()
        .expect("tag_name is not empty so must have at least one char");

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        bail!("Tag label must start with a letter or underscore (a-zA-Z_), got '{first_char}'",);
    }

    for (i, ch) in chars.enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Tag label must contain only letters, numbers, and underscores (a-zA-Z0-9_), found '{ch}' at position {}",
                i + 1
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
                "Reserved tag label '{forbidden}' cannot be used as a tag label. This name is reserved for {reason}. Please choose a different tag name."
            );
        }
    }
    if tag_name.starts_with("len_") {
        bail!(
            "Tag label '{tag_name}' cannot start with reserved prefix 'len_'. This prefix is reserved for length-related internal tags. Please choose a different tag name that doesn't start with 'len_'."
        );
    }
    Ok(())
}
impl TryFrom<&str> for TagLabel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match validate_tag_name(value) {
            Ok(()) => Ok(TagLabel(value.to_string())),
            Err(e) => return Err(e.to_string()),
        }
    }
}

toml_pretty_deser::impl_visitor_for_try_from_str!(TagLabel, "Invalid label");
