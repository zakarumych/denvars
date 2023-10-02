use base64::Engine;
use serde::de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};

use crate::{
    parser::Parser,
    unescape::{unescape, unescaped},
    Error, ValueDeserializer,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct BasicParser;

macro_rules! impl_num_from_str {
    ($($parse:ident $type:ident $visit:ident)*) => {
        $(
            #[inline]
            fn $parse<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                let with_err = || {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(value),
                        &"a potentially escaped string with number",
                    )
                };

                let value = unescaped(value.trim()).map_err(|_| with_err())?;
                let value = value
                    .parse::<$type>()
                    .map_err(|_| with_err())?;
                visitor.$visit(value)
            }
        )*
    };
}

impl Parser for BasicParser {
    #[inline]
    fn parse_bool<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let with_err = || {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(value),
                &"a potentially escaped string with boolean",
            )
        };

        let value = unescaped(value.trim()).map_err(|_| with_err())?;

        match &*value.to_lowercase() {
            "true" | "1" | "+" | "y" | "yea" | "yes" | "yeah" | "yah" | "aye" => {
                visitor.visit_bool(true)
            }
            "false" | "0" | "-" | "n" | "nay" | "no" | "nah" => visitor.visit_bool(false),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&*value),
                &"one of: true, false, 1, 0, +, -, y, n, ye, ya, yea, yeah, yah, aye, nay, no, nah",
            )),
        }
    }

    impl_num_from_str! {
        parse_i8 i8 visit_i8
        parse_i16 i16 visit_i16
        parse_i32 i32 visit_i32
        parse_i64 i64 visit_i64
        parse_i128 i128 visit_i128
        parse_u8 u8 visit_u8
        parse_u16 u16 visit_u16
        parse_u32 u32 visit_u32
        parse_u64 u64 visit_u64
        parse_u128 u128 visit_u128
        parse_f32 f32 visit_f32
        parse_f64 f64 visit_f64
    }

    fn parse_seq<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(BasicSeqAccess { value })
    }

    fn parse_map<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(BasicMapAccess { value })
    }

    fn parse_struct<'de, V>(
        self,
        value: &str,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(BasicMapAccess { value })
    }

    fn parse_enum<'de, V>(
        self,
        value: &str,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(BasicEnumAccess { value })
    }

    fn parse_bytes<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let with_err = || {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(value),
                &"a potentially escaped string with base64 sequence",
            )
        };
        let value = unescaped(value.trim()).map_err(|_| with_err())?;

        let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
            .decode(&*value)
            .map_err(|_| with_err())?;
        visitor.visit_byte_buf(decoded)
    }

    fn parse_any<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let with_err = || {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(value),
                &"a potentially escaped string value",
            )
        };

        let value = unescaped(value.trim()).map_err(|_| with_err())?;
        visitor.visit_str(&*value)
    }
}

struct BasicSeqAccess<'a> {
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

impl<'de, 'a> SeqAccess<'de> for BasicSeqAccess<'a> {
    type Error = Error;

    fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, Error>
    where
        U: serde::de::DeserializeSeed<'de>,
    {
        self.value = self.value.trim_start();
        if self.value.is_empty() {
            return Ok(None);
        }
        match self.value.strip_prefix('"') {
            None => match self.value.split_once(',') {
                None => {
                    let value = core::mem::take(&mut self.value).trim_end();
                    seed.deserialize(ValueDeserializer {
                        value,
                        parser: BasicParser,
                    })
                    .map(Some)
                }
                Some((head, tail)) => {
                    self.value = tail;
                    seed.deserialize(ValueDeserializer {
                        value: head.trim_end(),
                        parser: BasicParser,
                    })
                    .map(Some)
                }
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    parser: BasicParser,
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

struct BasicMapAccess<'a> {
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

impl<'de, 'a> MapAccess<'de> for BasicMapAccess<'a> {
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
                    let value = core::mem::take(&mut self.value).trim_end();
                    seed.deserialize(ValueDeserializer {
                        value,
                        parser: BasicParser,
                    })
                    .map(Some)
                }
                Some((head, tail)) => {
                    self.value = tail;
                    seed.deserialize(ValueDeserializer {
                        value: head.trim_end(),
                        parser: BasicParser,
                    })
                    .map(Some)
                }
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_colon_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    parser: BasicParser,
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
                    let value = core::mem::take(&mut self.value).trim_end();
                    seed.deserialize(ValueDeserializer {
                        value,
                        parser: BasicParser,
                    })
                }
                Some((head, tail)) => {
                    self.value = tail;
                    seed.deserialize(ValueDeserializer {
                        value: head.trim_end(),
                        parser: BasicParser,
                    })
                }
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_colon_separated_seq(self.value))?;

                let next = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    parser: BasicParser,
                })?;

                match tail {
                    None => Err(invalid_comma_colon_separated_seq(self.value)),
                    Some(tail) => {
                        let tail = tail.trim_start();
                        match tail.strip_prefix(',') {
                            None => Err(invalid_comma_colon_separated_seq(self.value)),
                            Some(tail) => {
                                self.value = tail;
                                Ok(next)
                            }
                        }
                    }
                }
            }
        }
    }
}

struct BasicEnumAccess<'a> {
    value: &'a str,
}

impl<'de, 'a> EnumAccess<'de> for BasicEnumAccess<'a> {
    type Error = Error;
    type Variant = BasicVariantAccess<'a>;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.value = self.value.trim_start();
        if self.value.is_empty() {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(self.value),
                &"a potentially escaped key:value pair",
            ));
        }
        match self.value.strip_prefix('"') {
            None => match self.value.split_once(':') {
                None => Ok((
                    seed.deserialize(ValueDeserializer {
                        value: self.value.trim_end(),
                        parser: BasicParser,
                    })?,
                    BasicVariantAccess { value: "" },
                )),
                Some((head, tail)) => Ok((
                    seed.deserialize(ValueDeserializer {
                        value: head.trim_end(),
                        parser: BasicParser,
                    })?,
                    BasicVariantAccess { value: tail },
                )),
            },
            Some(escaped) => {
                let (unescaped, tail) =
                    unescape(escaped).map_err(|_| invalid_comma_colon_separated_seq(self.value))?;

                let variant = seed.deserialize(ValueDeserializer {
                    value: &unescaped,
                    parser: BasicParser,
                })?;

                match tail {
                    None => Err(invalid_comma_colon_separated_seq(self.value)),
                    Some(tail) => {
                        let tail = tail.trim_start();
                        match tail.strip_prefix(':') {
                            None => Err(invalid_comma_colon_separated_seq(self.value)),
                            Some(tail) => Ok((variant, BasicVariantAccess { value: tail })),
                        }
                    }
                }
            }
        }
    }
}

struct BasicVariantAccess<'a> {
    value: &'a str,
}

impl<'de, 'a> VariantAccess<'de> for BasicVariantAccess<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(ValueDeserializer {
            value: self.value,
            parser: BasicParser,
        })
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        BasicParser.parse_seq(self.value, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(BasicMapAccess { value: self.value })
    }
}
