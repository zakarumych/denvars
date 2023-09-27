use serde::de::Visitor;

use crate::{
    r#struct::StructParser, Error, FromStrParser, MapParser, Options, PermissiveBoolParser,
    SeqParser,
};

#[derive(Clone, Copy)]
pub struct JsonParser;

impl SeqParser for JsonParser {
    fn parse_seq<'de, V, B, N, M, T>(
        &self,
        _options: Options<B, N, Self, M, T>,
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
    fn parse_map<'de, V, B, N, S, T>(
        &self,
        _options: Options<B, N, S, Self, T>,
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
    fn parse_struct<'de, V, B, N, S, M>(
        &self,
        _options: Options<B, N, S, M, Self>,
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

impl Options<PermissiveBoolParser, FromStrParser, JsonParser, JsonParser, JsonParser> {
    pub const fn json() -> Self {
        Options {
            bool_parser: PermissiveBoolParser,
            num_parser: FromStrParser,
            seq_parser: JsonParser,
            map_parser: JsonParser,
            struct_parser: JsonParser,
            ident_upper: true,
            bytes_base64: true,
        }
    }
}
