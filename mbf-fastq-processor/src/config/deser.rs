use crate::config::{Segment, SegmentOrAll};
use crate::dna;
use bstr::{BStr, BString};
use serde::{Deserialize, Deserializer, de};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;
use std::{fmt, marker::PhantomData};

use toml_edit::{Item, Value};

/// Collect all concurrent configuration errors
pub struct ErrorCollectorInner(Vec<Rc<RefCell<ConfigError>>>);
pub type ErrorCollector = Rc<RefCell<ErrorCollectorInner>>;

pub fn new_error_collector() -> ErrorCollector {
    Rc::new(RefCell::new(ErrorCollectorInner(Vec::new())))
}

pub trait ErrorCollectorExt {
    fn local<'a>(&self, table: &'a toml_edit::Table) -> TableErrorHelper<'a>;

    fn match_str<T: Copy>(&self, value: &toml_edit::Item, options: &[(&str, T)]) -> TomlResult<T>;

    fn add_item<T>(&self, item: &toml_edit::Item, msg: &str, help: &str) -> TomlResult<T>;
    fn add_table<T>(&self, table: &toml_edit::Table, msg: &str, help: &str) -> TomlResult<T>;
    fn add_value<T>(&self, value: &toml_edit::Value, msg: &str, help: &str) -> TomlResult<T>;
    fn add_key<T>(&self, key: &toml_edit::Key) -> TomlResult<T>;

    fn invalid_value<T>(&self, item: &toml_edit::Item, allowed: &[&str]) -> TomlResult<T>;

    fn render(&self, source: &str, source_name: &str) -> String;
}

impl ErrorCollectorExt for ErrorCollector {
    fn local<'a>(&self, table: &'a toml_edit::Table) -> TableErrorHelper<'a> {
        TableErrorHelper {
            table,
            allowed: Vec::new(),
            errors: self.clone(),
            unknown_handled: false,
        }
    }

    fn match_str<T: Copy>(&self, item: &toml_edit::Item, options: &[(&str, T)]) -> TomlResult<T> {
        if let Some(str_value) = item.as_str() {
            for (key, v) in options.iter() {
                if *key == str_value {
                    return Ok(*v);
                }
            }
            let allowed: Vec<_> = options.iter().map(|(s, _)| *s).collect();
            self.invalid_value(item, &allowed)
        } else {
            let allowed: Vec<_> = options.iter().map(|(s, _)| *s).collect();
            self.invalid_value(item, &allowed)
        }
    }

    fn add_item<T>(&self, item: &toml_edit::Item, msg: &str, help: &str) -> TomlResult<T> {
        let e = Rc::new(RefCell::new(ConfigError::from_item(msg, help, item)));
        self.as_ref().borrow_mut().0.push(e.clone());
        Err(e)
    }

    fn add_table<T>(&self, table: &toml_edit::Table, msg: &str, help: &str) -> TomlResult<T> {
        let span = table.span().map(|span| {
            let start = span.start;
            let mut end = span.end;
            for (k, v) in table.iter() {
                let s = match v {
                    Item::None => None,
                    Item::Value(value) => value.span(),
                    Item::Table(table) => table.span(),
                    Item::ArrayOfTables(array_of_tables) => array_of_tables.span(),
                };
                if let Some(s) = s
                    && s.end > end
                {
                    end = s.end;
                }
            }
            start..end
        });

        let e = Rc::new(RefCell::new(ConfigError::from_span(msg, help, span)));
        self.as_ref().borrow_mut().0.push(e.clone());
        Err(e)
    }

    fn add_value<T>(&self, value: &toml_edit::Value, msg: &str, help: &str) -> TomlResult<T> {
        let e = Rc::new(RefCell::new(ConfigError::new(msg, help, value.span())));
        self.as_ref().borrow_mut().0.push(e.clone());
        Err(e)
    }
    fn add_key<T>(&self, key: &toml_edit::Key) -> TomlResult<T> {
        let e = Rc::new(RefCell::new(ConfigError::new(
            "Invalid key",
            "Check documentation",
            key.span(),
        )));
        self.as_ref().borrow_mut().0.push(e.clone());
        Err(e)
    }

    fn invalid_value<T>(&self, item: &toml_edit::Item, allowed: &[&str]) -> TomlResult<T> {
        let seen_value = item.as_str();
        let msg = if let Some(seen_value) = seen_value {
            format!("Invalid value: {}", seen_value)
        } else {
            format!("Invalid value (wrong type")
        };
        let help = "Use one of the allowed values: ".to_string() + &allowed.join(", ");
        self.add_item(item, &msg, &help)
    }

    fn render(&self, source: &str, source_name: &str) -> String {
        let mut res = String::new();
        let mut first = true;
        for err in self.borrow().0.iter() {
            if !first {
                res += "\n"; //empty line between multiple errors
            } else {
                first = false;
            }
            res += &err.borrow().render(source, source_name);
        }
        res
    }
}

pub struct TableErrorHelper<'a> {
    pub table: &'a toml_edit::Table,
    allowed: Vec<String>,
    errors: ErrorCollector,
    unknown_handled: bool,
}

impl Drop for TableErrorHelper<'_> {
    fn drop(&mut self) {
        if !self.unknown_handled {
            if !std::thread::panicking() {
                panic!(
                    "TableErrorHelper dropped without calling accept_unknown() or deny_unknown(). Run with backtrace to debug"
                );
            }
        }
    }
}

fn format_range<T>(minimum: Option<T>, maximum: Option<T>) -> String
where
    T: Display,
{
    match (minimum, maximum) {
        (Some(min), Some(max)) => format!("[{}..{}]", min, max),
        (Some(min), None) => format!("[{}..]", min),
        (None, Some(max)) => format!("[..{}]", max),
        (None, None) => "[..]".to_string(),
    }
}

impl<'a> TableErrorHelper<'a> {
    pub fn local(&self, table: &'a toml_edit::Table) -> TableErrorHelper<'a> {
        TableErrorHelper {
            table,
            allowed: self.allowed.clone(),
            errors: self.errors.clone(),
            unknown_handled: false,
        }
    }

    pub fn get<T>(&mut self, key: &str) -> TomlResult<T>
    where
        T: FromToml,
    {
        self.allowed.push(key.to_string());
        match self.table.get(key) {
            Some(x) => Ok(T::from_toml(x, &self.errors)?),
            None => self
                .errors
                .add_table(self.table, &format!("Missing key: {}:", key), ""),
        }
    }

    pub fn get_segment(&mut self) -> TomlResult<Segment> {
        self.get::<String>("Segment").map(Into::into)
    }
    pub fn get_segment_all(&mut self) -> TomlResult<SegmentOrAll> {
        self.get::<String>("Segment").map(Into::into)
    }

    pub fn get_opt<T>(&mut self, key: &str) -> TomlResult<Option<T>>
    where
        T: FromToml,
    {
        self.allowed.push(key.to_string());
        Ok(match self.table.get(key) {
            Some(x) => Some(T::from_toml(x, &self.errors)?),
            None => None,
        })
    }

    pub fn get_clamped<T>(
        &mut self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> TomlResult<T>
    where
        T: FromToml + PartialOrd + Display,
    {
        self.allowed.push(key.to_string());
        match self.table.get(key) {
            Some(item) => {
                let x: T = T::from_toml(item, &self.errors)?;
                if let Some(mm) = minimum.as_ref()
                    && x < *mm
                {
                    return self.errors.add_item(
                        item,
                        &format!("Expected value >= {mm}"),
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }
                if let Some(mm) = maximum.as_ref()
                    && *mm > x
                {
                    return self.errors.add_item(
                        item,
                        &format!("Expected value <= {mm}"),
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }
                Ok(x)
            }
            None => self.errors.add_table(self.table, "Missing key: {key}", ""),
        }
    }

    pub fn get_opt_clamped<T>(
        &mut self,
        key: &str,
        minimum: Option<T>,
        maximum: Option<T>,
    ) -> TomlResult<Option<T>>
    where
        T: FromToml + PartialOrd + Display,
    {
        self.allowed.push(key.to_string());
        Ok(match self.table.get(key) {
            Some(item) => {
                let x: T = T::from_toml(item, &self.errors)?;
                if let Some(mm) = minimum.as_ref()
                    && x < *mm
                {
                    return self.errors.add_item(
                        item,
                        &format!("Expected value >= {mm}"),
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }
                if let Some(mm) = maximum.as_ref()
                    && *mm > x
                {
                    return self.errors.add_item(
                        item,
                        &format!("Expected value <= {mm}"),
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }

                Some(x)
            }
            None => None,
        })
    }

    pub fn get_opt_u8_from_char_or_number(
        &mut self,
        key: &str,
        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> TomlResult<Option<u8>> {
        match self.table.get(key) {
            Some(item) => {
                let res: Option<u8> = match item {
                    Item::Value(Value::Integer(i)) => (*i.value()).try_into().ok(),
                    Item::Value(Value::String(s)) => {
                        let b = s.value().as_bytes();
                        if b.len() != 1 { None } else { Some(b[0]) }
                    }
                    _ => None,
                };
                if let Some(res) = res {
                    if let Some(mm) = minimum
                        && res < mm
                    {
                        return self.errors.add_item(
                            item,
                            &format!("Expected value >= {mm}"),
                            &format!("Supply value in {})", format_range(minimum, maximum)),
                        );
                    }
                    if let Some(mm) = maximum
                        && mm > res
                    {
                        return self.errors.add_item(
                            item,
                            &format!("Expected value <= {mm}"),
                            &format!("Supply value in {})", format_range(minimum, maximum)),
                        );
                    }
                    Ok(Some(res))
                } else {
                    return self.errors.add_item(
                        item,
                        "Must be a single character string or a number between 0 and 255",
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }
            }
            None => Ok(None),
        }
    }

    pub fn accept_unknown(&mut self) {
        self.unknown_handled = true;
    }

    pub fn deny_unknown(&mut self) -> TomlResult<()> {
        self.unknown_handled = true;
        dbg!(&self.allowed);
        let mut first_err = Ok(());
        for (key, _) in self.table.iter() {
            dbg!(key);
            if !self.allowed.iter().any(|x| *x == key) {
                if let Err(e) = self
                    .errors
                    .add_key::<()>(self.table.key(key).expect("just iterated it"))
                {
                    first_err = Err(e.clone());
                }
            }
        }
        first_err
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConfigError {
    message: String,
    help: String,
    span: Option<std::ops::Range<usize>>,
}

impl ConfigError {
    pub fn new(
        msg: impl ToString,
        help: impl ToString,
        span: Option<std::ops::Range<usize>>,
    ) -> Self {
        Self {
            message: msg.to_string(),
            help: help.to_string(),
            span,
        }
    }

    pub fn from_item(msg: &str, help: &str, item: &toml_edit::Item) -> Self {
        Self::new(msg, help, item.span())
    }

    pub fn from_table(msg: &str, help: &str, table: &toml_edit::Table) -> Self {
        Self::new(msg, help, table.span())
    }

    pub fn from_span(msg: &str, help: &str, span: Option<std::ops::Range<usize>>) -> Self {
        Self::new(msg, help, span)
    }

    fn find_last_block_start(&self, source: &[u8]) -> (usize, usize) {
        use bstr::ByteSlice;
        let last_newline_pos = source.rfind(b"\n").unwrap_or(0);
        let last_block_start = source.rfind(b"[").unwrap_or(0);
        (last_newline_pos, last_block_start)
    }

    pub fn render(&self, source: &str, source_name: &str) -> String {
        use bstr::ByteSlice;
        use codesnake::{Block, CodeWidth, Label, LineIndex};
        use std::fmt::Write;

        if let Some(span) = self.span.as_ref() {
            let idx = LineIndex::new(source);
            let block = Block::new(
                &idx,
                [Label::new(span.clone()).with_text(&self.message[..])].into_iter(),
            )
            .unwrap();

            let previous_newline = memchr::memmem::rfind(&source.as_bytes()[..span.start], b"\n");

            let (lines_before, digits_needed) = match previous_newline {
                None => ("".to_string(), 1),
                Some(previous_newline) => {
                    let upto_span = &BStr::new(source.as_bytes())[..previous_newline];

                    let lines: Vec<_> = upto_span.lines().collect();
                    let str_line_no = format!("{}", lines.len());
                    let digits_needed = str_line_no.len();
                    let mut lines_before: Vec<_> = lines
                        .into_iter()
                        .enumerate()
                        .map(|(line_no, line)| (line_no, line))
                        .rev()
                        .take_while(|x| !BStr::new(x.1).trim_ascii_start().starts_with(b"["))
                        .map(|(line_no, line)| {
                            format!(
                                "{:>digits_needed$} │ {}",
                                line_no + 1,
                                std::string::String::from_utf8_lossy(line)
                            )
                        })
                        .collect();
                    lines_before.reverse();
                    (lines_before.join("\n"), digits_needed)
                }
            };
            let block = block.map_code(|c| CodeWidth::new(c, c.len()));
            let mut out = String::new();
            writeln!(&mut out, "{}{}", block.prologue(), source_name).expect("can't fail");
            write!(
                &mut out,
                " {:digits_needed$}┆\n{}\n",
                " ", lines_before
            )
            .expect("can't fail");
            let blockf: String = format!("{}", block)
                .lines()
                .skip(1)
                .map(|x| format!("{x}\n"))
                .collect();
            write!(&mut out, "{}", blockf).expect("can't fail");
            writeln!(&mut out, "{}", block.epilogue()).expect("can't fail");
            if self.help != "" {
                let mut first = true;
                write!(&mut out, "Hint: ").expect("Can't fail");
                for line in self.help.lines() {
                    if !first {
                        write!(&mut out, "      ").expect("can't fail");
                    }
                    first = false;
                    writeln!(&mut out, "{}", line).expect("can't fail");
                }
            }
            out
        } else {
            format!(
                "ConfigError at unknown location. Message {}. Help text: {}",
                &self.message, &self.help
            )
        }
    }
}

pub type TomlResult<T> = Result<T, Rc<RefCell<ConfigError>>>;

pub trait TomlResultExt {
    fn add_help(self, text: &str) -> Self;
}

impl<T> TomlResultExt for TomlResult<T> {
    fn add_help(self, text: &str) -> Self {
        if let Err(ce) = &self {
            ce.borrow_mut().help = format!("{}\n{}", ce.borrow().help, text);
        }
        self
    }
}

pub trait FromToml {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized;
}

pub trait FromTomlTable {
    fn from_toml_table(item: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized;
}

impl<T: FromTomlTable> FromToml for T {
    fn from_toml(item: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        match item.as_table() {
            Some(table) => Ok(Self::from_toml_table(table, collector)?),
            None => collector.add_item(item, "Expected a table", "Compare with documentation"),
        }
    }
}

pub trait FromTomlTableNested {
    fn from_toml_table(item: &toml_edit::Table, helper: TableErrorHelper) -> TomlResult<Self>
    where
        Self: Sized;
}

impl FromToml for String {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match value {
            Item::Value(Value::String(s)) => Ok(s.value().to_string()),
            item => collector.add_item(
                value,
                &format!("Expected a string, found {}", item.type_name()),
                "",
            ),
        }
    }
}
impl FromToml for bstr::BString {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match value {
            Item::Value(Value::String(s)) => Ok(s.value().to_string().into()),
            item => collector.add_item(
                value,
                &format!("Expected a string, found {}", item.type_name()),
                "",
            ),
        }
    }
}

impl FromToml for bool {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Boolean(s)) => Ok(*s.value()),
            item => collector.add_item(
                value,
                &format!(
                    "Expected a boolean (True/False), found {}",
                    item.type_name()
                ),
                "",
            ),
        }
    }
}

impl FromToml for usize {
    fn from_toml(item: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match item {
            Item::Value(Value::Integer(s)) => {
                let value: Result<usize, _> = (*s.value()).try_into();
                match value {
                    Ok(v) => Ok(v),
                    Err(_) => collector.add_item(
                        item,
                        "Expected a usize, found value outside usize range",
                        "Positive numbers only",
                    ),
                }
            }
            item => collector.add_item(
                item,
                &format!("Expected a usize, found {}", item.type_name()),
                "",
            ),
        }
    }
}

impl FromToml for u8 {
    fn from_toml(item: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match item {
            Item::Value(Value::Integer(s)) => {
                let value: Result<u8, _> = (*s.value()).try_into();
                match value {
                    Ok(v) => Ok(v),
                    Err(_) => collector.add_item(
                        item,
                        "Expected a u8, found value outside u8 range",
                        "[0..255] only",
                    ),
                }
            }
            item => collector.add_item(
                item,
                &format!("Expected a u8, found {}", item.type_name()),
                "",
            ),
        }
    }
}

impl FromToml for i32 {
    fn from_toml(item: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match item {
            Item::Value(Value::Integer(s)) => {
                let value: Result<i32, _> = (*s.value()).try_into();
                match value {
                    Ok(v) => Ok(v),
                    Err(_) => collector.add_item(
                        item,
                        "Expected an i32, found value outside i32 range",
                        "More than +-2^31?",
                    ),
                }
            }
            item => collector.add_item(
                item,
                &format!("Expected an i32, found {}", item.type_name()),
                "",
            ),
        }
    }
}

impl FromToml for Vec<String> {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match value {
            Item::Value(Value::Array(arr)) => {
                let res: TomlResult<Vec<String>> = arr
                    .iter()
                    .map(|x| match x.as_str() {
                        Some(x) => Ok(x.to_string()),
                        None => collector.add_value(x, "Expected a string", "Quote your value?"),
                    })
                    .collect();
                res
            }
            item => collector.add_item(
                value,
                &format!("Expected an array of string, found {}", item.type_name()),
                "",
            ),
        }
    }
}

impl<T: FromTomlTable> FromToml for Vec<T> {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match value {
            toml_edit::Item::ArrayOfTables(arr) => {
                let res: TomlResult<Vec<T>> = arr
                    .iter()
                    .map(|x| T::from_toml_table(x, collector))
                    .collect();
                res
            }
            item => collector.add_item(
                value,
                &format!("Expected an array of tables, found {}", item.type_name()),
                "",
            ),
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
