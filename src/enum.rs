use serde::de::Visitor;

use crate::{BoolParser, Error, MapParser, NumParser, Options, PassthroughParser, SeqParser};

pub trait EnumParser: Copy {
    fn parse_enum<'de, V, B, N, S, M, T>(
        &self,
        options: Options<B, N, S, M, T, Self>,
        value: &str,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        S: SeqParser,
        M: MapParser;
}

impl EnumParser for PassthroughParser {
    #[inline]
    fn parse_enum<'de, V, B, N, S, M, T>(
        &self,
        _options: Options<B, N, S, M, T, Self>,
        value: &str,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }
}
