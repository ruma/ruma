use std::{borrow::Cow, str};

use serde::de::{self, Deserializer, Unexpected, Visitor};

pub fn deserialize_cow_str<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(CowStrVisitor)
}

struct CowStrVisitor;

impl<'de> Visitor<'de> for CowStrVisitor {
    type Value = Cow<'de, str>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Borrowed(v))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Cow::Owned(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(Cow::Owned(s.to_owned())),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Bytes(v), &self)),
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(Cow::Owned(s)),
            Err(e) => Err(de::Error::invalid_value(Unexpected::Bytes(&e.into_bytes()), &self)),
        }
    }
}
