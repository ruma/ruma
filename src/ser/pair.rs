use ser::{Error, key, value};
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use url::form_urlencoded;

pub struct PairSerializer<'target, Target>(
        &'target mut form_urlencoded::Serializer<Target>)
    where Target: 'target + form_urlencoded::Target;

impl<'target, Target> PairSerializer<'target, Target>
    where Target: 'target + form_urlencoded::Target
{
    pub fn new(
            serializer: &'target mut form_urlencoded::Serializer<Target>)
            -> Self {
        PairSerializer(serializer)
    }
}

pub struct TupleState(Option<Option<Cow<'static, str>>>);
pub struct TupleStructState(TupleState);

impl<'target, Target> Serializer for PairSerializer<'target, Target>
    where Target: 'target + form_urlencoded::Target
{
    type Error = Error;
    type SeqState = ();
    type TupleState = TupleState;
    type TupleStructState = TupleStructState;
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_bool(&mut self, _v: bool) -> Result<(), Error> {
        Err(Error::Custom("booleans are not supported values".into()))
    }

    fn serialize_isize(&mut self, _v: isize) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_i8(&mut self, _v: i8) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_i16(&mut self, _v: i16) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_i32(&mut self, _v: i32) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_i64(&mut self, _v: i64) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_usize(&mut self, _v: usize) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_u8(&mut self, _v: u8) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_u16(&mut self, _v: u16) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_u32(&mut self, _v: u32) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_u64(&mut self, _v: u64) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_f32(&mut self, _v: f32) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_f64(&mut self, _v: f64) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_char(&mut self, _v: char) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_str(&mut self, _value: &str) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_bytes(&mut self, _value: &[u8]) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit(&mut self) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit_struct(
            &mut self, _name: &'static str)
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_unit_variant(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str)
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_newtype_struct<T>(
            &mut self, _name: &'static str, value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str,
            _value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_some<T>(&mut self, _value: T) -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_seq(&mut self, _len: Option<usize>)
                     -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_seq_elt<T>(&mut self, _state: &mut (), _value: T)
                            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_seq_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_seq_fixed_size(&mut self, _size: usize)
                                -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_tuple(&mut self, len: usize) -> Result<TupleState, Error> {
        if len == 2 {
            Ok(TupleState(None))
        } else {
            Err(Error::unsupported_pair())
        }
    }

    fn serialize_tuple_elt<T>(
            &mut self, state: &mut TupleState, value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        match state.0.take() {
            None => {
                let mut key = None;
                {
                    let mut key_serializer =
                        key::MapKeySerializer::new(&mut key);
                    try!(value.serialize(&mut key_serializer));
                }
                state.0 = Some(key);
                Ok(())
            },
            Some(ref mut key) => {
                {
                    let mut value_serializer =
                        value::ValueSerializer::new(key, &mut self.0).unwrap();
                    try!(value.serialize(&mut value_serializer));
                }
                state.0 = Some(None);
                Ok(())
            }
        }
    }

    fn serialize_tuple_end(&mut self, _state: TupleState) -> Result<(), Error> {
        Ok(())
    }

    fn serialize_tuple_struct(
            &mut self, _name: &'static str, len: usize)
            -> Result<TupleStructState, Error> {
        self.serialize_tuple(len).map(TupleStructState)
    }

    fn serialize_tuple_struct_elt<T>(
            &mut self, state: &mut TupleStructState, value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        self.serialize_tuple_elt(&mut state.0, value)
    }

    fn serialize_tuple_struct_end(
            &mut self, _state: TupleStructState)
            -> Result<(), Error> {
        Ok(())
    }

    fn serialize_tuple_variant(
        &mut self,
        _name: &'static str,
        _variant_index: usize,
        _variant: &'static str,
        _len: usize)
        -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_tuple_variant_elt<T>(
            &mut self, _state: &mut (), _value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_tuple_variant_end(
            &mut self, _state: ())
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_map(
            &mut self, _len: Option<usize>)
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_map_key<T>(
            &mut self, _state: &mut (), _key: T)
            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_map_value<T>(
            &mut self, _state: &mut (), _value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_map_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct(&mut self, _name: &'static str, _len: usize)
                        -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct_elt<T>(
            &mut self,
            _state: &mut (),
            _key: &'static str,
            _value: T)
            -> Result<(), Error>
        where T: Serialize
    {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct_end(
            &mut self, _state: ())
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct_variant(
            &mut self,
            _name: &'static str,
            _variant_index: usize,
            _variant: &'static str,
            _len: usize)
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }
    fn serialize_struct_variant_elt<T>(
            &mut self,
            _state: &mut (),
            _key: &'static str,
            _value: T)
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }

    fn serialize_struct_variant_end(
            &mut self, _state: ())
            -> Result<(), Error> {
        Err(Error::unsupported_pair())
    }
}

impl Error {    
    fn unsupported_pair() -> Self {
        Error::Custom("unsupported pair".into())
    }
}
