use serde::de::Visitor;

use crate::{
    r#enum::EnumParser, r#struct::StructParser, Error, FromStrParser, MapParser, Options,
    PermissiveBoolParser, SeqParser,
};

#[derive(Clone, Copy)]
pub struct TomlParser;

impl SeqParser for TomlParser {
    fn parse_seq<'de, V, B, N, M, T, E>(
        &self,
        _options: Options<B, N, Self, M, T, E>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(toml::Deserializer::new(value), visitor)
            .map_err(serde::de::Error::custom)
    }
}

impl MapParser for TomlParser {
    fn parse_map<'de, V, B, N, S, T, E>(
        &self,
        _options: Options<B, N, S, Self, T, E>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(toml::de::Deserializer::new(value), visitor)
            .map_err(serde::de::Error::custom)
    }
}

impl StructParser for TomlParser {
    fn parse_struct<'de, V, B, N, S, M, E>(
        &self,
        _options: Options<B, N, S, M, Self, E>,
        value: &str,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_struct(
            toml::de::Deserializer::new(value),
            name,
            fields,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl EnumParser for TomlParser {
    fn parse_enum<'de, V, B, N, S, M, T>(
        &self,
        _options: Options<B, N, S, M, T, Self>,
        value: &str,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_enum(
            toml::de::Deserializer::new(value),
            name,
            variants,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Options<PermissiveBoolParser, FromStrParser, TomlParser, TomlParser, TomlParser, TomlParser> {
    pub const fn toml() -> Self {
        Options {
            bool_parser: PermissiveBoolParser,
            num_parser: FromStrParser,
            seq_parser: TomlParser,
            map_parser: TomlParser,
            struct_parser: TomlParser,
            enum_parser: TomlParser,
            ident_upper: true,
            bytes_base64: true,
        }
    }
}
