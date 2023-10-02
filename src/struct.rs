use serde::de::Visitor;

use crate::{
    r#enum::EnumParser, BoolParser, Error, MapParser, NumParser, Options, PassthroughParser,
    SeqParser,
};

pub trait StructParser: Copy {
    fn parse_struct<'de, V, B, N, S, M, E>(
        &self,
        options: Options<B, N, S, M, Self, E>,
        value: &str,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        S: SeqParser,
        M: MapParser,
        E: EnumParser;
}

impl StructParser for PassthroughParser {
    #[inline]
    fn parse_struct<'de, V, B, N, S, M, E>(
        &self,
        _options: Options<B, N, S, M, Self, E>,
        value: &str,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }
}
