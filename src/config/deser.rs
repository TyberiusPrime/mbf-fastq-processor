use crate::dna;
use bstr::BString;
/// all our serde deserializers in one place.
///
use serde::{Deserialize, Deserializer, de};
use std::collections::{BTreeMap, HashMap};
use std::{fmt, marker::PhantomData};

pub(crate) fn default_comment_insert_char() -> u8 {
    b' '
}

pub fn deserialize_map_of_string_or_seq_string<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapStringOrVec(PhantomData<HashMap<String, Vec<String>>>);

    impl<'de> de::Visitor<'de> for MapStringOrVec {
        type Value = HashMap<String, Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys and string or list of strings values")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let mut result = HashMap::new();

            while let Some(key) = map.next_key::<String>()? {
                let value = map.next_value_seed(StringOrVecSeed)?;
                result.insert(key, value);
            }

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

pub fn string_or_seq_string_or_none<'de, D>(
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
}

pub fn string_or_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Option<Vec<String>>>);

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
            Ok(Deserialize::deserialize(
                de::value::SeqAccessDeserializer::new(visitor),
            )?)
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
    let s: Result<BTreeMap<BString, String>, _> = s
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
    s
}

pub fn bstring_from_string<'de, D>(deserializer: D) -> core::result::Result<BString, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.as_bytes().into())
}

pub fn option_bstring_from_string<'de, D>(
    deserializer: D,
) -> core::result::Result<Option<BString>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Deserialize::deserialize(deserializer)?;
    Ok(o.map(|s| s.as_bytes().into()))
}

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

pub fn option_btreemap_dna_string_from_string<'de, D>(
    deserializer: D,
) -> core::result::Result<Option<BTreeMap<BString, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<BTreeMap<String, String>> = Option::deserialize(deserializer)?;
    let result = match s {
        Some(map) => {
            let s: BTreeMap<BString, String> = map
                .into_iter()
                .map(|(k, v)| {
                    let k: String = k
                        .to_uppercase()
                        .chars()
                        .filter(|c| matches!(c, 'A' | 'C' | 'G' | 'T' | 'N'))
                        .collect();
                    (k.as_bytes().into(), v)
                })
                .collect();
            Some(s)
        }
        None => None,
    };
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

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

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
                1 => Ok(v.bytes().next().unwrap()),
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
        assert_eq!(test_deserialize(r#"{"value": "A"}"#).unwrap(), b'A');
        assert_eq!(test_deserialize(r#"{"value": "!"}"#).unwrap(), b'!');
        assert_eq!(test_deserialize(r#"{"value": " "}"#).unwrap(), b' ');
        assert_eq!(test_deserialize(r#"{"value": "0"}"#).unwrap(), b'0');
        assert_eq!(test_deserialize(r#"{"value": "~"}"#).unwrap(), b'~');
    }

    #[test]
    fn test_u8_from_char_or_number_empty_string() {
        let result = test_deserialize(r#"{"value": ""}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty string"));
    }

    #[test]
    fn test_u8_from_char_or_number_multi_character_string() {
        let result = test_deserialize(r#"{"value": "ab"}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("string should be exactly one character long")
        );

        let result = test_deserialize(r#"{"value": "123"}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("string should be exactly one character long")
        );
    }

    #[test]
    fn test_u8_from_char_or_number_valid_numbers() {
        assert_eq!(test_deserialize(r#"{"value": 0}"#).unwrap(), 0);
        assert_eq!(test_deserialize(r#"{"value": 127}"#).unwrap(), 127);
        assert_eq!(test_deserialize(r#"{"value": 255}"#).unwrap(), 255);
        assert_eq!(test_deserialize(r#"{"value": 65}"#).unwrap(), 65);
    }

    #[test]
    fn test_u8_from_char_or_number_negative_numbers() {
        let result = test_deserialize(r#"{"value": -1}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Number must be between 0 and 255")
        );

        let result = test_deserialize(r#"{"value": -128}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Number must be between 0 and 255")
        );
    }

    #[test]
    fn test_u8_from_char_or_number_out_of_range_numbers() {
        let result = test_deserialize(r#"{"value": 256}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Number must be between 0 and 255")
        );

        let result = test_deserialize(r#"{"value": 1000}"#);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Number must be between 0 and 255")
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_some_string() {
        assert_eq!(
            test_deserialize_opt(r#"{"value": "A"}"#).unwrap(),
            Some(b'A')
        );
    }

    #[test]
    fn test_opt_u8_from_char_or_number_some_number() {
        assert_eq!(test_deserialize_opt(r#"{"value": 42}"#).unwrap(), Some(42));
    }

    #[test]
    fn test_opt_u8_from_char_or_number_none() {
        assert_eq!(test_deserialize_opt(r#"{"value": null}"#).unwrap(), None);
    }

    #[test]
    fn test_opt_u8_from_char_or_number_invalid() {
        let result = test_deserialize_opt(r#"{"value": ""}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty string"));
    }
}
