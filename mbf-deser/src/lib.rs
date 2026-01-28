#![allow(unused)]
//! # MBF Deserialization Utilities
//!
//! This crate provides utilities for deserialization and error collection
//! focused on producing user-friendly error messages.

use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    default,
    fmt::Display,
    ops::Range,
    rc::Rc,
};

use num_traits::{Bounded, FromPrimitive, NumCast, ToPrimitive};
use toml_edit::{Document, TomlError};

pub use mbf_deser_derive::make_partial;

#[derive(Debug)]
pub enum DeserError<P> {
    ParsingFailure(TomlError),
    DeserFailure(Vec<HydratedAnnotatedError>, P),
}

/// The primary entry point.
/// Given a target struct `T` and a partial variant of it called `PartialT`'
/// (made via [#make_partial] on the struct)
/// turn it either into a parsed, deserialized and validated T,
/// or a `DeserError<PartialT>`
pub fn deserialize<P, T>(source: &str) -> Result<T, DeserError<P>>
where
    P: FromTomlTable<P, ()> + ToConcrete<T> + Default,
{
    let parsed_toml = source.parse::<Document<String>>()?;
    let source = Rc::new(RefCell::new(source.to_string()));

    let mut helper = TomlHelper::new(parsed_toml.as_table());

    let mut partial = P::default();
    match P::from_toml_table(&mut helper, &mut partial) {
        Ok(_) => {}
        Err(()) => {
            return Err(DeserError::DeserFailure(
                helper.into_inner(&source),
                partial,
            ));
        }
    };
    if let Err(()) = helper.deny_unknown() {
        return Err(DeserError::DeserFailure(
            helper.into_inner(&source),
            partial,
        ));
    };

    if partial.can_concrete() {
        Ok(partial
            .to_concrete()
            .expect("Can concrete claimed it was ok!"))
    } else {
        Err(DeserError::DeserFailure(helper.into_inner(&source), partial))
    }
}

pub struct TomlHelper<'a> {
    table: &'a toml_edit::Table,
    expected: Vec<String>,
    allowed: Vec<String>,
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
}

/// Helper function to format a list of strings with single quotes
/// and correct English grammar (commas and 'or').
fn format_quoted_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [single] => format!("'{}'", single),
        [first, second] => format!("'{}' or '{}'", first, second),
        rest => {
            // Split into the initial items and the very last item
            let (last, init) = rest.split_last().unwrap();

            // Join initial items with commas and quotes
            let start = init
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ");

            // Append "or" and the final item
            format!("{}, or '{}'", start, last)
        }
    }
}

pub fn suggest_alternatives<T: AsRef<str>>(current: &str, available: &[T]) -> String {
    if current.is_empty() {
        let mut sorted: Vec<&str> = available.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
        sorted.sort();
        return format!("Available are: {}", format_quoted_list(&sorted));
    }

    let mut distances: Vec<(usize, &str)> = available
        .iter()
        .map(|item| {
            let item_str = item.as_ref();
            // strsim handles strings directly
            let dist = strsim::levenshtein(current, item_str);
            (dist, item_str)
        })
        .collect();

    distances.sort_by_key(|k| k.0);

    let closest: Vec<&str> = distances.into_iter().take(3).map(|(_, s)| s).collect();

    format!("Did you mean: {}?", format_quoted_list(&closest))
}

pub struct OptDeserResultItem<'a> {
    requested_key: String,
    used_key: Option<String>,
    item: Option<&'a toml_edit::Item>,
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    parent_span: Range<usize>,
}

impl<'a> OptDeserResultItem<'a> {
    fn ok(
        item: &'a toml_edit::Item,
        requested_key: &str,
        used_key: &str,
        parent_span: Range<usize>,
        errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    ) -> OptDeserResultItem<'a> {
        OptDeserResultItem {
            requested_key: requested_key.to_string(),
            used_key: Some(used_key.to_string()),
            item: Some(item),
            parent_span,
            errors,
        }
    }

    fn not_found(
        requested_key: &str,
        parent_span: Range<usize>,
        errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    ) -> OptDeserResultItem<'a> {
        OptDeserResultItem {
            requested_key: requested_key.to_string(),
            used_key: None,
            item: None,
            parent_span,
            errors,
        }
    }

    fn require(self) -> Result<DeserResultItem<'a>, ()> {
        match self.item {
            Some(item) => Ok(DeserResultItem {
                requested_key: self.requested_key,
                used_key: self.used_key.unwrap(),
                item: item,
                errors: self.errors.clone(),
                parent_span: self.parent_span.clone(),
            }),
            None => {
                self.errors.borrow_mut().push(AnnotatedError::placed(
                    self.parent_span,
                    &format!("Missing key: '{}'", self.requested_key),
                    "",
                ));

                Err(())
            }
        }
    }

    fn as_str(self) -> Result<Option<&'a str>, ()> {
        match &self.item {
            Some(toml_edit::Item::Value(toml_edit::Value::String(formatted))) => {
                Ok(Some(formatted.value()))
            }
            Some(item) => register_type_error(self.errors, item, "string", ""),
            None => Ok(None),
        }
    }

    fn as_number<T>(self) -> Result<Option<T>, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
        self._as_number(T::min_value(), T::max_value())
    }

    fn _as_number<T>(self, lower: T, upper: T) -> Result<Option<T>, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
        let err_msg = || format!("Supply a value between {} and {}", lower, upper);
        match &self.item {
            Some(toml_edit::Item::Value(toml_edit::Value::Integer(formatted))) => {
                let value = formatted.value();
                match T::from_i64(*value) {
                    Some(converted) => Ok(Some(converted)),
                    None => {
                        let span = formatted.span().unwrap_or(0..0);
                        self.errors.borrow_mut().push(AnnotatedError::placed(
                            span,
                            &format!("Value outside {} range", std::any::type_name::<T>()),
                            &err_msg(),
                        ));
                        Err(())
                    }
                }
            }
            Some(item) => register_type_error(self.errors, item, "integer", &err_msg()),
            None => Ok(None),
        }
    }

    fn clamp<T>(self, lower: Option<T>, upper: Option<T>) -> Result<Option<T>, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive + PartialOrd + Copy,
    {
        let errors = self.errors.clone();
        let span = self.item.and_then(|x| x.span()).unwrap_or(0..0);

        let lower = lower.unwrap_or_else(|| T::min_value());
        let upper = upper.unwrap_or_else(|| T::max_value());
        let found: Result<Option<T>, ()> = self._as_number(lower, upper);
        match found {
            Ok(Some(value)) => {
                if value < lower {
                    errors.borrow_mut().push(AnnotatedError::placed(
                        span,
                        "Value too low",
                        &format!("Supply a value between {} and {}", lower, upper),
                    ));
                    return Err(());
                } else if value > upper {
                    errors.borrow_mut().push(AnnotatedError::placed(
                        span,
                        "Value too large",
                        &format!("Supply a value between {} and {}", lower, upper),
                    ));
                    return Err(());
                } else {
                    Ok(Some(value))
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn as_enum<T: std::str::FromStr + strum::VariantNames>(self) -> Result<Option<T>, ()> {
        match &self.item {
            Some(toml_edit::Item::Value(toml_edit::Value::String(formatted))) => {
                match T::from_str(formatted.value()) {
                    Ok(value) => Ok(Some(value)),
                    Err(_e) => {
                        let available = T::VARIANTS;
                        self.errors.borrow_mut().push(AnnotatedError::placed(
                            formatted.span().expect("Span should be available"),
                            "Invalid value",
                            &suggest_alternatives(formatted.value(), available),
                        ));
                        Err(())
                    }
                }
            }
            Some(item) => {
                let available = T::VARIANTS;
                register_type_error(
                    self.errors,
                    item,
                    "string",
                    &suggest_alternatives("", available),
                )
            }
            None => Ok(None),
        }
    }
}

fn register_type_error<T>(
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    item: &toml_edit::Item,
    expected: &str,
    help: &str,
) -> Result<T, ()> {
    let span = item.span().unwrap_or(0..0);
    errors.borrow_mut().push(AnnotatedError::placed(
        span,
        &format!(
            "Unexpected type: {}, expected: {expected}",
            item.type_name()
        ),
        help,
    ));
    Err(())
}

pub struct DeserResultItem<'a> {
    requested_key: String,
    used_key: String,
    item: &'a toml_edit::Item,
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    parent_span: Range<usize>,
}

impl<'a> DeserResultItem<'a> {
    fn as_str(self) -> Result<&'a str, ()> {
        match &self.item {
            toml_edit::Item::Value(toml_edit::Value::String(formatted)) => Ok(formatted.value()),
            item => register_type_error(self.errors, item, "string", ""),
        }
    }
    fn as_number<T>(self) -> Result<T, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
        self._as_number(T::min_value(), T::max_value())
    }

    fn _as_number<T>(self, lower: T, upper: T) -> Result<T, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
        let err_msg = || format!("Supply a value between {} and {}", lower, upper);
        match &self.item {
            toml_edit::Item::Value(toml_edit::Value::Integer(formatted)) => {
                let value = formatted.value();
                match T::from_i64(*value) {
                    Some(converted) => Ok(converted),
                    None => {
                        let span = formatted.span().unwrap_or(0..0);
                        self.errors.borrow_mut().push(AnnotatedError::placed(
                            span,
                            &format!("Value outside {} range", std::any::type_name::<T>()),
                            &err_msg(),
                        ));
                        Err(())
                    }
                }
            }
            item => register_type_error(self.errors, item, "integer", &err_msg()),
        }
    }

    fn clamp<T>(self, lower: Option<T>, upper: Option<T>) -> Result<T, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive + PartialOrd + Copy,
    {
        let errors = self.errors.clone();
        let span = self.item.span().unwrap_or(0..0);
        let lower = lower.unwrap_or_else(|| T::min_value());
        let upper = upper.unwrap_or_else(|| T::max_value());
        let found: Result<T, ()> = self._as_number(lower, upper);
        match found {
            Ok(value) => {
                if value < lower {
                    errors.borrow_mut().push(AnnotatedError::placed(
                        span,
                        "Value too low",
                        &format!("Supply a value between {} and {}", lower, upper),
                    ));
                    return Err(());
                } else if value > upper {
                    errors.borrow_mut().push(AnnotatedError::placed(
                        span,
                        "Value too large",
                        &format!("Supply a value between {} and {}", lower, upper),
                    ));
                    return Err(());
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn as_enum<T: std::str::FromStr + strum::VariantNames>(self) -> Result<T, ()> {
        match &self.item {
            toml_edit::Item::Value(toml_edit::Value::String(formatted)) => {
                match T::from_str(formatted.value()) {
                    Ok(value) => Ok(value),
                    Err(_e) => {
                        let available = T::VARIANTS;
                        self.errors.borrow_mut().push(AnnotatedError::placed(
                            formatted.span().expect("Span should be available"),
                            "Invalid value",
                            &suggest_alternatives(formatted.value(), available),
                        ));
                        Err(())
                    }
                }
            }
            item => {
                let available = T::VARIANTS;
                register_type_error(
                    self.errors,
                    item,
                    "string",
                    &suggest_alternatives("", available),
                )
            }
        }
    }
}

pub trait KeyOrAlias<'b> {
    fn display(&self) -> &str;
    fn get<'a>(
        &self,
        table: &'a toml_edit::Table,
    ) -> Result<Option<(&'a str, &'a toml_edit::Item)>, Vec<String>>;
}

impl<'b> KeyOrAlias<'b> for &'b str {
    fn display(&self) -> &str {
        self
    }
    fn get<'a>(
        &self,
        table: &'a toml_edit::Table,
    ) -> Result<Option<(&'a str, &'a toml_edit::Item)>, Vec<String>> {
        let lower_key = self.to_lowercase();
        let mut hits: Vec<(&str, &toml_edit::Item)> = Vec::new();

        for (key, item) in table.iter() {
            if self.to_lowercase() == key.to_lowercase() {
                hits.push((key, item));
            }
        }

        if hits.len() == 1 {
            return Ok(Some(hits[0]));
        } else if hits.is_empty() {
            return Ok(None);
        } else {
            let found = hits.iter().map(|(key, _0)| key.to_string()).collect();
            return Err(found);
        }
    }
}

impl<'b> KeyOrAlias<'b> for &'b [&str] {
    fn display(&self) -> &str {
        self[0]
    }
    fn get<'a>(
        &self,
        table: &'a toml_edit::Table,
    ) -> Result<Option<(&'a str, &'a toml_edit::Item)>, Vec<String>> {
        let mut hits: Vec<(&str, &toml_edit::Item)> = Vec::new();
        let mut found_keys: Vec<String> = Vec::new();

        for alias in *self {
            for (key, item) in table.iter() {
                if alias.to_lowercase() == key.to_lowercase() {
                    hits.push((key, item));
                    found_keys.push(key.to_string());
                }
            }
        }

        if hits.len() == 1 {
            return Ok(Some(hits[0]));
        } else if hits.is_empty() {
            return Ok(None);
        } else {
            return Err(found_keys);
        }
    }
}

impl<'a> TomlHelper<'a> {
    fn new(table: &'a toml_edit::Table) -> TomlHelper<'a> {
        TomlHelper {
            table,
            expected: vec![],
            allowed: vec![],
            errors: Rc::new(RefCell::new(vec![])),
        }
    }

    fn into_inner(self, source: &Rc<RefCell<String>>) -> Vec<HydratedAnnotatedError> {
        self.errors
            .borrow_mut()
            .drain(..)
            .map(|wrapped| HydratedAnnotatedError {
                source: source.clone(),
                inner: Rc::into_inner(wrapped).expect("RC error").into_inner(),
            })
            .collect()
    }

    pub fn require_table(&mut self, table_key: &str) -> Result<TomlHelper<'a>, ()> {
        let item = self.get(table_key)?.item;
        match item {
            Some(toml_edit::Item::Table(table)) => Ok(TomlHelper {
                table,
                expected: vec![],
                allowed: vec![],
                errors: self.errors.clone(),
            }),
            Some(item) => register_type_error(self.errors.clone(), item, "table", "")?,
            None => {
                self.add_missing_key(table_key, "");
                Err(())
            }
        }
    }

    pub fn get<'b>(&mut self, key: impl KeyOrAlias<'b>) -> Result<OptDeserResultItem<'a>, ()> {
        let found = key.get(self.table);
        self.expected.push(key.display().to_string());
        match found {
            Ok(Some((used_key, item))) => {
                self.allowed.push(used_key.to_string());
                Ok(OptDeserResultItem::ok(
                    item,
                    key.display(),
                    used_key,
                    self.table.span().unwrap_or(0..0),
                    self.errors.clone(),
                ))
            }
            Ok(None) => Ok(OptDeserResultItem::not_found(
                key.display(),
                self.table.span().unwrap_or(0..0),
                self.errors.clone(),
            )),
            Err(found_keys) => {
                let da_err = AnnotatedError::placed(
                    self.table
                        .get(&found_keys[0])
                        .and_then(|item| item.span())
                        .expect("Key was found and then missing?!"),
                    "Multiple keys (aliases) found",
                    &format!(
                        "Please use only one of the keys, preferentially '{}' (canonical)",
                        key.display()
                    ),
                );
                for other_key in found_keys.iter().skip(1) {
                    da_err.add_span(
                        self.table
                            .get(other_key)
                            .and_then(|item| item.span())
                            .expect("Key was found and then missing?!"),
                        "Conflicts",
                    )
                }
                self.add_err(da_err);
                Err(())
            }
        }
    }

    fn add_err(&self, err: Rc<RefCell<AnnotatedError>>) {
        self.errors.borrow_mut().push(err);
    }

    fn add_err_by_key(&self, key: &str, msg: &str, help: &str) {
        let span = self
            .table
            .get(key)
            .and_then(|item| item.span())
            .unwrap_or(0..0);
        self.errors
            .borrow_mut()
            .push(AnnotatedError::placed(span, msg, help));
    }

    // fn add_err_by_item(&self, item: &toml_edit::Item, msg: &str, help: &str) {
    //     let span = item.span().unwrap_or(0..0);
    //     self.errors
    //         .borrow_mut()
    //         .push(AnnotatedError::placed(span, msg, help));
    // }
    //
    // fn add_err_by_span(&self, span: Range<usize>, msg: &str, help: &str) {
    //     self.errors
    //         .borrow_mut()
    //         .push(AnnotatedError::placed(span, msg, help));
    // }

    fn add_missing_key(&self, key: &str, help: &str) {
        let span = self.table.span().unwrap_or(0..0);
        self.errors.borrow_mut().push(AnnotatedError::placed(
            span,
            &format!("Missing key: '{}'", key),
            help,
        ));
    }

    fn deny_unknown(&self) -> Result<(), ()> {
        let mut had_unknown = false;
        for (key, _) in self.table.iter() {
            if !self.allowed.iter().any(|x| *x == key) {
                let still_available: Vec<_> = self
                    .expected
                    .iter()
                    .filter(|s| !self.allowed.contains(&s))
                    .collect();
                self.add_err_by_key(
                    key,
                    &format!("Unknown key: {key}"),
                    &suggest_alternatives(key, &still_available),
                );

                had_unknown = true;
            }
        }
        if had_unknown { Err(()) } else { Ok(()) }
    }
}

pub trait FromTomlTable<T, S> {
    fn from_toml_table(helper: &mut TomlHelper<'_>, partial: &mut T) -> Result<S, ()>
    where
        Self: Sized;
}

impl<P> From<TomlError> for DeserError<P> {
    fn from(value: TomlError) -> Self {
        DeserError::ParsingFailure(value)
    }
}

pub trait ToConcrete<T> {
    fn can_concrete(&self) -> bool;
    fn to_concrete(self) -> Option<T>;
}

#[derive(Debug, Clone)]
struct SpannedMessage {
    span: Range<usize>,
    msg: String,
}

#[derive(Debug)]
pub struct AnnotatedError {
    spans: Vec<SpannedMessage>,
    help: Option<String>,
}

#[derive(Debug)]
pub struct HydratedAnnotatedError {
    source: Rc<RefCell<String>>,
    inner: AnnotatedError,
}

impl HydratedAnnotatedError {
    pub fn pretty(&self, source_name: &str) -> String {
        use bstr::{BStr, ByteSlice};
        use codesnake::{Block, CodeWidth, Label, LineIndex};
        use std::fmt::Write;
        let source = self.source.borrow();

        if !self.inner.spans.is_empty() {
            let idx = LineIndex::new(&source);
            let mut spans = self.inner.spans.clone();
            spans.sort_by_key(|span| span.span.start);

            let previous_newline =
                memchr::memmem::rfind(&source.as_bytes()[..spans[0].span.start], b"\n");
            let mut labels = Vec::new();

            for span in spans.into_iter() {
                labels.push(Label::new(span.span).with_text(span.msg));
            }
            let block = Block::new(&idx, labels).unwrap_or_else(||{
                let mut spans = self.inner.spans.clone();
                spans.sort_by_key(|span| span.span.start);
                let span_str: Vec<_>  = spans.iter().map(|
                    span| format!("{}..{}: {}", span.span.start, span.span.end, span.msg)
                ).collect();
                let span_str = span_str.join("\n");
                panic!(
                    "Error spans overlapping so we were unable to process a pretty code block:\n {span_str}"
            );
            });

            let (lines_before, digits_needed) = match previous_newline {
                None => ("".to_string(), 1),
                Some(previous_newline) => {
                    let upto_span = &BStr::new(source.as_bytes())[..previous_newline];

                    let lines: Vec<_> = upto_span.lines().collect();
                    let str_line_no = format!("{}", lines.len());
                    let digits_needed = str_line_no.len();
                    let mut seen_opening = false;
                    let mut lines_before: Vec<_> = lines
                        .into_iter()
                        .enumerate()
                        .map(|(line_no, line)| (line_no, line))
                        .rev()
                        .take_while(move |x| {
                            if BStr::new(x.1).trim_ascii_start().starts_with(b"[") {
                                seen_opening = true;
                                true
                            } else {
                                !seen_opening
                            }
                        })
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
            write!(&mut out, " {:digits_needed$}┆\n{}\n", " ", lines_before).expect("can't fail");
            let blockf: String = format!("{}", block)
                .lines()
                .skip(1)
                .map(|x| format!("{x}\n"))
                .collect();
            write!(&mut out, "{}", blockf).expect("can't fail");
            writeln!(&mut out, "{}", block.epilogue()).expect("can't fail");
            if let Some(help) = self.inner.help.as_ref()
                && !help.is_empty()
            {
                let mut first = true;
                write!(&mut out, "Hint: ").expect("Can't fail");
                for line in help.lines() {
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
                "ConfigError at unknown location.Help text: {}",
                &self
                    .inner
                    .help
                    .as_ref()
                    .map(|x| x.as_str())
                    .unwrap_or("None available")
            )
        }
    }
}

impl AnnotatedError {
    fn unplaced(help: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(AnnotatedError {
            spans: vec![],
            help: Some(help.to_string()),
        }))
    }

    fn placed(span: Range<usize>, msg: &str, help: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(AnnotatedError {
            spans: vec![SpannedMessage {
                span,
                msg: msg.to_string(),
            }],
            help: Some(help.to_string()),
        }))
    }
}

trait AnnotatedErrorExt {
    fn get_out(self) -> AnnotatedError;

    fn add_span(&self, span: Range<usize>, msg: &str);
}

impl AnnotatedErrorExt for Rc<RefCell<AnnotatedError>> {
    fn get_out(self) -> AnnotatedError {
        Rc::into_inner(self).unwrap().into_inner()
    }

    fn add_span(&self, span: Range<usize>, msg: &str) {
        self.borrow_mut().spans.push(SpannedMessage {
            span,
            msg: msg.to_string(),
        });
    }
}

#[cfg(test)]
mod test {
    use std::default;

    use strum;

    use crate::{DeserError, FromTomlTable, ToConcrete, TomlHelper};

    use super::{deserialize, make_partial};

    #[derive(Debug)]
    struct HelloWorld {
        hello: Hello,
    }

    #[derive(Debug)]
    struct Hello {
        world: String,
    }

    #[derive(Debug, Default)]
    struct PartialHelloWorld {
        hello: Option<Hello>,
    }

    impl FromTomlTable<PartialHelloWorld, ()> for PartialHelloWorld {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialHelloWorld,
        ) -> Result<(), ()> {
            let mut hello = helper.require_table("hello")?;
            let world: &str = hello.get("world")?.require()?.as_str()?;
            let world_opt: Option<&str> = hello.get("world")?.as_str()?;
            assert_eq!(world, world_opt.unwrap());
            hello.deny_unknown()?;

            partial.hello = Some(Hello {
                world: world.to_string(),
            });

            Ok(())
        }
    }

    impl ToConcrete<HelloWorld> for PartialHelloWorld {
        fn can_concrete(&self) -> bool {
            self.hello.is_some()
        }
        fn to_concrete(self) -> Option<HelloWorld> {
            Some(HelloWorld { hello: self.hello? })
        }
    }

    #[test]
    fn test_simple() {
        let source = "[hello]\nworld='today'";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.hello.world, "today")
        }
    }

    #[test]
    fn test_missing() {
        let source = "[hello]\n";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Missing key: 'world'");
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_wrong_value_type() {
        let source = "[hello]\n world=12";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(
                    err.inner.spans[0].msg,
                    "Unexpected type: integer, expected: string"
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_not_a_table() {
        let source = "hello = 124";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(
                    err.inner.spans[0].msg,
                    "Unexpected type: integer, expected: table"
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_additional() {
        let source = "hello.world = 'hi'\nhallo=123";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Unknown key: hallo");
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_additional_inner() {
        let source = "hello.world = 'hi'\nhello.shu=123";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Unknown key: shu");
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[derive(Debug)]
    struct ConfigExample {
        level1: Level1,
        level2: Level2,
    }

    #[derive(Default, Debug)]
    struct PartialConfigExample {
        level1: Option<Level1>,
        level2: Option<Level2>,
    }

    #[derive(Debug)]
    struct Level1 {
        n: u8,
        o: Option<isize>,
    }

    #[derive(Debug)]
    struct Level2 {
        p: i32,
        calc_p: i64,
    }

    impl FromTomlTable<PartialConfigExample, ()> for PartialConfigExample {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialConfigExample,
        ) -> Result<(), ()> {
            partial.level1 =
                Level1::from_toml_table(&mut helper.require_table("level1")?, partial).ok();
            partial.level2 =
                Level2::from_toml_table(&mut helper.require_table("level2")?, partial).ok();
            helper.deny_unknown()?;

            if partial.level1.is_some() && partial.level2.is_some() {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    impl FromTomlTable<PartialConfigExample, Level1> for Level1 {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialConfigExample,
        ) -> Result<Level1, ()>
        where
            Self: Sized,
        {
            let n = helper
                .get(&["n", "enn"][..])?
                .require()?
                .clamp(Some(1), Some(50));
            if let Ok(n) = n {
                let n2 = helper.get(&["n", "enn"][..])?.clamp(Some(1), Some(50))?;
                assert!(n2 == Some(n));
            }
            let o = helper.get("o")?.clamp(Some(-5), Some(55));
            Ok(Level1 { n: n?, o: o? })
        }
    }

    impl FromTomlTable<PartialConfigExample, Level2> for Level2 {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialConfigExample,
        ) -> Result<Level2, ()>
        where
            Self: Sized,
        {
            let p = helper.get("p")?.require()?.as_number()?;
            Ok(Level2 {
                p,
                calc_p: (partial.level1.as_ref().ok_or(())?.n as i64) + p as i64,
            })
        }
    }

    impl ToConcrete<ConfigExample> for PartialConfigExample {
        fn can_concrete(&self) -> bool {
            self.level1.is_some() && self.level2.is_some()
        }
        fn to_concrete(self) -> Option<ConfigExample> {
            Some(ConfigExample {
                level1: self.level1?,
                level2: self.level2?,
            })
        }
    }

    #[test]
    fn test_nested_happy() {
        let source = "
    [level1]
        n = 23
        o = 45
    [level2]
        p = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.level1.n, 23);
            assert_eq!(res.level1.o, Some(45));
            assert_eq!(res.level2.p, -23);
            assert_eq!(res.level2.calc_p, 0);
        }
    }
    #[test]
    fn test_nested_sub_missing() {
        let source = "
    [level1]
        n = 23
        o = 45
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Missing key: 'level2'");

                assert_eq!(err.inner.help, Some("".to_string()));
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_nested_happy_casing() {
        let source = "
    [LeVeL1]
        N = 23
        o = 45
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.level1.n, 23);
            assert_eq!(res.level1.o, Some(45));
            assert_eq!(res.level2.p, -23);
            assert_eq!(res.level2.calc_p, 0);
        }
    }
    #[test]
    fn test_nested_conflict() {
        let source = "
    [LeVeL1]
        N = 23
        n = 43
        o = 45
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);

        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 2);
                assert_eq!(err.inner.spans[0].msg, "Multiple keys (aliases) found");
                assert_eq!(err.inner.spans[1].msg, "Conflicts");

                assert_eq!(
                    err.inner.help,
                    Some(
                        "Please use only one of the keys, preferentially 'n' (canonical)"
                            .to_string()
                    )
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_nested_alias() {
        let source = "
    [LeVeL1]
        enn = 23
        o = 45
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert_eq!(res.level1.n, 23);
            assert_eq!(res.level1.o, Some(45));
            assert_eq!(res.level2.p, -23);
            assert_eq!(res.level2.calc_p, 0);
        }
    }

    #[test]
    fn test_nested_alias_clamp_to_large() {
        let source = "
    [LeVeL1]
        enn = 230
        o = 450
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        assert!(res.is_err());
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 2);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too large");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
                let err = &errs[1];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too large");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between -5 and 55".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_nested_alias_clamp_to_large_exact() {
        let source = "
    [LeVeL1]
        enn = 51
        o = 23
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        assert!(res.is_err());
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too large");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_nested_alias_clamp_to_small() {
        let source = "
    [LeVeL1]
        enn = 0 
        o = 1
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too low");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_nested_alias_clamp_to_small_just() {
        let source = "
    [LeVeL1]
        enn = 0 
        o = -6
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 2);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too low");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );

                let err = &errs[1];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value too low");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between -5 and 55".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_nested_alias_clamp_to_small_just_ok() {
        let source = "
    [LeVeL1]
        enn = 1 
        o = 5 
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        assert!(res.is_ok());
    }

    #[test]
    fn test_nested_alias_clamp_to_large_just_ok() {
        let source = "
    [LeVeL1]
        enn = 50 
        o = 55 
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        assert!(res.is_ok());
    }

    #[test]
    fn test_nested_alias_outside_of_range_lower() {
        let source = "
    [LeVeL1]
        enn = -1
        o = 45
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value outside u8 range");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_nested_alias_outside_of_range_upper() {
        let source = "
    [LeVeL1]
        enn = 256
        o = 45
    [levEl2]
        P = -23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Value outside u8 range");

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[derive(Debug, strum_macros::EnumString, strum_macros::VariantNames)]
    enum HelloEnum {
        SomeValue,
        OtherKind,
        OtherKind2,
        OtherKind3,
        SomeValueType2,
        SomeValueType3,
        SomeChum,
    }

    #[make_partial]
    #[derive(Debug)]
    struct Hello2 {
        hello: HelloEnum,
        hello2: Option<HelloEnum>,
        optional_number: Option<isize>,
    }

    impl FromTomlTable<PartialHello2, ()> for PartialHello2 {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialHello2,
        ) -> Result<(), ()> {
            partial.hello = Some(helper.get("hello")?.require()?.as_enum()?);
            partial.hello2 = Some(helper.get("hello2")?.as_enum()?);
            partial.optional_number = Some(helper.get("optional_number")?.as_number()?);
            helper.deny_unknown()?;

            Ok(())
        }
    }

    #[test]
    fn test_make_partial_and_enum_happy() {
        let source = "
        hello = 'SomeValue'
        hello2 = 'OtherKind'
        optional_number = 23
    ";
        let res = deserialize::<PartialHello2, Hello2>(source);
        dbg!(&res);
        assert!(res.is_ok());
        if let Ok(res) = res {
            assert!(matches!(res.hello, HelloEnum::SomeValue));
            assert!(matches!(res.hello2, Some(HelloEnum::OtherKind)));
            assert!(matches!(res.optional_number, Some(23)));
        }
    }
    #[test]
    fn test_make_partial_and_enum_wrong_value() {
        let source = "
        hello = 'SomeXalue'
    ";
        let res = deserialize::<PartialHello2, Hello2>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Invalid value");

                assert_eq!(
                    err.inner.help,
                    Some("Did you mean: 'SomeValue', 'SomeChum', or 'SomeValueType2'?".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_make_partial_and_enum_empty_value() {
        let source = "
        hello = ''
    ";
        let res = deserialize::<PartialHello2, Hello2>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Invalid value");

                assert_eq!(
                    err.inner.help,
                    Some("Available are: 'OtherKind', 'OtherKind2', 'OtherKind3', 'SomeChum', 'SomeValue', 'SomeValueType2', or 'SomeValueType3'".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_make_partial_and_enum_wrong_type() {
        let source = "
        hello = 123
    ";
        let res = deserialize::<PartialHello2, Hello2>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(
                    err.inner.spans[0].msg,
                    "Unexpected type: integer, expected: string"
                );

                assert_eq!(
                    err.inner.help,
                    Some("Available are: 'OtherKind', 'OtherKind2', 'OtherKind3', 'SomeChum', 'SomeValue', 'SomeValueType2', or 'SomeValueType3'".to_string())
                );
                println!("{}", err.pretty("input.toml"));
                assert_eq!(err.pretty("input.toml"), "  ╭─input.toml
  ┆

2 │         hello = 123
  ┆                 ─┬─
  ┆                  │ 
  ┆                  ╰── Unexpected type: integer, expected: string
──╯
Hint: Available are: 'OtherKind', 'OtherKind2', 'OtherKind3', 'SomeChum', 'SomeValue', 'SomeValueType2', or 'SomeValueType3'
");
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[test]
    fn test_pretty_finding_section() {
        let source = "
        #this should get ignored
        [level1]
        # this should be kept
        n = 'huh'

        [level2]
            p = 23
    ";
        let res = deserialize::<PartialConfigExample, ConfigExample>(source);
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(
                    err.inner.spans[0].msg,
                    "Unexpected type: string, expected: integer"
                );

                assert_eq!(
                    err.inner.help,
                    Some("Supply a value between 1 and 50".to_string())
                );
                println!("{}", err.pretty("input.toml"));
                assert_eq!(
                    err.pretty("input.toml"),
                    "  ╭─input.toml
  ┆
3 │         [level1]
4 │         # this should be kept
5 │         n = 'huh'
  ┆             ──┬──
  ┆               │  
  ┆               ╰─── Unexpected type: string, expected: integer
──╯
Hint: Supply a value between 1 and 50
"
                );
            }
            _ => panic!("Expected DeserFailure"),
        }
    }

    #[make_partial]
    #[derive(Debug)]
    struct ConfigExampleForDeny {
        level: Level1,
        other: Level2,
        different: String,
    }

    impl FromTomlTable<PartialConfigExampleForDeny, ()> for PartialConfigExampleForDeny {
        fn from_toml_table(
            helper: &mut TomlHelper<'_>,
            partial: &mut PartialConfigExampleForDeny,
        ) -> Result<(), ()> {
            helper.get("level");
            helper.get("other");
            helper.get("different");
            helper.deny_unknown()?;
            Ok(())
        }
    }

    #[test]
    fn test_deny_unknown() {
        let res = deserialize::<PartialConfigExampleForDeny, ConfigExampleForDeny>("smother = 23");
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Unknown key: smother");

                assert_eq!(
                    err.inner.help,
                    Some("Did you mean: 'other', 'level', or 'different'?".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        };
    }
    #[test]
    fn test_deny_unknown_used_existing() {
        let res = deserialize::<PartialConfigExampleForDeny, ConfigExampleForDeny>(
            "smother = 23\ndifferent='shu'",
        );
        dbg!(&res);
        match res {
            Err(DeserError::DeserFailure(errs, partial)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.inner.spans.len(), 1);
                assert_eq!(err.inner.spans[0].msg, "Unknown key: smother");

                assert_eq!(
                    err.inner.help,
                    Some("Did you mean: 'other' or 'level'?".to_string())
                );
            }
            _ => panic!("Expected DeserFailure"),
        };
    }
}
