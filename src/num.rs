use serde::de::Visitor;

use crate::{Error, FromStrParser, PassthroughParser};

pub trait NumParser: Copy {
    fn parse_i8<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i16<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_i128<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u8<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u16<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_u128<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_f32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;

    fn parse_f64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>;
}

impl NumParser for PassthroughParser {
    fn parse_i8<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_i16<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_i32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_i64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_i128<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_u8<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_u16<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_u32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_u64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_u128<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_f32<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }

    fn parse_f64<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(value)
    }
}

macro_rules! impl_parse_from_str {
    ($($parse:ident $type:ident $visit:ident)*) => {
        $(
            #[inline]
            fn $parse<'de, V>(&self, value: &str, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                value
                    .parse::<$type>()
                    .map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str(value),
                            &"a floating point number",
                        )
                    })
                    .and_then(|value| visitor.$visit(value))
            }
        )*
    };
}

impl NumParser for FromStrParser {
    impl_parse_from_str! {
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
}
