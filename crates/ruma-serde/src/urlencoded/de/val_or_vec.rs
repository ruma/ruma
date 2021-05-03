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
            // To transform a Self::Val into a Self::Vec, we take the existing
            // value out via ptr::read and add it to a vector, together with the
            // new value. Since setting self to `ValOrVec::Vec` normally would
            // cause T's Drop implementation to run if it has one (which would
            // free resources that will now be owned by the first vec element),
            // we instead use ptr::write to set self to Self::Vec.
            ValOrVec::Val(val) => {
                let mut vec = Vec::with_capacity(2);
                // Safety: since the vec is pre-allocated, push can't panic, so
                //   there is no opportunity for outside code to observe an
                //   invalid state of self.
                unsafe {
                    let existing_val = ptr::read(val);
                    vec.push(existing_val);
                    vec.push(new_val);
                    ptr::write(self, ValOrVec::Vec(vec))
                }
            }
            ValOrVec::Vec(vec) => vec.push(new_val),
        }
    }

    fn deserialize_val<U, E, F>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
        E: de::Error,
    {
        match self {
            ValOrVec::Val(val) => f(val),
            ValOrVec::Vec(_) => Err(de::Error::custom("unsupported value")),
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
            ValOrVec::Val(val) => IntoIter::Val(iter::once(val)),
            ValOrVec::Vec(vec) => IntoIter::Vec(vec.into_iter()),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IntoIter::Val(iter) => iter.next(),
            IntoIter::Vec(iter) => iter.next(),
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
                self.deserialize_val(move |val| val.$method(visitor))
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
            ValOrVec::Val(val) => val.deserialize_any(visitor),
            ValOrVec::Vec(_) => self.deserialize_seq(visitor),
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
        self.deserialize_val(move |val| val.deserialize_enum(name, variants, visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_val(move |val| val.deserialize_tuple(len, visitor))
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
        self.deserialize_val(move |val| val.deserialize_struct(name, fields, visitor))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_val(move |val| val.deserialize_unit_struct(name, visitor))
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
        self.deserialize_val(move |val| val.deserialize_tuple_struct(name, len, visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_val(move |val| val.deserialize_newtype_struct(name, visitor))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
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
        deserialize_map,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use matches::assert_matches;

    use super::ValOrVec;

    #[test]
    fn cow_borrowed() {
        let mut x = ValOrVec::Val(Cow::Borrowed("a"));
        x.push(Cow::Borrowed("b"));
        x.push(Cow::Borrowed("c"));
        assert_matches!(x, ValOrVec::Vec(v) if v == vec!["a", "b", "c"]);
    }

    #[test]
    fn cow_owned() {
        let mut x = ValOrVec::Val(Cow::from("a".to_owned()));
        x.push(Cow::from("b".to_owned()));
        x.push(Cow::from("c".to_owned()));
        assert_matches!(
            x,
            ValOrVec::Vec(v) if v == vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
        );
    }
}
