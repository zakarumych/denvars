use serde::de::{SeqAccess, Visitor};

use crate::{
    bool::BoolParser, map::MapParser, num::NumParser, r#struct::StructParser, unescape::unescape,
    Error, Options, PassthroughParser, ValueDeserializer,
};

pub trait SeqParser: Copy {
    fn parse_seq<'de, V, B, N, M, T>(
        &self,
        options: Options<B, N, Self, M, T>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        M: MapParser,
        T: StructParser;
}

impl SeqParser for PassthroughParser {
    fn parse_seq<'de, V, B, N, M, T>(
        &self,
        _options: Options<B, N, Self, M, T>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }
}

#[derive(Clone, Copy)]
pub struct CommaSeparatedParser;

impl SeqParser for CommaSeparatedParser {
    fn parse_seq<'de, V, B, N, M, T>(
        &self,
        options: Options<B, N, Self, M, T>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        M: MapParser,
        T: StructParser,
    {
        visitor.visit_seq(CommaSeparatedParserSeqAccess { options, value })
    }
}

struct CommaSeparatedParserSeqAccess<'a, B, N, M, T> {
    options: Options<B, N, CommaSeparatedParser, M, T>,
    value: &'a str,
}

fn invalid_comma_separated_seq<E>(s: &str) -> E
where
    E: serde::de::Error,
{
    serde::de::Error::invalid_value(
        serde::de::Unexpected::Str(s),
        &"a potentially escaped strings delimited by comma",
    )
}

impl<'de, 'a, B, N, M, T> SeqAccess<'de> for CommaSeparatedParserSeqAccess<'a, B, N, M, T>
where
    B: BoolParser,
    N: NumParser,
    M: MapParser,
    T: StructParser,
{
    type Error = Error;

    fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, Error>
    where
        U: serde::de::DeserializeSeed<'de>,
    {
        if self.value.is_empty() {
            return Ok(None);
        }
        self.value = self.value.trim();
        match self.value.strip_prefix('"') {
            None => match self.value.split_once(',') {
                None => {
                    let value = core::mem::take(&mut self.value);
                    seed.deserialize(ValueDeserializer {
                        value,
                        options: self.options,
                    })
                    .map(Some)
                }
                Some((head, tail)) => {
                    self.value = tail;
                    seed.deserialize(ValueDeserializer {
                        value: head,
                        options: self.options,
                    })
                    .map(Some)
                }
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    options: self.options,
                })?;

                match tail {
                    None => return Err(invalid_comma_separated_seq(self.value)),
                    Some(tail) => {
                        let tail = tail.trim_start();
                        if tail.is_empty() {
                            self.value = tail;
                            return Ok(Some(next));
                        }
                        match tail.strip_prefix(',') {
                            None => return Err(invalid_comma_separated_seq(self.value)),
                            Some(tail) => {
                                self.value = tail;
                            }
                        }
                    }
                }

                Ok(Some(next))
            }
        }
    }
}
