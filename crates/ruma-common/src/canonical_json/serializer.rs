#![allow(clippy::exhaustive_structs)]

use std::fmt;

use serde::ser::{Impossible, Serialize};

use super::{CanonicalJsonError, CanonicalJsonObject, CanonicalJsonValue, to_canonical_value};

/// Type alias for serialization results.
type Result<T> = std::result::Result<T, CanonicalJsonError>;

/// A [`serde::Serializer`] whose output is a [`CanonicalJsonValue`].
///
/// This behaves similarly to [`serde_json::value::Serializer`], except for the following
/// restrictions which return errors:
///
/// - Integers must be in the range accepted by [`js_int::Int`].
/// - Floats and bytes are not serializable.
/// - Booleans and integers cannot be used as keys for an object. `serde_json` accepts those types
///   as keys by serializing them as strings.
/// - The same key cannot be serialized twice in an object. `serde_json` uses the last value that is
///   serialized for the same key (at the time of writing).
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = SerializeNamedValue<SerializeArray>;
    type SerializeMap = SerializeObject;
    type SerializeStruct = SerializeObject;
    type SerializeStructVariant = SerializeNamedValue<SerializeObject>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(
            value.try_into().map_err(|_| CanonicalJsonError::IntegerOutOfRange)?,
        ))
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(
            value.try_into().map_err(|_| CanonicalJsonError::IntegerOutOfRange)?,
        ))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(value.into()))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(
            value.try_into().map_err(|_| CanonicalJsonError::IntegerOutOfRange)?,
        ))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Integer(
            value.try_into().map_err(|_| CanonicalJsonError::IntegerOutOfRange)?,
        ))
    }

    #[inline]
    fn serialize_f32(self, _float: f32) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("float".to_owned()))
    }

    #[inline]
    fn serialize_f64(self, _float: f64) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("float".to_owned()))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::String(value.into()))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::String(value.to_owned()))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("bytes".to_owned()))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        // Serialize as `null`.
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        // Serialize the name of the variant.
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the inner value.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // Serialize as a `{ variant: value }` object.
        let mut values = CanonicalJsonObject::new();
        values.insert(variant.to_owned(), to_canonical_value(value)?);
        Ok(CanonicalJsonValue::Object(values))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        // Serialize as an array.
        Ok(SerializeArray(Vec::with_capacity(len.unwrap_or_default())))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        // Serialize as an array.
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        // Serialize as an array.
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        // Serialize as a `{ variant: [fields…] }` object.
        Ok(SerializeNamedValue {
            name: String::from(variant),
            serialize: self.serialize_tuple(len)?,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeObject { object: CanonicalJsonObject::new(), next_key: None })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        // Serialize as a `{ variant: { fields… } }` object.
        Ok(SerializeNamedValue {
            name: String::from(variant),
            serialize: self.serialize_struct(variant, len)?,
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + fmt::Display,
    {
        Ok(CanonicalJsonValue::String(value.to_string()))
    }
}

/// Serializer to [`CanonicalJsonValue::Array`].
pub struct SerializeArray(Vec<CanonicalJsonValue>);

impl serde::ser::SerializeSeq for SerializeArray {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0.push(to_canonical_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Array(self.0))
    }
}

impl serde::ser::SerializeTuple for SerializeArray {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeArray {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

/// Serializer to [`CanonicalJsonValue::Object`].
pub struct SerializeObject {
    /// The serialized object.
    object: CanonicalJsonObject,
    /// Cache for the key to use for the next value when parsing maps.
    next_key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeObject {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(ObjectKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = self.next_key.take().expect("serialize_value called before serialize_key");

        if self.object.contains_key(&key) {
            return Err(CanonicalJsonError::DuplicateObjectKey(key));
        }

        self.object.insert(key, to_canonical_value(value)?);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(CanonicalJsonValue::Object(self.object))
    }
}

impl serde::ser::SerializeStruct for SerializeObject {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

/// Serializer for a [`CanonicalJsonValue`] with a name.
///
/// Serializes to [`CanonicalJsonValue::Object`].
pub struct SerializeNamedValue<V> {
    /// The name.
    name: String,
    /// The value serializer.
    serialize: V,
}

impl serde::ser::SerializeTupleVariant for SerializeNamedValue<SerializeArray> {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(&mut self.serialize, value)
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = CanonicalJsonObject::new();
        object.insert(self.name, serde::ser::SerializeSeq::end(self.serialize)?);
        Ok(CanonicalJsonValue::Object(object))
    }
}

impl serde::ser::SerializeStructVariant for SerializeNamedValue<SerializeObject> {
    type Ok = CanonicalJsonValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(&mut self.serialize, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = CanonicalJsonObject::new();
        object.insert(self.name, serde::ser::SerializeMap::end(self.serialize)?);
        Ok(CanonicalJsonValue::Object(object))
    }
}

/// Serializer for the key of a map.
///
/// Only accepts strings.
struct ObjectKeySerializer;

impl serde::Serializer for ObjectKeySerializer {
    type Ok = String;
    type Error = CanonicalJsonError;

    type SerializeSeq = Impossible<String, CanonicalJsonError>;
    type SerializeTuple = Impossible<String, CanonicalJsonError>;
    type SerializeTupleStruct = Impossible<String, CanonicalJsonError>;
    type SerializeTupleVariant = Impossible<String, CanonicalJsonError>;
    type SerializeMap = Impossible<String, CanonicalJsonError>;
    type SerializeStruct = Impossible<String, CanonicalJsonError>;
    type SerializeStructVariant = Impossible<String, CanonicalJsonError>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("bool".to_owned()))
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("integer".to_owned()))
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("float".to_owned()))
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("integer".to_owned()))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidType("bytes".to_owned()))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("()".to_owned()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType(name.to_owned()))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(CanonicalJsonError::InvalidObjectKeyType(format!("{name}::{variant}")))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(CanonicalJsonError::InvalidObjectKeyType("Option".to_owned()))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(CanonicalJsonError::InvalidObjectKeyType("Option".to_owned()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(CanonicalJsonError::InvalidObjectKeyType("sequence".to_owned()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(CanonicalJsonError::InvalidObjectKeyType("tuple".to_owned()))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(CanonicalJsonError::InvalidObjectKeyType(name.to_owned()))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(CanonicalJsonError::InvalidObjectKeyType(format!("{name}::{variant}")))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(CanonicalJsonError::InvalidObjectKeyType("map".to_owned()))
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(CanonicalJsonError::InvalidObjectKeyType(name.to_owned()))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(CanonicalJsonError::InvalidObjectKeyType(format!("{name}::{variant}")))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + fmt::Display,
    {
        Ok(value.to_string())
    }
}
