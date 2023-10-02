use serde::de::Visitor;

use crate::{parser::Parser, Error, Options};

#[derive(Clone, Copy, Debug, Default)]
pub struct TomlParser;

macro_rules! impl_parse {
    ($($parse:ident $deserialize:ident)*) => {
        $(
            #[inline]
            fn $parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                serde::de::Deserializer::$deserialize(
                    toml::Deserializer::new(value),
                    visitor,
                )
                .map_err(serde::de::Error::custom)
            }
        )*
    };
}

impl Parser for TomlParser {
    impl_parse! {
        parse_bool deserialize_bool
        parse_i8 deserialize_i8
        parse_i16 deserialize_i16
        parse_i32 deserialize_i32
        parse_i64 deserialize_i64
        parse_i128 deserialize_i128
        parse_u8 deserialize_u8
        parse_u16 deserialize_u16
        parse_u32 deserialize_u32
        parse_u64 deserialize_u64
        parse_u128 deserialize_u128
        parse_f32 deserialize_f32
        parse_f64 deserialize_f64
        parse_seq deserialize_seq
        parse_map deserialize_map
        parse_bytes deserialize_bytes
        parse_any deserialize_any
    }

    fn parse_enum<'de, V>(
        self,
        value: &str,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_enum(
            toml::Deserializer::new(value),
            name,
            variants,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }

    fn parse_struct<'de, V>(
        self,
        value: &str,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_struct(
            toml::Deserializer::new(value),
            name,
            fields,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Options<TomlParser> {
    pub const fn toml() -> Self {
        Options {
            parser: TomlParser,
            ident_upper: true,
        }
    }
}
