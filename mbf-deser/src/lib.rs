#![allow(unused)]
//! # MBF Deserialization Utilities
//!
//! This crate provides utilities for deserialization and error collection
//! focused on producing user-friendly error messages.

use std::{cell::RefCell, collections::HashSet, default, fmt::Display, ops::Range, rc::Rc};

use num_traits::{Bounded, FromPrimitive, NumCast, ToPrimitive};
use toml_edit::{Document, TomlError};

#[derive(Debug)]
pub enum DeserError {
    ParsingFailure(TomlError),
    DeserFailure(Vec<AnnotatedError>),
}

pub fn deserialize<S, T>(source: &str) -> Result<T, DeserError>
where
    S: FromTomlTable<S, ()> + ToConcrete<T> + Default,
{
    let parsed_toml = source.parse::<Document<String>>()?;
    let source = Rc::new(RefCell::new(source.to_string()));

    let mut helper = TomlHelper::new(parsed_toml.as_table(), source.clone());

    let mut partial = S::default();
    match S::from_toml_table(&mut helper, &mut partial) {
        Ok(_) => {}
        Err(()) => {
            return Err(DeserError::DeserFailure(helper.into_inner()));
        }
    };
    if let Err(()) = helper.deny_unknown() {
        return Err(DeserError::DeserFailure(helper.into_inner()));
    };

    Ok(partial.to_concrete())
}

pub struct TomlHelper<'a> {
    table: &'a toml_edit::Table,
    allowed: Vec<String>,
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    source: Rc<RefCell<String>>,
}

fn join_strings<T: AsRef<str>>(slice: &[T], separator: &str) -> String {
    let mut result = String::new();
    for (i, s) in slice.iter().enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        result.push_str(s.as_ref());
    }
    result
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
            Some(item) => register_type_error(self.errors, item, "string"),
            None => Ok(None),
        }
    }

    fn as_number<T>(self) -> Result<Option<T>, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
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
                            &format!(
                                "Supply a value between {} and {}",
                                T::min_value(),
                                T::max_value()
                            ),
                        ));
                        Err(())
                    }
                }
            }
            Some(item) => register_type_error(self.errors, item, "integer"),
            None => Ok(None),
        }
    }
}

fn register_type_error<T>(
    errors: Rc<RefCell<Vec<Rc<RefCell<AnnotatedError>>>>>,
    item: &toml_edit::Item,
    expected: &str,
) -> Result<T, ()> {
    let span = item.span().unwrap_or(0..0);
    errors.borrow_mut().push(AnnotatedError::placed(
        span,
        &format!(
            "Unexpected type: {}, expected: {expected}",
            item.type_name()
        ),
        "",
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
            item => register_type_error(self.errors, item, "string"),
        }
    }

    fn as_number<T>(self) -> Result<T, ()>
    where
        T: NumCast + Bounded + Display + FromPrimitive,
    {
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
                            &format!(
                                "Supply a value between {} and {}",
                                T::min_value(),
                                T::max_value()
                            ),
                        ));
                        Err(())
                    }
                }
            }
            item => register_type_error(self.errors, item, "integer"),
        }
    }
}

impl<'a> TomlHelper<'a> {
    fn new(table: &'a toml_edit::Table, source: Rc<RefCell<String>>) -> TomlHelper<'a> {
        TomlHelper {
            table,
            allowed: vec![],
            errors: Rc::new(RefCell::new(vec![])),
            source,
        }
    }

    fn into_inner(self) -> Vec<AnnotatedError> {
        self.errors
            .borrow_mut()
            .drain(..)
            .map(|wrapped| Rc::into_inner(wrapped).expect("RC error").into_inner())
            .collect()
    }

    pub fn require_table(&mut self, table_key: &str) -> Result<TomlHelper<'a>, ()> {
        let item = self.table.get(table_key);
        self.allowed.push(table_key.to_string());
        match item {
            Some(toml_edit::Item::Table(table)) => Ok(TomlHelper {
                table,
                allowed: vec![],
                errors: self.errors.clone(),
                source: self.source.clone(),
            }),
            Some(item) => register_type_error(self.errors.clone(), item, "table")?,
            None => {
                self.add_missing_key(table_key, "");
                Err(())
            }
        }
    }

    pub fn get(&mut self, key: &str) -> OptDeserResultItem<'a> {
        self.allowed.push(key.to_string());
        let item = self.table.get(key);
        match item {
            Some(item) => OptDeserResultItem::ok(
                item,
                key,
                key,
                self.table.span().unwrap_or(0..0),
                self.errors.clone(),
            ),
            None => {
                //self.add_missing_key(key, help);
                OptDeserResultItem::not_found(
                    key,
                    self.table.span().unwrap_or(0..0),
                    self.errors.clone(),
                )
            }
        }
    }

    fn suggest_alternatives<T: AsRef<str>>(_current: &str, available: &[T]) -> String {
        format!("Available are are: {}", join_strings(available, ", "))
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

    fn add_err_by_item(&self, item: &toml_edit::Item, msg: &str, help: &str) {
        let span = item.span().unwrap_or(0..0);
        self.errors
            .borrow_mut()
            .push(AnnotatedError::placed(span, msg, help));
    }

    fn add_err_by_span(&self, span: Range<usize>, msg: &str, help: &str) {
        self.errors
            .borrow_mut()
            .push(AnnotatedError::placed(span, msg, help));
    }

    fn add_missing_key(&self, key: &str, help: &str) {
        let span = self.table.span().unwrap_or(0..0);
        self.errors.borrow_mut().push(AnnotatedError::placed(
            span,
            &format!("Missing key '{}'", key),
            help,
        ));
    }

    fn deny_unknown(&self) -> Result<(), ()> {
        let mut had_unknown = false;
        let mut seen = HashSet::new();
        for (key, _) in self.table.iter() {
            println!("Lookin at {}", key);
            seen.insert(key);
            if !self.allowed.iter().any(|x| *x == key) {
                println!("{key} was not in allowed");
                let still_available: Vec<_> = self
                    .allowed
                    .iter()
                    .filter(|s| !seen.contains(&s[..]))
                    .collect();
                self.add_err_by_key(
                    key,
                    &format!("Unknown key: {key}"),
                    &TomlHelper::suggest_alternatives(key, &still_available),
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

impl From<TomlError> for DeserError {
    fn from(value: TomlError) -> Self {
        DeserError::ParsingFailure(value)
    }
}

pub trait ToConcrete<T> {
    fn to_concrete(self) -> T;
}

#[derive(Debug)]
struct SpannedMessage {
    span: Range<usize>,
    msg: String,
}

#[derive(Debug)]
pub struct AnnotatedError {
    source: Rc<RefCell<String>>,
    spans: Vec<SpannedMessage>,
    help: Option<String>,
}

impl AnnotatedError {
    fn unplaced(source: Rc<RefCell<String>>, help: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(AnnotatedError {
            source,
            spans: vec![],
            help: Some(help.to_string()),
        }))
    }

    fn placed(span: Range<usize>, msg: &str, help: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(AnnotatedError {
            source: Rc::new(RefCell::new(String::new())),
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
}

impl AnnotatedErrorExt for Rc<RefCell<AnnotatedError>> {
    fn get_out(self) -> AnnotatedError {
        Rc::into_inner(self).unwrap().into_inner()
    }
}

#[cfg(test)]
mod test {
    use std::default;

    use crate::{DeserError, FromTomlTable, ToConcrete, TomlHelper};

    use super::deserialize;

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
            let world: &str = hello.get("world").require()?.as_str()?;
            hello.deny_unknown()?;

            partial.hello = Some(Hello {
                world: world.to_string(),
            });

            Ok(())
        }
    }

    impl ToConcrete<HelloWorld> for PartialHelloWorld {
        fn to_concrete(self) -> HelloWorld {
            HelloWorld {
                hello: self.hello.unwrap(),
            }
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
            Err(DeserError::DeserFailure(errs)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.spans.len(), 1);
                assert_eq!(err.spans[0].msg, "Missing key: 'world'");
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
            Err(DeserError::DeserFailure(errs)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.spans.len(), 1);
                assert_eq!(
                    err.spans[0].msg,
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
            Err(DeserError::DeserFailure(errs)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.spans.len(), 1);
                assert_eq!(
                    err.spans[0].msg,
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
        dbg!(&res);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.spans.len(), 1);
                assert_eq!(err.spans[0].msg, "Unknown key: hallo");
            }
            _ => panic!("Expected DeserFailure"),
        }
    }
    #[test]
    fn test_additional_inner() {
        let source = "hello.world = 'hi'\nhello.shu=123";
        let res = deserialize::<PartialHelloWorld, HelloWorld>(source);
        dbg!(&res);
        assert!(res.is_err());
        match res {
            Err(DeserError::DeserFailure(errs)) => {
                assert_eq!(errs.len(), 1);
                let err = &errs[0];
                assert_eq!(err.spans.len(), 1);
                assert_eq!(err.spans[0].msg, "Unknown key: shu");
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
        n: usize,
        o: Option<usize>,
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
            let n = helper.get("n").require()?.as_number()?;
            let o = helper.get("o").as_number()?;
            Ok(Level1 { n: n, o: o })
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
            let p = helper.get("p").require()?.as_number()?;
            Ok(Level2 {
                p,
                calc_p: (partial.level1.as_ref().expect("level1 decoded first").n as i64)
                    + p as i64,
            })
        }
    }

    impl ToConcrete<ConfigExample> for PartialConfigExample {
        fn to_concrete(self) -> ConfigExample {
            ConfigExample {
                level1: self.level1.unwrap(),
                level2: self.level2.unwrap(),
            }
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
}
