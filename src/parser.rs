use serde::de::Visitor;

use crate::Error;

pub trait Parser: Copy {
    fn parse_bool<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i8<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i16<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i32<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i64<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i128<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u8<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u16<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u32<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u64<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u128<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_f32<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_f64<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_seq<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_map<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_enum<'de, V>(
        self,
        value: &str,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_struct<'de, V>(
        self,
        value: &str,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_bytes<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_any<'de, V>(self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;
}
