use crate::config::{SegmentIndex, SegmentIndexOrAll};
use crate::dna;
use crate::transformations::{ResolvedSourceAll, ResolvedSourceNoAll};
use bstr::{BStr, BString};
use serde::{Deserialize, Deserializer, de};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;
use std::{fmt, marker::PhantomData};

use toml_edit::{Item, Value};

/// Collect all concurrent configuration errors
pub struct ErrorCollectorInner {
    errors: Vec<Rc<RefCell<ConfigError>>>,
    segment_order: Option<Vec<String>>,
}

impl ErrorCollectorInner {
    pub fn set_segment_order(&mut self, segments_in_order: Vec<String>) {
        self.segment_order = Some(segments_in_order);
    }

    pub fn get_segment_order(&self) -> &Vec<String> {
        self.segment_order
            .as_ref()
            .expect("Called before segments were parsed from configuration")
    }
}

pub type ErrorCollector = Rc<RefCell<ErrorCollectorInner>>;

pub fn new_error_collector() -> ErrorCollector {
    Rc::new(RefCell::new(ErrorCollectorInner {
        errors: Vec::new(),
        segment_order: None,
    }))
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
            error_count_at_start: self.borrow().errors.len(),
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
        self.as_ref().borrow_mut().errors.push(e.clone());
        Err(e)
    }

    fn add_table<T>(&self, table: &toml_edit::Table, msg: &str, help: &str) -> TomlResult<T> {
        let span = table.span().map(|span| {
            let start = span.start;
            let mut end = span.end;
            for (_k, v) in table.iter() {
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
        self.as_ref().borrow_mut().errors.push(e.clone());
        Err(e)
    }

    fn add_value<T>(&self, value: &toml_edit::Value, msg: &str, help: &str) -> TomlResult<T> {
        let e = Rc::new(RefCell::new(ConfigError::new(msg, help, value.span())));
        self.as_ref().borrow_mut().errors.push(e.clone());
        Err(e)
    }

    fn add_key<T>(&self, key: &toml_edit::Key) -> TomlResult<T> {
        let e = Rc::new(RefCell::new(ConfigError::new(
            "Invalid key",
            "Check documentation",
            key.span(),
        )));
        self.as_ref().borrow_mut().errors.push(e.clone());
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
        for err in self.borrow().errors.iter() {
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
    pub errors: ErrorCollector,
    unknown_handled: bool,
    error_count_at_start: usize,
}

impl std::fmt::Debug for TableErrorHelper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableErrorHelper")
            .field("table", &self.table)
            .field("allowed", &self.allowed)
            .finish()
    }
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

pub trait KeyOrAlias<'b> {
    fn get<'a>(&self, table: &'a toml_edit::Table) -> Option<(&'b str, &'a toml_edit::Item)>;
    fn display(&self) -> &str;
}

impl<'b> KeyOrAlias<'b> for &'b str {
    fn get<'a>(&self, table: &'a toml_edit::Table) -> Option<(&'b str, &'a toml_edit::Item)> {
        table.get(self).map(|value| (*self, value)).or_else(|| {
            let key = self.to_lowercase();
            for (table_key, value) in table.iter() {
                if table_key.to_lowercase() == key {
                    return Some((*self, value));
                }
            }
            None
        })
    }
    fn display(&self) -> &str {
        self
    }
}
impl<'b> KeyOrAlias<'b> for &'b [&str] {
    fn get<'a>(&self, table: &'a toml_edit::Table) -> Option<(&'b str, &'a toml_edit::Item)> {
        for possible_key in *self {
            if let Some(res) = KeyOrAlias::get(possible_key, table) {
                return Some((possible_key, res.1));
            }
        }
        None
    }

    fn display(&self) -> &str {
        self[0]
    }
}

impl<'a> TableErrorHelper<'a> {
    pub fn add_table<T>(&self, table: &toml_edit::Table, msg: &str, help: &str) -> TomlResult<T> {
        self.errors.add_table(table, msg, help)
    }

    pub fn add_err<T>(&self, err: Rc<RefCell<ConfigError>>) -> TomlResult<T> {
        self.errors.borrow_mut().errors.push(err.clone());
        Err(err)
    }
    pub fn add_err_by_key<T>(&self, key: &str, msg: &str, help: &str) -> TomlResult<T> {
        match self.table.get_key_value(key) {
            Some((_tkey, value)) => self.errors.add_item(value, msg, help),
            None => self.errors.add_table(&self.table, msg, help),
        }
    }

    pub fn local(&self, table: &'a toml_edit::Table) -> TableErrorHelper<'a> {
        TableErrorHelper {
            table,
            allowed: self.allowed.clone(),
            errors: self.errors.clone(),
            unknown_handled: false,
            error_count_at_start: self.errors.borrow().errors.len(),
        }
    }

    pub fn get<'b, T>(&mut self, key: impl KeyOrAlias<'b>) -> TomlResult<T>
    where
        T: FromToml,
    {
        //self.allowed.push(key.to_string());
        match key.get(self.table) {
            Some((matched_key, value)) => {
                self.allowed.push(matched_key.to_string());
                Ok(T::from_toml(value, &self.errors)?)
            }
            None => {
                self.errors
                    .add_table(self.table, &format!("Missing key: {}", key.display()), "")
            }
        }
    }

    pub fn get_alias<'b, T>(&mut self, key: impl KeyOrAlias<'b>) -> TomlResult<(&'b str, T)>
    where
        T: FromToml,
    {
        //self.allowed.push(key.to_string());
        match key.get(self.table) {
            Some((matched_key, value)) => {
                self.allowed.push(matched_key.to_string());
                Ok((matched_key, T::from_toml(value, &self.errors)?))
            }
            None => {
                self.errors
                    .add_table(self.table, &format!("Missing key: {}", key.display()), "")
            }
        }
    }
    pub fn get_opt_alias<'b, T>(
        &mut self,
        key: impl KeyOrAlias<'b>,
    ) -> TomlResult<Option<(&'b str, T)>>
    where
        T: FromToml,
    {
        //self.allowed.push(key.to_string());
        match key.get(self.table) {
            Some((matched_key, value)) => {
                self.allowed.push(matched_key.to_string());
                Ok(Some((matched_key, T::from_toml(value, &self.errors)?)))
            }
            None => Ok(None),
        }
    }

    pub fn get_tag<'b>(&mut self, key: impl KeyOrAlias<'b>) -> TomlResult<String> {
        let (matched_key, value) = key.get(self.table).ok_or_else(|| {
            self.errors
                .add_table::<String>(self.table, &format!("Missing key: {}", key.display()), "")
                .unwrap_err()
        })?;
        self.allowed.push(matched_key.to_string());
        if let toml_edit::Item::Value(toml_edit::Value::String(str_value)) = value {
            if let Err(e) = super::validate_tag_name(str_value.value()) {
                self.add_err_by_key(matched_key, "Invalid tag name.", &e.to_string())
            } else {
                Ok(str_value.value().to_string())
            }
        } else {
            self.add_err_by_key(matched_key, "Expected a string", "") //todo
        }
    }

    pub fn get_segment(&mut self, default_to_one_and_only: bool) -> TomlResult<SegmentIndex> {
        let res = self.get_opt::<String>("Segment")?;
        match res {
            None => {
                if default_to_one_and_only {
                    Ok(SegmentIndex(0))
                } else {
                    self.errors.add_table(
                        self.table,
                        "segment = '...' missing, but multiple segments in analysis",
                        &format!(
                            "Set segment to one of {:?}",
                            self.errors.borrow().get_segment_order()
                        ),
                    )
                }
            }
            Some(segment_name) => {
                if let Some(idx) = self
                    .errors
                    .borrow()
                    .get_segment_order()
                    .iter()
                    .position(|sn| **sn == segment_name)
                {
                    Ok(SegmentIndex(idx))
                } else {
                    self.errors.add_table(
                        self.table,
                        "Unknown segment.",
                        &format!(
                            "Set segment to one of {:?}",
                            self.errors.borrow().get_segment_order()
                        ),
                    )
                }
            }
        }
    }

    pub fn get_segment_all(
        &mut self,
        default_to_one_and_only: bool,
    ) -> TomlResult<SegmentIndexOrAll> {
        let res = self.get_opt::<String>("Segment")?;
        match res {
            None => {
                if default_to_one_and_only {
                    Ok(SegmentIndexOrAll::Indexed(0))
                } else {
                    self.errors.add_table(
                        self.table,
                        "segment = '...' missing, but multiple segments in analysis",
                        &format!(
                            "Set segment to one of {:?}",
                            self.errors.borrow().get_segment_order()
                        ),
                    )
                }
            }
            Some(segment_name) => {
                if segment_name == "All" {
                    Ok(SegmentIndexOrAll::All)
                } else {
                    if let Some(idx) = self
                        .errors
                        .borrow()
                        .get_segment_order()
                        .iter()
                        .position(|sn| **sn == segment_name)
                    {
                        Ok(SegmentIndexOrAll::Indexed(idx))
                    } else {
                        self.errors.add_table(
                            self.table,
                            "Unknown segment.",
                            &format!(
                                "Set segment to one of {:?}",
                                self.errors.borrow().get_segment_order()
                            ),
                        )
                    }
                }
            }
        }
    }

    pub fn get_source_no_all<'b>(
        &mut self,
        key: impl KeyOrAlias<'b>,
        default_to_one_and_only: bool,
    ) -> TomlResult<(String, ResolvedSourceNoAll)> {
        let res = key.get(self.table);
        let (matched_key, value) = match res {
            None => ("", None),
            Some((matched_key, toml_edit::Item::Value(toml_edit::Value::String(s)))) => {
                self.allowed.push(matched_key.to_string());
                (matched_key, Some(s.value()))
            }
            Some((matched_key, _)) => {
                return self.add_err_by_key(
                    matched_key,
                    "Expected astring value.",
                    "Try passing a 'segment', a 'name:<segment>', or 'tag:<tagname>'.", //todo
                );
            }
        };

        match value {
            None => {
                if default_to_one_and_only {
                    Ok((
                        self.errors
                            .borrow()
                            .get_segment_order()
                            .iter()
                            .next()
                            .expect("Must have one segment defined")
                            .to_string(),
                        ResolvedSourceNoAll::Segment(SegmentIndex(0)),
                    ))
                } else {
                    self.errors.add_table(
                        self.table,
                        &format!(
                            "{} missing, but multiple segments in analysis",
                            key.display()
                        ),
                        &format!(
                            "Set to a segment ({:?}), a name:<segment> or a tag:<tag-name>",
                            self.errors.borrow().get_segment_order() //todo: list tags?
                        ),
                    )
                }
            }
            Some(source) => {
                let source = source.trim();
                let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
                    let trimmed = tag_name.trim();
                    if trimmed.is_empty() {
                        return self.add_err_by_key(
                            matched_key,
                            "Source/target tag:<name> may not have an empty name.",
                            "", //todo: available tags
                        );
                    }
                    ResolvedSourceNoAll::Tag(trimmed.to_string())
                } else if let Some(segment_name) = source.strip_prefix("name:") {
                    let trimmed = segment_name.trim();
                    if trimmed.is_empty() {
                        return self.add_err_by_key(
                            matched_key,
                            "Source/target name:<segment> may not have an empty name.",
                            "", //todo: available tags
                        );
                    }
                    let segment_index = if segment_name.to_lowercase() == "all" {
                        return self.add_err_by_key(
                            matched_key,
                            "'All' is not a valid value here.",
                            "", //todo: available segments
                        );
                    } else if let Some(idx) = self
                        .errors
                        .borrow()
                        .get_segment_order()
                        .iter()
                        .position(|sn| *sn == segment_name)
                    {
                        SegmentIndex(idx)
                    } else {
                        return self.add_err_by_key(
                            matched_key,
                            "Segment unknown",
                            "", //todo: available segments
                        );
                    };
                    ResolvedSourceNoAll::Name {
                        segment_index,
                        split_character: b'@', //TODO //input_def.options.read_comment_character,
                    }
                } else {
                    let segment_name = source;
                    let segment_idx_or_all = if segment_name.to_lowercase() == "all" {
                        return self.add_err_by_key(
                            matched_key,
                            "'All' is not a valid value here.",
                            "", //todo: available segments
                        );
                    } else if let Some(idx) = self
                        .errors
                        .borrow()
                        .get_segment_order()
                        .iter()
                        .position(|sn| *sn == segment_name)
                    {
                        SegmentIndex(idx)
                    } else {
                        return self.add_err_by_key(
                            matched_key,
                            "Segment unknown",
                            "", //todo: available segments
                        );
                    };
                    ResolvedSourceNoAll::Segment(segment_idx_or_all)
                };
                Ok((source.to_string(), resolved))
            }
        }
    }
    pub fn get_source_all<'b>(
        &mut self,
        key: impl KeyOrAlias<'b>,
        default_to_one_and_only: bool,
    ) -> TomlResult<(String, ResolvedSourceAll)> {
        let res = key.get(self.table);
        let (matched_key, value) = match res {
            None => ("", None),
            Some((matched_key, toml_edit::Item::Value(toml_edit::Value::String(s)))) => {
                self.allowed.push(matched_key.to_string());
                (matched_key, Some(s.value()))
            }
            Some((matched_key, _)) => {
                return self.add_err_by_key(
                    matched_key,
                    "Expected astring value.",
                    "Try passing a 'segment', a 'name:<segment>', or 'tag:<tagname>'.", //todo
                );
            }
        };

        match value {
            None => {
                if default_to_one_and_only {
                    Ok((
                        self.errors
                            .borrow()
                            .get_segment_order()
                            .iter()
                            .next()
                            .expect("Must have one segment defined")
                            .to_string(),
                        ResolvedSourceAll::Segment(SegmentIndexOrAll::Indexed(0)),
                    ))
                } else {
                    self.errors.add_table(
                        self.table,
                        &format!(
                            "{} missing, but multiple segments in analysis",
                            key.display()
                        ),
                        &format!(
                            "Set to a segment ({:?}), a name:<segment> or a tag:<tag-name>",
                            self.errors.borrow().get_segment_order() //todo: list tags?
                        ),
                    )
                }
            }
            Some(source) => {
                let source = source.trim();
                let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
                    let trimmed = tag_name.trim();
                    if trimmed.is_empty() {
                        return self.add_err_by_key(
                            matched_key,
                            "Source/target tag:<name> may not have an empty name.",
                            "", //todo: available tags
                        );
                    }
                    ResolvedSourceAll::Tag(trimmed.to_string())
                } else if let Some(segment_name) = source.strip_prefix("name:") {
                    let trimmed = segment_name.trim();
                    if trimmed.is_empty() {
                        return self.add_err_by_key(
                            matched_key,
                            "Source/target name:<segment> may not have an empty name.",
                            "", //todo: available tags
                        );
                    }
                    let segment_index_or_all = if segment_name.to_lowercase() == "all" {
                        SegmentIndexOrAll::All
                    } else if let Some(idx) = self
                        .errors
                        .borrow()
                        .get_segment_order()
                        .iter()
                        .position(|sn| *sn == segment_name)
                    {
                        SegmentIndexOrAll::Indexed(idx)
                    } else {
                        return self.add_err_by_key(
                            matched_key,
                            "Segment unknown",
                            "", //todo: available segments
                        );
                    };
                    ResolvedSourceAll::Name {
                        segment_index_or_all,
                        split_character: b'@', //TODO //input_def.options.read_comment_character,
                    }
                } else {
                    let segment_name = source;
                    let segment_idx_or_all = if segment_name.to_lowercase() == "all" {
                        SegmentIndexOrAll::All
                    } else if let Some(idx) = self
                        .errors
                        .borrow()
                        .get_segment_order()
                        .iter()
                        .position(|sn| *sn == segment_name)
                    {
                        SegmentIndexOrAll::Indexed(idx)
                    } else {
                        return self.add_err_by_key(
                            matched_key,
                            "Segment unknown",
                            "", //todo: available segments
                        );
                    };
                    ResolvedSourceAll::Segment(segment_idx_or_all)
                };
                Ok((source.to_string(), resolved))
            }
        }
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
                    && x > *mm
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
                    && *mm >  x
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

    pub fn get_opt_u8_from_char_or_number<'b>(
        &mut self,
        key: impl KeyOrAlias<'b>,
        minimum: Option<u8>,
        maximum: Option<u8>,
    ) -> TomlResult<Option<(&'b str, u8)>> {
        match key.get(self.table) {
            Some((matched_key, item)) => {
                self.allowed.push(matched_key.to_string());
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
                            &format!("Expected value >= {mm}. Observed: {}", res),
                            &format!("Supply value in {})", format_range(minimum, maximum)),
                        );
                    }
                    if let Some(mm) = maximum
                        && res > mm
                    {
                        return self.errors.add_item(
                            item,
                            &format!("Expected value <= {mm}. Observed: {}", res),
                            &format!("Supply value in {})", format_range(minimum, maximum)),
                        );
                    }
                    Ok(Some((matched_key, res)))
                } else {
                    return self.errors.add_item(
                        item,
                        "Must be a single character string or an integer byte value.",
                        &format!("Supply value in {})", format_range(minimum, maximum)),
                    );
                }
            }
            None => Ok(None),
        }
    }

    pub fn accept_unknown(&mut self) -> TomlResult<()> {
        self.unknown_handled = true;
        self._check_for_new_errors()
    }

    fn _check_for_new_errors(&self) -> TomlResult<()> {
        if self.error_count_at_start != self.errors.borrow().errors.len() {
            Err(self
                .errors
                .borrow()
                .errors
                .iter()
                .last()
                .expect("error_occured but no errors present!?")
                .clone())
        } else {
            Ok(())
        }
    }

    pub fn deny_unknown(&mut self) -> TomlResult<()> {
        self.unknown_handled = true;
        for (key, _) in self.table.iter() {
            if !self.allowed.iter().any(|x| *x == key) {
                //get's turned into an error return below.
                self.errors
                    .add_key::<()>(self.table.key(key).expect("just iterated it"))
                    .ok();
            }
        }
        self._check_for_new_errors()
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ConfigError {
    help: String,
    spans: Vec<(std::ops::Range<usize>, String)>,
}

impl ConfigError {
    pub fn new(
        msg: impl ToString,
        help: impl ToString,
        span: Option<std::ops::Range<usize>>,
    ) -> Self {
        Self {
            help: help.to_string(),
            spans: if let Some(span) = span {
                vec![(span, msg.to_string())]
            } else {
                vec![]
            },
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

    pub fn render(&self, source: &str, source_name: &str) -> String {
        use bstr::ByteSlice;
        use codesnake::{Block, CodeWidth, Label, LineIndex};
        use std::fmt::Write;

        if !self.spans.is_empty() {
            let idx = LineIndex::new(source);
            let mut spans = self.spans.clone();
            spans.sort_by_key(|(span, _msg)| span.start);

            let previous_newline =
                memchr::memmem::rfind(&source.as_bytes()[..spans[0].0.start], b"\n");
            let mut labels = Vec::new();

            for (span, text) in spans.into_iter() {
                labels.push(Label::new(span).with_text(text));
            }
            let block = Block::new(&idx, labels).unwrap_or_else(||{
                let mut spans = self.spans.clone();
                spans.sort_by_key(|(span, _msg)| span.start);
                let span_str: Vec<_>  = spans.iter().map(|
                (span, name)| format!("{}..{}: {}", span.start, span.end, name)
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
            format!("ConfigError at unknown location.Help text: {}", &self.help)
        }
    }
}

pub type TomlResult<T> = Result<T, Rc<RefCell<ConfigError>>>;

pub trait TomlResultExt<T> {
    fn new_err(text: &str, help: &str, range: Option<std::ops::Range<usize>>) -> TomlResult<T>;
    fn add_help(self, text: &str) -> Self;
    fn add_span(&self, span: std::ops::Range<usize>, msg: &str) -> &Self;
    fn has_overlapping_span(&self, span: &std::ops::Range<usize>) -> bool;
}

impl<T> TomlResultExt<T> for TomlResult<T> {
    fn new_err(text: &str, help: &str, range: Option<std::ops::Range<usize>>) -> TomlResult<T> {
        return Err(Rc::new(RefCell::new(ConfigError::new(text, help, range))));
    }
    fn add_help(self, text: &str) -> Self {
        if let Err(ce) = &self {
            ce.borrow_mut().help = format!("{}\n{}", ce.borrow().help, text);
        }
        self
    }

    fn add_span(&self, span: std::ops::Range<usize>, msg: &str) -> &Self {
        if let Err(ce) = &self {
            ce.borrow_mut().spans.push((span, msg.to_string()));
        }
        &self
    }

    fn has_overlapping_span(&self, span: &std::ops::Range<usize>) -> bool {
        if let Err(ce) = &self {
            for other_span in ce.borrow_mut().spans.iter() {
                if span.start < other_span.0.end && other_span.0.start < span.end {
                    return true;
                }
            }
        }
        false
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
impl FromToml for isize {
    fn from_toml(item: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        match item {
            Item::Value(Value::Integer(s)) => {
                let value: Result<isize, _> = (*s.value()).try_into();
                match value {
                    Ok(v) => Ok(v),
                    Err(_) => collector.add_item(
                        item,
                        "Expected an isize, found value outside isize range",
                        "More than +-(2^64-1)?",
                    ),
                }
            }
            item => collector.add_item(
                item,
                &format!("Expected an isize, found {}", item.type_name()),
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
