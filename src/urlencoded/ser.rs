//! Serialization support for the `application/x-www-form-urlencoded` format.

mod key;
mod pair;
mod part;
mod value;

use std::{borrow::Cow, str};

use serde::ser;
use url::form_urlencoded::{
    Serializer as UrlEncodedSerializer, Target as UrlEncodedTarget,
};

use crate::urlencoded::error::{Error, Result};

/// Serializes a value into a `application/x-www-form-urlencoded` `String` buffer.
///
/// ```
/// let meal = &[
///     ("bread", "baguette"),
///     ("cheese", "comté"),
///     ("meat", "ham"),
///     ("fat", "butter"),
/// ];
///
/// assert_eq!(
///     ruma_serde::urlencoded::to_string(meal),
///     Ok("bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter".to_owned()));
/// ```
pub fn to_string<T: ser::Serialize>(input: T) -> Result<String> {
    let mut urlencoder = UrlEncodedSerializer::new("".to_owned());
    input.serialize(Serializer::new(&mut urlencoder))?;
    Ok(urlencoder.finish())
}

/// A serializer for the `application/x-www-form-urlencoded` format.
///
/// * Supported top-level inputs are structs, maps and sequences of pairs,
///   with or without a given length.
///
/// * Supported keys and values are integers, bytes (if convertible to strings),
///   unit structs and unit variants.
///
/// * Newtype structs defer to their inner values.
pub struct Serializer<'input, 'output, Target: 'output + UrlEncodedTarget> {
    urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
}

impl<'input, 'output, Target: 'output + UrlEncodedTarget>
    Serializer<'input, 'output, Target>
{
    /// Returns a new `Serializer`.
    pub fn new(
        urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
    ) -> Self {
        Serializer { urlencoder }
    }
}

/// Sequence serializer.
pub struct SeqSerializer<'input, 'output, Target: 'output + UrlEncodedTarget> {
    urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
    key: Option<Cow<'input, str>>,
    count: usize,
}

/// Tuple serializer.
///
/// Mostly used for arrays.
pub struct TupleSerializer<'input, 'output, Target: 'output + UrlEncodedTarget>
{
    urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
}

/// Tuple struct serializer.
///
/// Never instantiated, tuple structs are not supported.
pub struct TupleStructSerializer<'input, 'output, T: 'output + UrlEncodedTarget>
{
    inner: ser::Impossible<&'output mut UrlEncodedSerializer<'input, T>, Error>,
}

/// Tuple variant serializer.
///
/// Never instantiated, tuple variants are not supported.
pub struct TupleVariantSerializer<
    'input,
    'output,
    T: 'output + UrlEncodedTarget,
> {
    inner: ser::Impossible<&'output mut UrlEncodedSerializer<'input, T>, Error>,
}

/// Map serializer.
pub struct MapSerializer<'input, 'output, Target: 'output + UrlEncodedTarget> {
    urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
    key: Option<Cow<'static, str>>,
}

/// Struct serializer.
pub struct StructSerializer<'input, 'output, Target: 'output + UrlEncodedTarget>
{
    urlencoder: &'output mut UrlEncodedSerializer<'input, Target>,
}

/// Struct variant serializer.
///
/// Never instantiated, struct variants are not supported.
pub struct StructVariantSerializer<
    'input,
    'output,
    T: 'output + UrlEncodedTarget,
> {
    inner: ser::Impossible<&'output mut UrlEncodedSerializer<'input, T>, Error>,
}

impl<'input, 'output, Target> ser::Serializer
    for Serializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;
    type SerializeSeq = SeqSerializer<'input, 'output, Target>;
    type SerializeTuple = TupleSerializer<'input, 'output, Target>;
    type SerializeTupleStruct = TupleStructSerializer<'input, 'output, Target>;
    type SerializeTupleVariant =
        TupleVariantSerializer<'input, 'output, Target>;
    type SerializeMap = MapSerializer<'input, 'output, Target>;
    type SerializeStruct = StructSerializer<'input, 'output, Target>;
    type SerializeStructVariant =
        StructVariantSerializer<'input, 'output, Target>;

    /// Returns an error.
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_str(self, _value: &str) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns `Ok`.
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }

    /// Returns `Ok`.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }

    /// Returns an error.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Serializes the inner value, ignoring the newtype name.
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    /// Returns an error.
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::top_level())
    }

    /// Returns `Ok`.
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }

    /// Serializes the given value.
    fn serialize_some<T: ?Sized + ser::Serialize>(
        self,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    /// Serialize a sequence, given length (if any) is ignored.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            urlencoder: self.urlencoder,
            key: None,
            count: 0,
        })
    }

    /// Returns an error.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleSerializer {
            urlencoder: self.urlencoder,
        })
    }

    /// Returns an error.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::top_level())
    }

    /// Serializes a map, given length is ignored.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            urlencoder: self.urlencoder,
            key: None,
        })
    }

    /// Serializes a struct, given length is ignored.
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Ok(StructSerializer {
            urlencoder: self.urlencoder,
        })
    }

    /// Returns an error.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::top_level())
    }
}

impl<'input, 'output, Target> ser::SerializeSeq
    for SeqSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        value.serialize(pair::PairSerializer::new(
            self.urlencoder,
            self.key.as_ref(),
            &mut self.count,
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }
}

impl<'input, 'output, Target> ser::SerializeTuple
    for TupleSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        value.serialize(pair::PairSerializer::new(
            self.urlencoder,
            None,
            &mut 0,
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }
}

impl<'input, 'output, Target> ser::SerializeTupleStruct
    for TupleStructSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end()
    }
}

impl<'input, 'output, Target> ser::SerializeTupleVariant
    for TupleVariantSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end()
    }
}

impl<'input, 'output, Target> ser::SerializeMap
    for MapSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_entry<
        K: ?Sized + ser::Serialize,
        V: ?Sized + ser::Serialize,
    >(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<()> {
        let key_sink = key::KeySink::new(|key| {
            let value_sink = value::ValueSink::new(self.urlencoder, &key);
            value.serialize(part::PartSerializer::new(value_sink))?;
            self.key = None;
            Ok(())
        });
        let entry_serializer = part::PartSerializer::new(key_sink);
        key.serialize(entry_serializer)
    }

    fn serialize_key<T: ?Sized + ser::Serialize>(
        &mut self,
        key: &T,
    ) -> Result<()> {
        let key_sink = key::KeySink::new(|key| Ok(key.into()));
        let key_serializer = part::PartSerializer::new(key_sink);
        self.key = Some(key.serialize(key_serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        {
            let key = self.key.as_ref().ok_or_else(Error::no_key)?;
            let value_sink = value::ValueSink::new(self.urlencoder, &key);
            value.serialize(part::PartSerializer::new(value_sink))?;
        }
        self.key = None;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }
}

impl<'input, 'output, Target> ser::SerializeStruct
    for StructSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        let key = Cow::Borrowed(key);
        let mut count = 0;
        let value_sink =
            pair::PairSerializer::new(self.urlencoder, Some(&key), &mut count);
        value.serialize(value_sink)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.urlencoder)
    }
}

impl<'input, 'output, Target> ser::SerializeStructVariant
    for StructVariantSerializer<'input, 'output, Target>
where
    Target: 'output + UrlEncodedTarget,
{
    type Ok = &'output mut UrlEncodedSerializer<'input, Target>;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end()
    }
}
