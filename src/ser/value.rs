use ser::Error;
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use std::str;
use url::form_urlencoded;

pub struct ValueSerializer<'key, 'target, Target>
    where Target: 'target + form_urlencoded::Target,
{
    key: &'key mut Option<Cow<'static, str>>,
    serializer: &'target mut form_urlencoded::Serializer<Target>,
}

impl<'key, 'target, Target> ValueSerializer<'key, 'target, Target>
    where Target: 'target + form_urlencoded::Target,
{
    pub fn new(key: &'key mut Option<Cow<'static, str>>,
               serializer: &'target mut form_urlencoded::Serializer<Target>)
               -> Result<Self, Error> {
        if key.is_some() {
            Ok(ValueSerializer {
                key: key,
                serializer: serializer,
            })
        } else {
            Err(Error::no_key())
        }
    }

    fn append_pair(&mut self, value: &str) -> Result<(), Error> {
        if let Some(key) = self.key.take() {
            self.serializer.append_pair(&key, value);
            Ok(())
        } else {
            Err(Error::no_key())
        }
    }
}

impl<'key, 'target, Target> Serializer
    for ValueSerializer<'key, 'target, Target>
    where Target: 'target + form_urlencoded::Target,
{
    type Error = Error;
    type SeqState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_bool(&mut self, v: bool) -> Result<(), Error> {
        self.append_pair(if v { "true" } else { "false" })
    }

    fn serialize_isize(&mut self, v: isize) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_usize(&mut self, v: usize) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_char(&mut self, v: char) -> Result<(), Error> {
        self.append_pair(&v.to_string())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), Error> {
        self.append_pair(value)
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        match str::from_utf8(value) {
            Ok(value) => self.append_pair(value),
            Err(err) => Err(Error::Utf8(err)),
        }
    }

    fn serialize_unit(&mut self) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_unit_struct(&mut self,
                             name: &'static str)
                             -> Result<(), Error> {
        self.append_pair(name)
    }

    fn serialize_unit_variant(&mut self,
                              _name: &'static str,
                              _variant_index: usize,
                              variant: &'static str)
                              -> Result<(), Error> {
        self.append_pair(variant)
    }

    fn serialize_newtype_struct<T>(&mut self,
                                   _name: &'static str,
                                   value: T)
                                   -> Result<(), Error>
        where T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(&mut self,
                                    _name: &'static str,
                                    _variant_index: usize,
                                    _variant: &'static str,
                                    _value: T)
                                    -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        if let Some(_) = self.key.take() {
            Ok(())
        } else {
            Err(Error::no_key())
        }
    }

    fn serialize_some<T>(&mut self, value: T) -> Result<(), Error>
        where T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(&mut self, _len: Option<usize>) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_seq_elt<T>(&mut self,
                            _state: &mut (),
                            _value: T)
                            -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_seq_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_seq_fixed_size(&mut self, _size: usize) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple(&mut self, _len: usize) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_elt<T>(&mut self,
                              _state: &mut (),
                              _value: T)
                              -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_struct(&mut self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_struct_elt<T>(&mut self,
                                     _state: &mut (),
                                     _value: T)
                                     -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_struct_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_variant(&mut self,
                               _name: &'static str,
                               _variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_variant_elt<T>(&mut self,
                                      _state: &mut (),
                                      _value: T)
                                      -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_tuple_variant_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_map(&mut self, _len: Option<usize>) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_map_key<T>(&mut self,
                            _state: &mut (),
                            _key: T)
                            -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_map_value<T>(&mut self,
                              _state: &mut (),
                              _value: T)
                              -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_map_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_struct(&mut self,
                        _name: &'static str,
                        _len: usize)
                        -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_struct_elt<T>(&mut self,
                               _state: &mut (),
                               _key: &'static str,
                               _value: T)
                               -> Result<(), Error>
        where T: Serialize,
    {
        Err(Error::unsupported_value())
    }

    fn serialize_struct_end(&mut self, _state: ()) -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_struct_variant(&mut self,
                                _name: &'static str,
                                _variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<(), Error> {
        Err(Error::unsupported_value())
    }
    fn serialize_struct_variant_elt<T>(&mut self,
                                       _state: &mut (),
                                       _key: &'static str,
                                       _value: T)
                                       -> Result<(), Error> {
        Err(Error::unsupported_value())
    }

    fn serialize_struct_variant_end(&mut self,
                                    _state: ())
                                    -> Result<(), Error> {
        Err(Error::unsupported_value())
    }
}

impl Error {
    fn no_key() -> Self {
        Error::Custom("tried to serialize a value before serializing key"
            .into())
    }

    fn unsupported_value() -> Self {
        Error::Custom("unsupported value".into())
    }
}
