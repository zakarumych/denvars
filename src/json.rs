use serde::de::Visitor;

use crate::{
    r#enum::EnumParser, r#struct::StructParser, Error, FromStrParser, MapParser, Options,
    PermissiveBoolParser, SeqParser,
};

#[derive(Clone, Copy)]
pub struct JsonParser;

impl SeqParser for JsonParser {
    fn parse_seq<'de, V, B, N, M, T, E>(
        &self,
        _options: Options<B, N, Self, M, T, E>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(
            &mut serde_json::de::Deserializer::from_reader(value.as_bytes()),
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl MapParser for JsonParser {
    fn parse_map<'de, V, B, N, S, T, E>(
        &self,
        _options: Options<B, N, S, Self, T, E>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(
            &mut serde_json::de::Deserializer::from_reader(value.as_bytes()),
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl StructParser for JsonParser {
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
            &mut serde_json::de::Deserializer::from_reader(value.as_bytes()),
            name,
            fields,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl EnumParser for JsonParser {
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
            &mut serde_json::de::Deserializer::from_reader(value.as_bytes()),
            name,
            variants,
            visitor,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Options<PermissiveBoolParser, FromStrParser, JsonParser, JsonParser, JsonParser, JsonParser> {
    pub const fn json() -> Self {
        Options {
            bool_parser: PermissiveBoolParser,
            num_parser: FromStrParser,
            seq_parser: JsonParser,
            map_parser: JsonParser,
            struct_parser: JsonParser,
            enum_parser: JsonParser,
            ident_upper: true,
            bytes_base64: true,
        }
    }
}
