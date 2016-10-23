//! Serialization support for the `application/x-www-form-urlencoded` format.

mod key;
mod pair;
mod value;

use serde::ser;
use std::borrow::Cow;
use std::error;
use std::fmt;
use std::str;
use url::form_urlencoded::Serializer as UrlEncodedSerializer;
use url::form_urlencoded::Target as UrlEncodedTarget;

/// Serializes a value into a `application/x-wwww-url-encoded` `String` buffer.
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
///     serde_urlencoded::to_string(meal),
///     Ok("bread=baguette&cheese=comt%C3%A9&meat=ham&fat=butter".to_owned()));
/// ```
pub fn to_string<T: ser::Serialize>(input: &T) -> Result<String, Error> {
    let mut output = String::new();
    {
        let mut urlencoder = UrlEncodedSerializer::new(&mut output);
        try!(input.serialize(&mut Serializer::new(&mut urlencoder)));
    }
    Ok(output)
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
pub struct Serializer<'output, T: 'output + UrlEncodedTarget> {
    urlencoder: &'output mut UrlEncodedSerializer<T>
}

impl<'output, T: 'output + UrlEncodedTarget> Serializer<'output, T> {
    /// Returns a new `Serializer`.
    pub fn new(urlencoder: &'output mut UrlEncodedSerializer<T>) -> Self {
        Serializer { urlencoder: urlencoder }
    }
}

/// Errors returned during serializing to `application/x-www-form-urlencoded`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Custom(Cow<'static, str>),
    InvalidValue(Cow<'static, str>),
    Utf8(str::Utf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Custom(ref msg) => msg.fmt(f),
            Error::InvalidValue(ref msg) => write!(f, "invalid value: {}", msg),
            Error::Utf8(ref err) => write!(f, "invalid UTF-8: {}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Custom(ref msg) => msg,
            Error::InvalidValue(ref msg) => msg,
            Error::Utf8(ref err) => error::Error::description(err),
        }
    }

    /// The lower-level cause of this error, in the case of a `Utf8` error.
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Custom(_) | Error::InvalidValue(_) => None,
            Error::Utf8(ref err) => Some(err),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into().into())
    }

    fn invalid_value(msg: &str) -> Self {
        Error::InvalidValue(String::from(msg).into())
    }
}

/// State used when serializing sequences.
pub struct SeqState {
    _state: (),
}

/// State used when serializing tuples.
pub struct TupleState {
    _state: (),
}

/// State used when serializing tuple structs.
pub struct TupleStructState {
    _state: (),
}

/// State used when serializing tuple variants.
pub struct TupleVariantState {
    _state: (),
}

/// State used when serializing maps.
pub struct MapState {
    key: Option<Cow<'static, str>>
}

/// State used when serializing structs.
pub struct StructState {
    _state: (),
}

/// State used when serializing struct variants.
pub struct StructVariantState {
    _state: (),
}

impl<'output, Target> ser::Serializer for Serializer<'output, Target>
    where Target: 'output + UrlEncodedTarget
{
    type Error = Error;

    /// State used when serializing sequences.
    type SeqState = SeqState;

    /// State used when serializing tuples.
    type TupleState = TupleState;

    /// State used when serializing tuple structs.
    type TupleStructState = TupleStructState;

    /// State used when serializing tuple variants.
    type TupleVariantState = TupleVariantState;

    /// State used when serializing maps.
    type MapState = MapState;

    /// State used when serializing structs.
    type StructState = StructState;

    /// State used when serializing struct variants.
    type StructVariantState = StructVariantState;

    /// Returns an error.
    fn serialize_bool(&mut self, _v: bool) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_isize(&mut self, _v: isize) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i8(&mut self, _v: i8) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i16(&mut self, _v: i16) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i32(&mut self, _v: i32) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_i64(&mut self, _v: i64) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_usize(&mut self, _v: usize) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u8(&mut self, _v: u8) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u16(&mut self, _v: u16) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u32(&mut self, _v: u32) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_u64(&mut self, _v: u64) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_f32(&mut self, _v: f32) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_f64(&mut self, _v: f64) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_char(&mut self, _v: char) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_str(&mut self, _value: &str) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_bytes(&mut self, _value: &[u8]) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_unit(&mut self) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_unit_struct(
            &mut self, _name: &'static str)
            -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_unit_variant(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str)
            -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Serializes the inner value, ignoring the newtype name.
    fn serialize_newtype_struct<T>(
            &mut self, _name: &'static str, value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        value.serialize(self)
    }

    /// Returns an error.
    fn serialize_newtype_variant<T>(
            &mut self,                                    
            _name: &'static str,                                    
            _variant_index: usize,                                    
            _variant: &'static str,                                    
            _value: T)                                    
            -> Result<(), Error>
        where T: ser::Serialize
    {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_none(&mut self) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_some<T>(&mut self, _value: T) -> Result<(), Error>
        where T: ser::Serialize
    {
        Err(Error::top_level())
    }

    /// Begins to serialize a sequence, given length (if any) is ignored.
    fn serialize_seq(
            &mut self, _len: Option<usize>)
            -> Result<SeqState, Error> {
        Ok(SeqState { _state: () })
    }

    /// Serializes a sequence element.
    fn serialize_seq_elt<T>(
            &mut self, _state: &mut SeqState, value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        value.serialize(&mut pair::PairSerializer::new(self.urlencoder))
    }

    /// Finishes serializing a sequence.
    fn serialize_seq_end(&mut self, _state: SeqState) -> Result<(), Error> {
        Ok(())
    }

    /// Begins to serialize a sequence, given length is ignored.
    fn serialize_seq_fixed_size(
            &mut self, _length: usize)
            -> Result<SeqState, Error> {
        Ok(SeqState { _state: () })
    }

    /// Returns an error.
    fn serialize_tuple(&mut self, _len: usize) -> Result<TupleState, Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_elt<T>(
            &mut self, _state: &mut TupleState, _value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_end(&mut self, _state: TupleState) -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_struct(
            &mut self, _name: &'static str, _len: usize)
            -> Result<TupleStructState, Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_struct_elt<T>(
            &mut self, _state: &mut TupleStructState, _value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_struct_end(
            &mut self, _state: TupleStructState)
            -> Result<(), Error>
    {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_variant(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str,
            _len: usize)
            -> Result<TupleVariantState, Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_variant_elt<T>(
            &mut self, _state: &mut TupleVariantState, _value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_tuple_variant_end(
            &mut self, _state: TupleVariantState)
            -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Begins to serialize a map, given length (if any) is ignored.
    fn serialize_map(
            &mut self, _len: Option<usize>)
            -> Result<MapState, Error> {
        Ok(MapState { key: None })
    }

    /// Serializes a map key.
    fn serialize_map_key<T>(
            &mut self, state: &mut MapState, key: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        key.serialize(&mut key::MapKeySerializer::new(&mut state.key))
    }

    /// Serializes a map value.
    fn serialize_map_value<T>(
            &mut self, state: &mut MapState, value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        let mut value_serializer =
            try!(value::ValueSerializer::new(&mut state.key, self.urlencoder));
        value.serialize(&mut value_serializer)
    }

    /// Finishes serializing a map.
    fn serialize_map_end(&mut self, _state: MapState) -> Result<(), Error> {
        Ok(())
    }

    /// Begins to serialize a struct, given length is ignored.
    fn serialize_struct(
            &mut self, _name: &'static str, _len: usize)
            -> Result<StructState, Error> {
        Ok(StructState { _state: () })
    }

    /// Serializes a struct element.
    fn serialize_struct_elt<T>(
            &mut self,
            _state: &mut StructState,
            key: &'static str,
            value: T)
            -> Result<(), Error>
        where T: ser::Serialize
    {
        let mut key = Some(key.into());
        let mut value_serializer =
            value::ValueSerializer::new(&mut key, self.urlencoder).unwrap();
        value.serialize(&mut value_serializer)
    }

    /// Finishes serializing a struct.
    fn serialize_struct_end(&mut self, _state: StructState)
                            -> Result<(), Error> {
        Ok(())
    }

    /// Returns an error.
    fn serialize_struct_variant(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str,
            _len: usize)
            -> Result<StructVariantState, Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_struct_variant_elt<T>(
            &mut self,
            _state: &mut StructVariantState,
            _key: &'static str,
            _value: T)
            -> Result<(), Error> {
        Err(Error::top_level())
    }

    /// Returns an error.
    fn serialize_struct_variant_end(
            &mut self, _state: StructVariantState)
            -> Result<(), Error> {
        Err(Error::top_level())
    }
}

impl Error {
    fn top_level() -> Self {
        Error::Custom(
            "top-level serializer supports only maps and structs".into())
    }
}
