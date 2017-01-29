use ser::Error;
use serde::ser;
use std::marker::PhantomData;
use void;

pub struct VoidSerializer<Ok> {
    void: void::Void,
    _marker: PhantomData<Ok>,
}

impl<Ok> ser::SerializeSeq for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self,
                                                     _value: &T)
                                                     -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeTuple for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self,
                                                     _value: &T)
                                                     -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeTupleStruct for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeTupleVariant for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeMap for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_key<T: ?Sized + ser::Serialize>(&mut self,
                                                 _key: &T)
                                                 -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn serialize_value<T: ?Sized + ser::Serialize>(&mut self,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeStruct for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   _key: &'static str,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}

impl<Ok> ser::SerializeStructVariant for VoidSerializer<Ok> {
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   _key: &'static str,
                                                   _value: &T)
                                                   -> Result<(), Error> {
        void::unreachable(self.void)
    }

    fn end(self) -> Result<Ok, Error> {
        void::unreachable(self.void)
    }
}
