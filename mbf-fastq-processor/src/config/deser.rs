use crate::dna;
use bstr::BString;
use serde::{Deserialize, Deserializer, de};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::{fmt, marker::PhantomData};

use anyhow::{anyhow, bail};
use toml_edit::{Item, Value};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConfigError {
    message: String,
    keys: Vec<String>,
    span: Option<std::ops::Range<usize>>,
    source: Option<String>,
}

impl ConfigError {
    pub fn new(msg: impl ToString, item: &toml_edit::Item) -> Self {
        Self {
            message: msg.to_string(),
            keys: Vec::new(),
            span: item.span(),
            source: None,
        }
    }

    pub fn from_table(msg: &str, table: &toml_edit::Table) -> Self {
        Self {
            message: msg.to_string(),
            keys: Vec::new(),
            span: table.span(),
            source: None,
        }
    }

    pub fn set_source(&mut self, source: String) {
        self.source = Some(source);
    }
}

impl std::fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            None => {
                f.write_str("ConfigError. Path: ")?;
                let mut first = true;
                for p in self.keys.iter().rev() {
                    if first {
                        first = false;
                    } else {
                        f.write_str(".")?;
                    }
                    f.write_str(p)?;
                }
                f.write_str(" ")?;
                f.write_str(&self.message)?;
            }

            Some(source) => f.write_str("todo")?,
        }
        Ok(())
    }
}

// impl Display for ConfigError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.write_str(&self.message)?; //todo
//         Ok(())
//     }
// }

pub type TomlResult<T> = Result<T, ConfigError>;

pub trait TomlToAnyhow<T> {
    fn to_anyhow(self) -> anyhow::Result<T>;
}

impl<T> TomlToAnyhow<T> for TomlResult<T> {
    fn to_anyhow(self) -> anyhow::Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow::anyhow!(format!("{:?}", e))),
        }
    }
}

pub trait TomlResultContext<T> {
    fn context(self, msg: &str, item: &toml_edit::Item) -> TomlResult<T>;
    fn with_context<F: FnOnce() -> String>(
        self,
        callback: F,
        item: &toml_edit::Item,
    ) -> TomlResult<T>;
}

impl<T> TomlResultContext<T> for Option<T> {
    fn context(self, msg: &str, item: &toml_edit::Item) -> TomlResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(ConfigError::new(msg, item)),
        }
    }

    fn with_context<F: FnOnce() -> String>(
        self,
        callback: F,
        item: &toml_edit::Item,
    ) -> TomlResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(ConfigError::new(callback(), item)),
        }
    }
}

impl<T, E: Display> TomlResultContext<T> for Result<T, E> {
    fn context(self, msg: &str, item: &toml_edit::Item) -> TomlResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(ConfigError::new(format!("{}\n{}", msg, e), item)),
        }
    }
    fn with_context<F: FnOnce() -> String>(
        self,
        callback: F,
        item: &toml_edit::Item,
    ) -> TomlResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let mut msg = callback();
                msg.push('\n');
                msg.push_str(&format!("{}", e));
                Err(ConfigError::new(msg, item))
            }
        }
    }
}

pub trait TomlResultKeys<T> {
    fn path(self, path: &str) -> TomlResult<T>;
}

impl<T> TomlResultKeys<T> for TomlResult<T> {
    fn path(mut self, path: &str) -> TomlResult<T> {
        match self {
            Ok(_) => self,
            Err(ref mut e) => {
                e.keys.push(path.to_string());
                self
            }
        }
    }
}

pub trait FromToml {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self>
    where
        Self: Sized;
}

pub trait FromTomlTable {
    fn from_toml_table(item: &toml_edit::Table) -> TomlResult<Self>
    where
        Self: Sized;
}

impl<T: FromTomlTable> FromToml for T {
    fn from_toml(item: &toml_edit::Item) -> TomlResult<Self>
    where
        Self: Sized,
    {
        Self::from_toml_table(item.as_table().context("Expected a [table]", item)?)
    }
}

impl FromToml for String {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        match value {
            Item::Value(Value::String(s)) => Ok(s.value().to_string()),
            item => Err(ConfigError::new("Expected a string", item)),
        }
    }
}

impl FromToml for bool {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Boolean(b)) => Ok(*b.value()),
            item => Err(ConfigError::new(
                &format!("Wrong type: {}, expected bool", item.type_name()),
                value,
            )),
        }
    }
}
impl FromToml for u8 {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed usize range", value)?),
            _ => Err(ConfigError::new(
                &format!("Wrong type: {}, expected u8", value.type_name()),
                value,
            )),
        }
    }
}

impl FromToml for usize {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed usize range", value)?),
            _ => Err(ConfigError::new(
                &format!("Wrong type: {}, expected usize", value.type_name()),
                value,
            )),
        }
    }
}

impl FromToml for i32 {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Integer(b)) => Ok((*b.value())
                .try_into()
                .context("Value outside allowed i32 range", value)?),
            _ => Err(ConfigError::new(
                &format!("Wrong type: {}, expected i32", value.type_name()),
                value,
            )),
        }
    }
}

//
impl FromToml for Vec<String> {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self> {
        let arr = value
            .as_array()
            .context("Expected array of strings", value)?;
        let mut result = Vec::new();
        for item in arr.iter() {
            match item {
                Value::String(s) => result.push(s.value().to_string()),
                other => {
                    return Err(ConfigError::new(
                        &format!(
                            "Wrong type in array: was {}, expected string",
                            other.type_name()
                        ),
                        value,
                    ));
                }
            }
        }
        Ok(result)
    }
}

impl<T: FromTomlTable> FromToml for Vec<T> {
    fn from_toml(value: &toml_edit::Item) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let steps = value
            .as_array_of_tables()
            .context("Must be an array of tables", value)?;
        let res: Vec<TomlResult<T>> = steps
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
    fn getx<T>(&self, key: &str) -> TomlResult<T>
    where
        T: FromToml;

    fn getx_opt<T>(&self, key: &str) -> TomlResult<Option<T>>
    where
        T: FromToml;

    fn getx_clamped<T>(&self, key: &str, minimum: Option<T>, maximum: Option<T>) -> TomlResult<T>
    where
        T: FromToml + PartialOrd + Display;

    fn getx_opt_clamped<T>(
        &self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> TomlResult<Option<T>>
    where
        T: FromToml + PartialOrd + Display;

    //
    fn getx_opt_u8_from_char_or_number(
        &self,
        key: &str,

        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> TomlResult<Option<u8>>;
}

impl TableExt for toml_edit::Table {
    fn getx<T>(&self, key: &str) -> TomlResult<T>
    where
        T: FromToml,
    {
        match self.get(key) {
            Some(x) => Ok(T::from_toml(x).path(key)?),
            None => Err(ConfigError::from_table("Missing key", self)).path(key),
        }
    }

    fn getx_opt<T>(&self, key: &str) -> TomlResult<Option<T>>
    where
        T: FromToml,
    {
        Ok(match self.get(key) {
            Some(x) => Some(T::from_toml(x).path(key)?),
            None => None,
        })
    }

    fn getx_clamped<T>(&self, key: &str, minimum: Option<T>, maximum: Option<T>) -> TomlResult<T>
    where
        T: FromToml + PartialOrd + Display,
    {
        match self.get(key) {
            Some(y) => {
                let x: T = T::from_toml(y).path(key)?;
                if let Some(minimum) = minimum
                    && x < minimum
                {
                    return Err(ConfigError::new(format!("Expected value >= {minimum}"), y))
                        .path(key);
                }
                if let Some(maximum) = maximum
                    && maximum > x
                {
                    return Err(ConfigError::new(format!("Expected value <= {maximum}"), y))
                        .path(key);
                }

                Ok(x)
            }
            None => Err(ConfigError::from_table("Missing key: {key}", self)),
        }
    }
    fn getx_opt_clamped<T>(
        &self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> TomlResult<Option<T>>
    where
        T: FromToml + PartialOrd + Display,
    {
        Ok(match self.get(key) {
            Some(y) => {
                let x: T = T::from_toml(y).path(key)?;
                if let Some(minimum) = minimum
                    && x < minimum
                {
                    return Err(ConfigError::new("Expected value >= {minimum}", y)).path(key);
                }
                if let Some(maximum) = maximum
                    && maximum > x
                {
                    return Err(ConfigError::new("Expected value <= {maximum}", y)).path(key);
                }

                Some(x)
            }
            None => None,
        })
    }

    fn getx_opt_u8_from_char_or_number(
        &self,
        key: &str,
        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> TomlResult<Option<u8>> {
        match self.get(key) {
            Some(v) => {
                let res: Option<u8> = match v {
                    Item::Value(Value::Integer(i)) => (*i.value()).try_into().ok(),
                    Item::Value(Value::String(s)) => {
                        let b = s.value().as_bytes();
                        if b.len() != 1 { None } else { Some(b[0]) }
                    }
                    _ => None,
                };
                if let Some(res) = res {
                    if let Some(minimum) = minimum
                        && res < minimum
                    {
                        return Err(ConfigError::new("Expected value >= {minimum}", v)).path(key);
                    }
                    if let Some(maximum) = maximum
                        && maximum > res
                    {
                        return Err(ConfigError::new("Expected value <= {maximum}", v)).path(key);
                    }
                    Ok(Some(res))
                } else {
                    return Err(ConfigError::new(
                        "Must be a single character string or a number between 0 and 255",
                        v,
                    ))
                    .path(key);
                }
            }
            None => Ok(None),
        }
    }
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
