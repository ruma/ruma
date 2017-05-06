//! Deserialization support for the `application/x-www-form-urlencoded` format.

use serde::de;
use serde::de::value::{MapDeserializer, ValueDeserializer as SerdeValueDeserializer};

#[doc(inline)]
pub use serde::de::value::Error;
use std::borrow::Cow;
use std::io::Read;
use std::iter::Map;
use std::marker::PhantomData;
use url::form_urlencoded::Parse as UrlEncodedParse;
use url::form_urlencoded::parse;

/// Deserializes a `application/x-wwww-url-encoded` value from a `&[u8]`.
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
///     serde_urlencoded::from_bytes::<Vec<(String, String)>>(
///         b"bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_bytes<T: de::Deserialize>(input: &[u8]) -> Result<T, Error> {
    T::deserialize(Deserializer::new(parse(input)))
}

/// Deserializes a `application/x-wwww-url-encoded` value from a `&str`.
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
///     serde_urlencoded::from_str::<Vec<(String, String)>>(
///         "bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter"),
///     Ok(meal));
/// ```
pub fn from_str<T: de::Deserialize>(input: &str) -> Result<T, Error> {
    from_bytes(input.as_bytes())
}

/// Convenience function that reads all bytes from `reader` and deserializes
/// them with `from_bytes`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T, Error>
    where T: de::Deserialize,
          R: Read,
{
    let mut buf = vec![];
    reader.read_to_end(&mut buf)
        .map_err(|e| {
            de::Error::custom(format_args!("could not read input: {}", e))
        })?;
    from_bytes(&buf)
}

/// A deserializer for the `application/x-www-form-urlencoded` format.
///
/// * Supported top-level outputs are structs, maps and sequences of pairs,
///   with or without a given length.
///
/// * Main `deserialize` methods defers to `deserialize_map`.
///
/// * Everything else but `deserialize_seq` and `deserialize_seq_fixed_size`
///   defers to `deserialize`.
pub struct Deserializer<'a> {
    inner: MapDeserializer<Map<UrlEncodedParse<'a>,
                               fn((Cow<'a, str>, Cow<'a, str>))
                                  -> (Cow<'a, str>, Value<'a>)>,
                           Error>,
}

impl<'a> Deserializer<'a> {
    /// Returns a new `Deserializer`.
    pub fn new(parser: UrlEncodedParse<'a>) -> Self {
        Deserializer {
            inner: MapDeserializer::new(parser.map(Value::wrap_pair)),
        }
    }
}

impl<'a> de::Deserializer for Deserializer<'a> {
    type Error = Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_map(self.inner)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(self.inner)
    }

    fn deserialize_seq_fixed_size<V>(self,
                                     _len: usize,
                                     visitor: V)
                                     -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_seq(self.inner)
    }

    forward_to_deserialize! {
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
        unit
        option
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        struct
        struct_field
        tuple
        enum
        ignored_any
    }
}

struct Value<'a>(Cow<'a, str>);

impl<'a> Value<'a> {
    fn wrap_pair((k, v): (Cow<'a, str>, Cow<'a, str>)) -> (Cow<'a, str>, Self) {
        (k, Value(v))
    }
}

impl<'a, E> SerdeValueDeserializer<E> for Value<'a>
    where E: de::Error,
{
    type Deserializer = ValueDeserializer<'a, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self.0,
            marker: PhantomData,
        }
    }
}

struct ValueDeserializer<'a, E> {
    value: Cow<'a, str>,
    marker: PhantomData<E>,
}

impl<'a, E> de::Deserializer for ValueDeserializer<'a, E>
    where E: de::Error,
{
    type Error = E;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        self.value.into_deserializer().deserialize(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor,
    {
        visitor.visit_some(self.value.into_deserializer())
    }

    forward_to_deserialize! {
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
        unit
        bytes
        byte_buf
        unit_struct
        newtype_struct
        tuple_struct
        struct
        struct_field
        tuple
        enum
        ignored_any
        seq
        seq_fixed_size
        map
    }
}
