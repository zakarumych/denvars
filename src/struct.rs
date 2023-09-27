use serde::de::Visitor;

use crate::{BoolParser, Error, MapParser, NumParser, Options, PassthroughParser, SeqParser};

pub trait StructParser: Copy {
    fn parse_struct<'de, V, B, N, S, M>(
        &self,
        options: Options<B, N, S, M, Self>,
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
        M: MapParser;
}

impl StructParser for PassthroughParser {
    #[inline]
    fn parse_struct<'de, V, B, N, S, M>(
        &self,
        _options: Options<B, N, S, M, Self>,
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
