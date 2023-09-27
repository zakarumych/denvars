use serde::de::Visitor;

use crate::{Error, FromStrParser, PassthroughParser};

pub trait BoolParser: Copy {
    fn parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;
}

impl BoolParser for PassthroughParser {
    fn parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }
}

impl BoolParser for FromStrParser {
    fn parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(value.parse().map_err(serde::de::Error::custom)?)
    }
}

/// Boolean parser that accepts many forms of `true` and `false`.
///
/// `true` values are `"true"`, `"1"`, `"+"`, `"y"`, `"yea"`, `"yes"`, `"yeah"`, `"yah"`, `"aye"`
/// `false` values are `"false"`, `"0"`, `"-"`, `"n"`, `"nay"`, `"no"`, `"nah"`
#[derive(Clone, Copy)]
pub struct PermissiveBoolParser;

impl BoolParser for PermissiveBoolParser {
    #[inline]
    fn parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match &*value.to_lowercase() {
            "true" | "1" | "+" | "y" | "yea" | "yes" | "yeah" | "yah" | "aye" => {
                visitor.visit_bool(true)
            }
            "false" | "0" | "-" | "n" | "nay" | "no" | "nah" => visitor.visit_bool(false),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(value),
                &"one of: true, false, 1, 0, +, -, y, n, ye, ya, yea, yeah, yah, aye, nay, no, nah",
            )),
        }
    }
}
