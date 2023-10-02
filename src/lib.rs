//! This crate provides deserializer that reads from environment variables or user-provided array of key-value pairs.
//! For convenience, it can be configured to call specific visiting method
//! for different kind of data.
//! By default it parses booleans from large set of possible values,
//! numbers using `FromStr`,
//! sequences from comma-separated values,
//! maps from comma-separated key:value pairs,
//! allows using potentially escaped strings in double quotes,
//! decodes base64-encoded byte arrays if configured (this is default behavior),
//! compare uppercase names of fields when deserializing struct from map of env vars if configured (this is default behavior),
//! It may treat values as JSON to support deserializing nested structures.
//! Custom string parsers may be implemented to support other formats.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use core::fmt;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde::de::{self, Visitor};

pub use self::{basic::BasicParser, parser::Parser, unescape::unescape};

mod basic;
mod parser;
mod unescape;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "toml")]
pub mod toml;

#[derive(Debug)]
pub struct Error {
    custom: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.custom {
            Some(custom) => write!(f, "{}", custom),
            None => write!(f, "unknown error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error {
            custom: Some(msg.to_string()),
        }
    }
}

/// Parser that uses `FromStr` to parse values
/// into requested type.
#[derive(Clone, Copy)]
pub struct FromStrParser;

#[derive(Clone, Copy)]
pub struct Options<P> {
    /// Controls how values are parsed from env var values.
    parser: P,

    /// Controls whether to compare uppercase names of fields when
    /// deserializing struct from map of env vars.
    ident_upper: bool,
}

type DefaultOptions = Options<BasicParser>;

impl Options<BasicParser> {
    pub const fn basic() -> Self {
        Options {
            parser: BasicParser,
            ident_upper: true,
        }
    }
}

impl Default for Options<BasicParser> {
    fn default() -> Self {
        Self::basic()
    }
}

pub struct Deserializer<O = DefaultOptions> {
    vars: Vec<(String, String)>,
    options: O,
}

impl Deserializer {
    #[cfg(feature = "std")]
    pub fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> Self {
        Deserializer {
            vars: vars.into_iter().collect(),
            options: DefaultOptions::basic(),
        }
    }
}

impl Deserializer {
    #[cfg(feature = "std")]
    pub fn from_env_vars() -> Self {
        let vars = std::env::vars_os().filter_map(|(key, value)| {
            Some((key.to_str()?.to_owned(), value.to_str()?.to_owned()))
        });

        Deserializer::from_vars(vars)
    }

    #[cfg(feature = "std")]
    pub fn from_prefixed_env_vars(prefix: &str) -> Self {
        let vars = std::env::vars_os().filter_map(|(key, value)| {
            let key = key.to_str()?;
            if let Some(key_suffix) = key.strip_prefix(prefix) {
                Some((key_suffix.to_owned(), value.to_str()?.to_owned()))
            } else {
                None
            }
        });

        Deserializer::from_vars(vars)
    }
}

impl<O> Deserializer<O> {
    /// Set options of the deserializer.
    pub fn with_options<X>(self, options: X) -> Deserializer<X> {
        Deserializer {
            vars: self.vars,
            options,
        }
    }
}

impl<'de, P> de::Deserializer<'de> for Deserializer<Options<P>>
where
    P: Parser,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(Map {
            next_value: None,
            vars: self
                .vars
                .into_iter()
                .map(|(key, value)| (key, VarAccess::Value(value)))
                .collect(),
            options: self.options,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let ident_upper = self.options.ident_upper;
        let mut vars = Vec::<(String, VarAccess)>::new();

        for (key, value) in self.vars {
            let name_suffix = fields
                .iter()
                .filter_map(|name| {
                    let suffix = if ident_upper {
                        key.strip_prefix(&*name.to_uppercase())?
                    } else {
                        key.strip_prefix(*name)?
                    };
                    Some((&**name, suffix))
                })
                .max_by_key(|(name, _)| name.len());

            match name_suffix {
                None => match vars.iter().position(|(ident, _)| **ident == *key) {
                    Some(index) => {
                        vars[index].1 = VarAccess::Value(value);
                    }
                    None => {
                        vars.push((key, VarAccess::Value(value)));
                    }
                },
                Some((name, "")) => match vars.iter().position(|(ident, _)| **ident == *name) {
                    Some(index) => {
                        vars[index].1 = VarAccess::Value(value);
                    }
                    None => {
                        vars.push((name.to_owned(), VarAccess::Value(value)));
                    }
                },
                Some((name, suffix)) => {
                    if let Some(suffix) = suffix.strip_prefix('_') {
                        match vars.iter().position(|(ident, _)| **ident == *name) {
                            Some(index) => match &mut vars[index].1 {
                                VarAccess::Vars(map) => {
                                    match map.iter().position(|(ident, _)| **ident == *suffix) {
                                        Some(index) => {
                                            map[index].1 = value;
                                        }
                                        None => {
                                            map.push((suffix.to_owned(), value));
                                        }
                                    }
                                }
                                VarAccess::Value(_) => {}
                            },
                            None => {
                                vars.push((
                                    name.to_owned(),
                                    VarAccess::Vars(vec![(suffix.to_owned(), value)]),
                                ));
                            }
                        }
                    }
                }
            }
        }

        visitor.visit_map(Map {
            next_value: None,
            vars,
            options: self.options,
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any
    }
}

enum VarAccess {
    Value(String),
    Vars(Vec<(String, String)>),
}

struct Map<O> {
    next_value: Option<VarAccess>,
    vars: Vec<(String, VarAccess)>,
    options: O,
}

impl<'de, P> de::MapAccess<'de> for Map<Options<P>>
where
    P: Parser,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.vars.pop() {
            Some((key, var)) => {
                let key = seed
                    .deserialize(de::value::StringDeserializer::new(key))
                    .map(Some)?;
                self.next_value = Some(var);
                Ok(key)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.next_value.take() {
            Some(VarAccess::Value(value)) => seed.deserialize(ValueDeserializer {
                value: &value,
                parser: self.options.parser,
            }),
            Some(VarAccess::Vars(vars)) => seed.deserialize(Deserializer {
                vars,
                options: self.options,
            }),
            None => panic!("next_value called before next_key"),
        }
    }
}

struct ValueDeserializer<'a, P> {
    value: &'a str,
    parser: P,
}

macro_rules! parse_num {
    ($($deserialize:ident $type:ident $parse:ident)*) => {$(
        fn $deserialize<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            self.parser.$parse(&self.value, visitor)
        }
    )*};
}

impl<'de, P> de::Deserializer<'de> for ValueDeserializer<'_, P>
where
    P: Parser,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_bool(&self.value, visitor)
    }

    parse_num! {
        deserialize_i8 i8 parse_i8
        deserialize_i16 i16 parse_i16
        deserialize_i32 i32 parse_i32
        deserialize_i64 i64 parse_i64
        deserialize_i128 i128 parse_i128
        deserialize_u8 u8 parse_u8
        deserialize_u16 u16 parse_u16
        deserialize_u32 u32 parse_u32
        deserialize_u64 u64 parse_u64
        deserialize_u128 u128 parse_u128
        deserialize_f32 f32 parse_f32
        deserialize_f64 f64 parse_f64
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .chars()
            .next()
            .map(|c| visitor.visit_char(c))
            .unwrap_or_else(|| Err(de::Error::custom("empty string")))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if let Some(escaped) = self.value.strip_prefix('"') {
            let (unescaped, tail) = unescape(escaped).map_err(|_| {
                de::Error::invalid_value(
                    de::Unexpected::Str(self.value),
                    &"Potentially escaped string",
                )
            })?;

            match tail {
                None => {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Str(self.value),
                        &"Escaped string with closing quote",
                    ));
                }
                Some(tail) => {
                    let tail = tail.trim_start();
                    if !tail.is_empty() {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(tail),
                            &"Potentially escaped string without characters after closing quote",
                        ));
                    }
                }
            }
            visitor.visit_string(unescaped)
        } else {
            visitor.visit_str(&self.value)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_bytes(self.value, visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_seq(&self.value, visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_map(&self.value, visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_struct(&self.value, name, fields, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.parser.parse_enum(&self.value, name, variants, visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
