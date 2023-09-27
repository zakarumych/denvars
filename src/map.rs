use serde::de::{MapAccess, Visitor};

use crate::{
    bool::BoolParser, num::NumParser, r#struct::StructParser, seq::SeqParser, unescape::unescape,
    Error, Options, PassthroughParser, ValueDeserializer,
};

pub trait MapParser: Copy {
    fn parse_map<'de, V, B, N, S, T>(
        &self,
        options: Options<B, N, S, Self, T>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        S: SeqParser,
        T: StructParser;
}

impl MapParser for PassthroughParser {
    #[inline]
    fn parse_map<'de, V, B, N, S, T>(
        &self,
        _options: Options<B, N, S, Self, T>,
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
pub struct CommaColonSeparatedParser;

impl MapParser for CommaColonSeparatedParser {
    fn parse_map<'de, V, B, N, S, T>(
        &self,
        options: Options<B, N, S, Self, T>,
        value: &str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        S: SeqParser,
        T: StructParser,
    {
        visitor.visit_map(CommaColonSeparatedParserMapAccess { options, value })
    }
}

impl StructParser for CommaColonSeparatedParser {
    fn parse_struct<'de, V, B, N, S, M>(
        &self,
        options: Options<B, N, S, M, Self>,
        value: &str,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
        B: BoolParser,
        N: NumParser,
        S: SeqParser,
        M: MapParser,
    {
        visitor.visit_map(CommaColonSeparatedParserMapAccess { options, value })
    }
}

struct CommaColonSeparatedParserMapAccess<'a, B, N, S, M, T> {
    options: Options<B, N, S, M, T>,
    value: &'a str,
}

fn invalid_comma_colon_separated_seq<E>(s: &str) -> E
where
    E: serde::de::Error,
{
    serde::de::Error::invalid_value(
        serde::de::Unexpected::Str(s),
        &"a potentially escaped key:value pairs delimited by comma",
    )
}

impl<'de, 'a, B, N, S, M, T> MapAccess<'de>
    for CommaColonSeparatedParserMapAccess<'a, B, N, S, M, T>
where
    B: BoolParser,
    N: NumParser,
    S: SeqParser,
    M: MapParser,
    T: StructParser,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        self.value = self.value.trim_start();
        if self.value.is_empty() {
            return Ok(None);
        }
        match self.value.strip_prefix('"') {
            None => match self.value.split_once(':') {
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
                    unescape(escaped).map_err(|_| invalid_comma_colon_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    options: self.options,
                })?;

                match tail {
                    None => return Err(invalid_comma_colon_separated_seq(self.value)),
                    Some(tail) => {
                        let tail = tail.trim_start();
                        match tail.strip_prefix(':') {
                            None => return Err(invalid_comma_colon_separated_seq(self.value)),
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

    fn next_value_seed<U>(&mut self, seed: U) -> Result<U::Value, Error>
    where
        U: serde::de::DeserializeSeed<'de>,
    {
        self.value = self.value.trim_start();
        match self.value.strip_prefix('"') {
            None => match self.value.split_once(',') {
                None => {
                    let value = core::mem::take(&mut self.value);
                    seed.deserialize(ValueDeserializer {
                        value,
                        options: self.options,
                    })
                }
                Some((head, tail)) => {
                    self.value = tail;
                    seed.deserialize(ValueDeserializer {
                        value: head,
                        options: self.options,
                    })
                }
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_colon_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    options: self.options,
                })?;

                match tail {
                    None => return Err(invalid_comma_colon_separated_seq(self.value)),
                    Some(tail) => {
                        let tail = tail.trim_start();
                        match tail.strip_prefix(',') {
                            None => return Err(invalid_comma_colon_separated_seq(self.value)),
                            Some(tail) => {
                                self.value = tail;
                            }
                        }
                    }
                }

                Ok(next)
            }
        }
    }
}
