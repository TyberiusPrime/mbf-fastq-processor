use crate::dna;
use bstr::BString;
use serde::{Deserialize, Deserializer, de};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::{fmt, marker::PhantomData};

use anyhow::{Context, Result, anyhow, bail};
use toml_edit::{Item, Value};

pub trait FromToml {
    fn from_toml(value: &toml_edit::Item) -> Result<Self>
    where
        Self: Sized;
}

pub trait FromTomlTable {
    fn from_toml_table(item: &toml_edit::Table) -> Result<Self>
    where
        Self: Sized;
}

impl<T: FromTomlTable> FromToml for T {
    fn from_toml(item: &toml_edit::Item) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_toml_table(item.as_table().context("Expected a [table]")?)
    }
}

impl FromToml for String {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        match value {
            Item::Value(Value::String(s)) => Ok(s.value().to_string()),
            item => Err(anyhow!("Wrong type: {}, expected string", item.type_name())),
        }
    }
}

impl FromToml for bool {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        match value {
            Item::Value(Value::Boolean(b)) => Ok(*b.value()),
            item => Err(anyhow!("Wrong type: {}, expected bool", item.type_name())),
        }
    }
}
impl FromToml for u8 {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed usize range")?),
            item => Err(anyhow!("Wrong type: {}, expected u8", item.type_name())),
        }
    }
}

impl FromToml for usize {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed usize range")?),
            item => Err(anyhow!("Wrong type: {}, expected usize", item.type_name())),
        }
    }
}

impl FromToml for i32 {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed i32 range")?),
            item => Err(anyhow!("Wrong type: {}, expected i32", item.type_name())),
        }
    }
}

//
impl FromToml for Vec<String> {
    fn from_toml(value: &toml_edit::Item) -> Result<Self> {
        let arr = value.as_array().context("Expected array of strings")?;
        let mut result = Vec::new();
        for item in arr.iter() {
            match item {
                Value::String(s) => result.push(s.value().to_string()),
                other => {
                    return Err(anyhow!(
                        "Wrong type in array: was {}, expected string",
                        other.type_name()
                    ));
                }
            }
        }
        Ok(result)
    }
}

impl<T: FromTomlTable> FromToml for Vec<T> {
    fn from_toml(value: &toml_edit::Item) -> Result<Self>
    where
        Self: Sized,
    {
        let steps = value
            .as_array_of_tables()
            .context("Must be an array of tables")?;
        let res: Vec<Result<T>> = steps
            .iter()
            .map(|value| T::from_toml_table(value))
            .collect();
        if res.iter().all(|r| r.is_ok()) {
            Ok(res.into_iter().map(|r| r.unwrap()).collect())
        } else {
            let first_err = res.into_iter().find(|r| r.is_err()).unwrap().err().unwrap();
            Err(first_err)
        }
    }
}

pub trait TableExt {
    fn getx<T>(&self, key: &str) -> Result<T>
    where
        T: FromToml;

    fn getx_opt<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: FromToml;

    fn getx_clamped<T>(&self, key: &str, minimum: Option<T>, maximum: Option<T>) -> Result<T>
    where
        T: FromToml + PartialOrd + Display;

    fn getx_opt_clamped<T>(
        &self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> Result<Option<T>>
    where
        T: FromToml + PartialOrd + Display;

    //
    fn getx_opt_u8_from_char_or_number(
        &self,
        key: &str,

        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> Result<Option<u8>>;
}

trait TomlContext<T> {
    fn toml_context(self, f: &toml_edit::Item) -> Result<T>;
}

impl<T> TomlContext<T> for anyhow::Result<T> {
    fn toml_context(self, f: &toml_edit::Item) -> Result<T> {
        match f.span() {
            None => self.context("No line information available"),
            Some(span) => {
                self.with_context(|| format!("Byte-location: {}..{}", span.start, span.end))
            }
        }
    }
}

impl TableExt for toml_edit::Table {
    fn getx<T>(&self, key: &str) -> Result<T>
    where
        T: FromToml,
    {
        match self.get(key) {
            Some(x) => Ok(T::from_toml(x).with_context(|| format!("Key: {key}"))?),
            None => bail!("Missing key: {key}"),
        }
    }

    fn getx_opt<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: FromToml,
    {
        Ok(match self.get(key) {
            Some(x) => Some(T::from_toml(x).with_context(|| format!("Key: {key}"))?),
            None => None,
        })
    }

    fn getx_clamped<T>(&self, key: &str, minimum: Option<T>, maximum: Option<T>) -> Result<T>
    where
        T: FromToml + PartialOrd + Display,
    {
        Ok(match self.get(key) {
            Some(x) => {
                let x: T = T::from_toml(x).with_context(|| format!("Key: {key}"))?;
                if let Some(minimum) = minimum
                    && x < minimum
                {
                    bail!("Key: {key}. Expected value >= {minimum}")
                }
                if let Some(maximum) = maximum
                    && maximum > x
                {
                    bail!("Key: {key}. Expected value <= {maximum}")
                }
                x
            }
            None => bail!("Missing key: {key}"),
        })
    }
    fn getx_opt_clamped<T>(
        &self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> Result<Option<T>>
    where
        T: FromToml + PartialOrd + Display,
    {
        Ok(match self.get(key) {
            Some(f) => {
                let x: T = T::from_toml(f).with_context(|| format!("Key: {key}"))?;
                if let Some(minimum) = minimum
                    && x < minimum
                {
                    return Err(anyhow!("Key: {key}. Expected a value >= {minimum}."))
                        .toml_context(f);
                }
                if let Some(maximum) = maximum
                    && maximum > x
                {
                    return Err(anyhow!("Key: {key}. Expected a value <= {maximum}."))
                        .toml_context(f);
                }
                Some(x)
            }
            None => None,
        })
    }

    //
    // fn get_enum<F>(&self, key: &str) -> Result<F> where F: FromToml
    // {
    //     let v = self.get(key).with_context(|| format!("{key} not found"))?;
    //     Ok(FromToml::from_toml(v)?)
    // }
    //
    //
    // fn get_optional<T, F>(&self, key: &str, func: F) -> Result<Option<T>>
    // where
    //     F: FnOnce(&Item) -> Result<T>,
    // {
    //     match self.get(key) {
    //         None => Ok(None),
    //         Some(v) => Ok(Some(func(v)?)),
    //     }
    // }
    //
    // fn get_optional_table<T, F>(&self, key: &str, func: F) -> Result<Option<T>>
    // where
    //     F: FnOnce(&Table) -> Result<T>,
    // {
    //     match self.get(key) {
    //         None => Ok(None),
    //         Some(Item::Table(v)) => Ok(Some(func(v)?)),
    //         Some(_) => bail!("Expected a table for {key}"),
    //     }
    // }
    //
    // fn get_optional_string(&self, key: &str) -> Result<Option<String>> {
    //     match self.get(key) {
    //         None => Ok(None),
    //         Some(_) => self.getx(key).map(|x| Some(x)), //todo: remove double lookup
    //     }
    // }
    //
    // fn get_bool(&self, key: &str) -> Result<bool> {
    //     match self.get(key) {
    //         Some(Item::Value(Value::Boolean(s))) => Ok(*s.value()),
    //         Some(item) => Err(anyhow!(
    //             "Wrong type: {}: was {}, expected bool",
    //             key,
    //             item.type_name()
    //         )),
    //         None => Err(anyhow!("Missing field {}", key)),
    //     }
    // }
    //
    // fn get_optional_bool(&self, key: &str) -> Result<Option<bool>> {
    //     match self.get(key) {
    //         None => Ok(None),
    //         Some(_) => self.get_bool(key).map(|x| Some(x)), //todo: remove double lookup
    //     }
    // }
    //
    // fn get_optional_vec_str(&self, key: &str) -> Result<Option<Vec<String>>> {
    //     self.get_optional(key, |v| {
    //         let arr = v.as_array().context("Expected array of strings")?;
    //         let mut result = Vec::new();
    //         for item in arr.iter() {
    //             match item {
    //                 Value::String(s) => result.push(s.value().to_string()),
    //                 other => {
    //                     return Err(anyhow!(
    //                         "Wrong type in array {}: was {}, expected string",
    //                         key,
    //                         other.type_name()
    //                     ));
    //                 }
    //             }
    //         }
    //         Ok(result)
    //     })
    // }
    //
    fn getx_opt_u8_from_char_or_number(
        &self,
        key: &str,
        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> Result<Option<u8>> {
        let res = self.get(key).map(|v| -> Result<u8> {
            match v {
                Item::Value(Value::Integer(v)) => {
                    Ok((*v.value()).try_into().with_context(|| {
                        format!("{key} Must be a number between 0 and 255 (or a byte character)")
                    })?)
                }
                Item::Value(Value::String(s)) => {
                    let b = s.value().as_bytes();
                    if b.len() != 1 {
                        bail!(
                            "{key} Must be a single character string or a number between 0 and 255"
                        )
                    }
                    Ok(b[0])
                }
                _ => bail!("{key} Must be a single character string or a number between 0 and 255"),
            }
        }).transpose()?;
        if let Some(res) = res {
            if let Some(minimum) = minimum
                && res < minimum
            {
                bail!("{key} must be >= {minimum}")
            }
            if let Some(maximum) = maximum
                && res > maximum
            {
                bail!("{key} must be <= {maximum}")
            }
        }
        Ok(res)
    }
    // fn get_usize(
    //     &self,
    //     key: &str,
    //     minimum: Option<usize>,
    //     maximum: Option<usize>,
    // ) -> Result<usize> {
    //     match self.get(key) {
    //         Some(Item::Value(Value::Integer(s))) => {
    //             let v: usize = (*s.value())
    //                 .try_into()
    //                 .with_context(|| format!("{key}: Must be positive"))?;
    //             if let Some(minimum) = minimum
    //                 && v < minimum
    //             {
    //                 bail!("{key} must be >= {minimum}")
    //             }
    //             if let Some(maximum) = maximum
    //                 && v > maximum
    //             {
    //                 bail!("{key} must be <= {maximum}")
    //             }
    //             Ok(v)
    //         }
    //         Some(item) => Err(anyhow!(
    //             "Wrong type: {}: was {}, expected string",
    //             key,
    //             item.type_name()
    //         )),
    //         None => Err(anyhow!("Missing field {}", key)),
    //     }
    // }
    //
    // fn get_optional_usize(
    //     &self,
    //     key: &str,
    //     minimum: Option<usize>,
    //     maximum: Option<usize>,
    // ) -> Result<Option<usize>> {
    //     self.get_optional(key, |value| match value {
    //         Item::Value(Value::Integer(s)) => {
    //             let v: usize = (*s.value())
    //                 .try_into()
    //                 .with_context(|| format!("{key}: Must be positive"))?;
    //             if let Some(minimum) = minimum
    //                 && v < minimum
    //             {
    //                 bail!("{key} must be >= {minimum}")
    //             }
    //             if let Some(maximum) = maximum
    //                 && v > maximum
    //             {
    //                 bail!("{key} must be <= {maximum}")
    //             }
    //             Ok(v)
    //         }
    //         item => Err(anyhow!(
    //             "Wrong type: {}: was {}, expected string",
    //             key,
    //             item.type_name()
    //         )),
    //     })
    // }
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
