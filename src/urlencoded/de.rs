//! Deserialization support for the `application/x-www-form-urlencoded` format.

use std::{borrow::Cow, io::Read};

use serde::de::{
    self,
    value::{MapDeserializer, SeqDeserializer},
    Error as de_Error, IntoDeserializer,
};
use url::form_urlencoded::{parse, Parse as UrlEncodedParse};

#[doc(inline)]
pub use serde::de::value::Error;

/// Deserializes a `application/x-www-form-urlencoded` value from a `&[u8]`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     ruma_serde::urlencoded::from_bytes::<Vec<(String, String)>>(
///         b"bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    T::deserialize(Deserializer::new(parse(input)))
}

/// Deserializes a `application/x-www-form-urlencoded` value from a `&str`.
///
/// ```
/// let meal = vec![
///     ("bread".to_owned(), "baguette".to_owned()),
///     ("cheese".to_owned(), "comté".to_owned()),
///     ("meat".to_owned(), "ham".to_owned()),
///     ("fat".to_owned(), "butter".to_owned()),
/// ];
///
/// assert_eq!(
///     ruma_serde::urlencoded::from_str::<Vec<(String, String)>>(
///         "bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_str<'de, T>(input: &'de str) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T, Error>
where
    T: de::DeserializeOwned,
    R: Read,
{
    let mut buf = vec![];
    reader.read_to_end(&mut buf).map_err(|e| {
        de::Error::custom(format_args!("could not read input: {}", e))
    })?;
    from_bytes(&buf)
}

/// A deserializer for the Matrix query string format.
///
/// * Supported top-level outputs are structs, maps and sequences,
///   with or without a given length.
///
/// * Main `deserialize` methods defers to `deserialize_map`.
///
/// * Everything else but `deserialize_seq` and `deserialize_seq_fixed_size`
///   defers to `deserialize`.
pub struct Deserializer<'de> {
    parser: UrlEncodedParse<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn new(parser: UrlEncodedParse<'de>) -> Self {
        Self { parser }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(PartIterator(self.parser)))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(MapDeserializer::new(PartIterator(self.parser)))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let pairs = self.parser.collect::<Vec<_>>();
        let mut map = std::collections::VecDeque::new();
        for field in fields {
            let values = pairs
                .iter()
                .filter(|(f, _)| f == field)
                .map(|(_f, v)| v.clone())
                .collect::<Vec<_>>();
            map.push_back((*field, values));
        }
        let parts = fields
            .iter()
            .map(|f| Part(Cow::Borrowed(f), None))
            .zip(PartIterator(self.parser).map(|(_, mut v)| {
                if let Some((_, val)) = map.pop_front() {
                    v.1 = Some(val);
                }
                v
            }))
            .map(|(field, value)| (field, value));
        visitor.visit_map(MapDeserializer::new(parts))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        MapDeserializer::new(PartIterator(self.parser)).end()?;
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bool
        u8
        u16
        u32
        u64
        i8
        i16
        i32
        i64
        f32
        f64
        char
        str
        string
        option
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        identifier
        tuple
        enum
        ignored_any
    }
}

struct PartIterator<'de>(UrlEncodedParse<'de>);

impl<'de> Iterator for PartIterator<'de> {
    type Item = (Part<'de>, Part<'de>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (Part(k, None), Part(v, None)))
    }
}

struct Part<'de>(Cow<'de, str>, Option<Vec<Cow<'de, str>>>);

impl<'de> IntoDeserializer<'de> for Part<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

macro_rules! forward_parsed_value {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor<'de>
            {
                match self.0.parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        )*
    }
}

impl<'de> de::Deserializer<'de> for Part<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Cow::Borrowed(value) => visitor.visit_borrowed_str(value),
            Cow::Owned(value) => visitor.visit_string(value),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(ValueEnumAccess(self.0))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let iter = self.1.ok_or_else(|| Error::custom("expected sequence"))?;
        visitor.visit_seq(SeqDeserializer::new(
            iter.into_iter().map(|v| Part(v, None)),
        ))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let pairs = self
            .1
            .ok_or_else(|| Error::custom("percent decoding may have failed"))?;

        let raw_json = pairs
            .get(0)
            .ok_or_else(|| Error::custom("no value found for nested struct"))?;

        let mut de =
            serde_json::de::Deserializer::from_reader(raw_json.as_bytes());
        de.deserialize_struct(name, fields, visitor)
            .map_err(|e| Error::custom(e.to_string()))
    }

    serde::forward_to_deserialize_any! {
        char
        str
        string
        unit
        bytes
        byte_buf
        unit_struct
        tuple_struct
        identifier
        tuple
        ignored_any
        map
    }

    forward_parsed_value! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }
}

struct ValueEnumAccess<'de>(Cow<'de, str>);

impl<'de> de::EnumAccess<'de> for ValueEnumAccess<'de> {
    type Error = Error;
    type Variant = UnitOnlyVariantAccess;

    fn variant_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.0.into_deserializer())?;
        Ok((variant, UnitOnlyVariantAccess))
    }
}

struct UnitOnlyVariantAccess;

impl<'de> de::VariantAccess<'de> for UnitOnlyVariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::custom("expected unit variant"))
    }

    fn tuple_variant<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("expected unit variant"))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::custom("expected unit variant"))
    }
}
