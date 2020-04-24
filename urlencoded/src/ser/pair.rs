use std::borrow::Cow;
use std::mem;

use serde::ser;

use url::form_urlencoded::{
    Serializer as UrlEncodedSerializer, Target as UrlEncodedTarget,
};

use crate::{
    error::{Error, Result},
    ser::{key::KeySink, part::PartSerializer, value::ValueSink},
};

macro_rules! serialize_pair {
    ($($ty:ty => $name:ident,)*) => {
        $(
            fn $name(self, value: $ty) -> Result<()> {
                let key = if let Some(key) = self.key {
                    key.clone()
                } else {
                    return Err(Error::no_key());
                };
                let value_sink = ValueSink::new(self.urlencoder, &key);
                let value_serializer = PartSerializer::new(value_sink);
                value_serializer.$name(value)
            }
        )*
    };
}

pub struct PairSerializer<'input, 'target, Target: 'target + UrlEncodedTarget> {
    urlencoder: &'target mut UrlEncodedSerializer<'input, Target>,
    state: PairState,
    key: Option<&'target Cow<'target, str>>,
    count: &'target mut usize,
}

impl<'input, 'target, Target> PairSerializer<'input, 'target, Target>
where
    Target: 'target + UrlEncodedTarget,
{
    pub fn new(
        urlencoder: &'target mut UrlEncodedSerializer<'input, Target>,
        key: Option<&'target Cow<'target, str>>,
        count: &'target mut usize,
    ) -> Self {
        PairSerializer {
            urlencoder,
            state: PairState::WaitingForKey,
            key,
            count,
        }
    }
}

impl<'input, 'target, Target> ser::Serializer
    for PairSerializer<'input, 'target, Target>
where
    Target: 'target + UrlEncodedTarget,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    serialize_pair! {
        bool => serialize_bool,
        u8  => serialize_u8,
        u16 => serialize_u16,
        u32 => serialize_u32,
        u64 => serialize_u64,
        i8  => serialize_i8,
        i16 => serialize_i16,
        i32 => serialize_i32,
        i64 => serialize_i64,
        f32 => serialize_f32,
        f64 => serialize_f64,
        char => serialize_char,
        &str => serialize_str,
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        let key = if let Some(key) = self.key {
            key.clone()
        } else {
            let key = Cow::Owned(self.count.to_string());
            *self.count += 1;
            key
        };
        let value_sink = ValueSink::new(self.urlencoder, &key);
        let value_serializer = PartSerializer::new(value_sink);
        value_serializer.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(Error::unsupported_pair())
    }

    fn serialize_none(self) -> Result<()> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + ser::Serialize>(
        self,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self> {
        if len == 2 {
            Ok(self)
        } else {
            Err(Error::unsupported_pair())
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::unsupported_pair())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::unsupported_pair())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::unsupported_pair())
    }
}

impl<'input, 'target, Target> ser::SerializeSeq
    for PairSerializer<'input, 'target, Target>
where
    Target: 'target + UrlEncodedTarget,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        value.serialize(PairSerializer::new(
            self.urlencoder,
            self.key,
            &mut self.count,
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'input, 'target, Target> ser::SerializeTuple
    for PairSerializer<'input, 'target, Target>
where
    Target: 'target + UrlEncodedTarget,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(
        &mut self,
        value: &T,
    ) -> Result<()> {
        match mem::replace(&mut self.state, PairState::Done) {
            PairState::WaitingForKey => {
                let key_sink = KeySink::new(|key| Ok(key.into()));
                let key_serializer = PartSerializer::new(key_sink);
                self.state = PairState::WaitingForValue {
                    key: value.serialize(key_serializer)?,
                };
                Ok(())
            },
            PairState::WaitingForValue { key } => {
                let result = {
                    let value_sink = ValueSink::new(self.urlencoder, &key);
                    let value_serializer = PartSerializer::new(value_sink);
                    value.serialize(value_serializer)
                };
                if result.is_ok() {
                    self.state = PairState::Done;
                } else {
                    self.state = PairState::WaitingForValue { key: key };
                }
                result
            },
            PairState::Done => Err(Error::done()),
        }
    }

    fn end(self) -> Result<()> {
        if let PairState::Done = self.state {
            Ok(())
        } else {
            Err(Error::not_done())
        }
    }
}

enum PairState {
    WaitingForKey,
    WaitingForValue { key: Cow<'static, str> },
    Done,
}
