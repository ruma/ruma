use std::{iter, ptr, vec};

use serde::de::{
    self,
    value::{Error, SeqDeserializer},
    Deserializer, IntoDeserializer,
};

#[derive(Debug)]
pub enum ValOrVec<T> {
    Val(T),
    Vec(Vec<T>),
}

impl<T> ValOrVec<T> {
    pub fn push(&mut self, new_val: T) {
        match self {
            Self::Val(val) => {
                let mut vec = Vec::with_capacity(2);
                // Safety:
                //
                // since the vec is pre-allocated, push can't panic, so there
                // is no opportunity for a panic in the unsafe code.
                unsafe {
                    let existing_val = ptr::read(val);
                    vec.push(existing_val);
                    vec.push(new_val);
                    ptr::write(self, Self::Vec(vec))
                }
            }
            Self::Vec(vec) => vec.push(new_val),
        }
    }
}

impl<T> IntoIterator for ValOrVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

pub enum IntoIter<T> {
    Val(iter::Once<T>),
    Vec(vec::IntoIter<T>),
}

impl<T> IntoIter<T> {
    fn new(vv: ValOrVec<T>) -> Self {
        match vv {
            ValOrVec::Val(val) => Self::Val(iter::once(val)),
            ValOrVec::Vec(vec) => Self::Vec(vec.into_iter()),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Val(iter) => iter.next(),
            Self::Vec(iter) => iter.next(),
        }
    }
}

impl<'de, T> IntoDeserializer<'de> for ValOrVec<T>
where
    T: IntoDeserializer<'de> + Deserializer<'de, Error = Error>,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

macro_rules! forward_to_part {
    ($($method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor<'de>
            {
                match self {
                    Self::Val(val) => val.$method(visitor),
                    Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
                }
            }
        )*
    }
}

impl<'de, T> Deserializer<'de> for ValOrVec<T>
where
    T: IntoDeserializer<'de> + Deserializer<'de, Error = Error>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_any(visitor),
            Self::Vec(_) => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(self.into_iter()))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_enum(name, variants, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
    }

    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_tuple(len, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
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
        match self {
            Self::Val(val) => val.deserialize_struct(name, fields, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_unit_struct(name, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_tuple_struct(name, len, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Val(val) => val.deserialize_newtype_struct(name, visitor),
            Self::Vec(_) => Err(de::Error::custom("TODO: Error message")),
        }
    }

    forward_to_part! {
        deserialize_bool,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_unit,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_f32,
        deserialize_f64,
        deserialize_option,
        deserialize_identifier,
        deserialize_ignored_any,
        deserialize_map,
    }
}
