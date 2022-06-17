use std::{collections::BTreeMap, fmt};

use js_int::Int;
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use serde_json::{to_string as to_json_string, Value as JsonValue};

use super::CanonicalJsonError;

/// The inner type of `CanonicalJsonValue::Object`.
#[cfg(feature = "canonical-json")]
pub type CanonicalJsonObject = BTreeMap<String, CanonicalJsonValue>;

/// Represents a canonical JSON value as per the Matrix specification.
#[cfg(feature = "canonical-json")]
#[derive(Clone, Eq, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum CanonicalJsonValue {
    /// Represents a JSON null value.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(null).try_into().unwrap();
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(true).try_into().unwrap();
    /// ```
    Bool(bool),

    /// Represents a JSON integer.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(12).try_into().unwrap();
    /// ```
    Integer(Int),

    /// Represents a JSON string.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!("a string").try_into().unwrap();
    /// ```
    String(String),

    /// Represents a JSON array.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(["an", "array"]).try_into().unwrap();
    /// ```
    Array(Vec<CanonicalJsonValue>),

    /// Represents a JSON object.
    ///
    /// The map is backed by a BTreeMap to guarantee the sorting of keys.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use ruma_common::serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!({ "an": "object" }).try_into().unwrap();
    /// ```
    Object(CanonicalJsonObject),
}

impl CanonicalJsonValue {
    /// If the `CanonicalJsonValue` is a `Bool`, return the inner value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is an `Integer`, return the inner value.
    pub fn as_integer(&self) -> Option<Int> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is a `String`, return a reference to the inner value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is an `Array`, return a reference to the inner value.
    pub fn as_array(&self) -> Option<&[CanonicalJsonValue]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is an `Object`, return a reference to the inner value.
    pub fn as_object(&self) -> Option<&CanonicalJsonObject> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is an `Array`, return a mutable reference to the inner value.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<CanonicalJsonValue>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    /// If the `CanonicalJsonValue` is an `Object`, return a mutable reference to the inner value.
    pub fn as_object_mut(&mut self) -> Option<&mut CanonicalJsonObject> {
        match self {
            Self::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Returns `true` if the `CanonicalJsonValue` is a `Bool`.
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if the `CanonicalJsonValue` is an `Integer`.
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    /// Returns `true` if the `CanonicalJsonValue` is a `String`.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if the `CanonicalJsonValue` is an `Array`.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns `true` if the `CanonicalJsonValue` is an `Object`.
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
}

impl Default for CanonicalJsonValue {
    fn default() -> Self {
        Self::Null
    }
}

impl fmt::Debug for CanonicalJsonValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Null => formatter.debug_tuple("Null").finish(),
            Self::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            Self::Integer(ref v) => fmt::Debug::fmt(v, formatter),
            Self::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            Self::Array(ref v) => {
                formatter.write_str("Array(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
            Self::Object(ref v) => {
                formatter.write_str("Object(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
        }
    }
}

impl fmt::Display for CanonicalJsonValue {
    /// Display this value as a string.
    ///
    /// This `Display` implementation is intentionally unaffected by any formatting parameters,
    /// because adding extra whitespace or otherwise pretty-printing it would make it not the
    /// canonical form anymore.
    ///
    /// If you want to pretty-print a `CanonicalJsonValue` for debugging purposes, use
    /// one of `serde_json::{to_string_pretty, to_vec_pretty, to_writer_pretty}`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", to_json_string(&self).map_err(|_| fmt::Error)?)
    }
}

impl TryFrom<JsonValue> for CanonicalJsonValue {
    type Error = CanonicalJsonError;

    fn try_from(val: JsonValue) -> Result<Self, Self::Error> {
        Ok(match val {
            JsonValue::Bool(b) => Self::Bool(b),
            JsonValue::Number(num) => Self::Integer(
                Int::try_from(num.as_i64().ok_or(CanonicalJsonError::IntConvert)?)
                    .map_err(|_| CanonicalJsonError::IntConvert)?,
            ),
            JsonValue::Array(vec) => {
                Self::Array(vec.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?)
            }
            JsonValue::String(string) => Self::String(string),
            JsonValue::Object(obj) => Self::Object(
                obj.into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<Result<CanonicalJsonObject, _>>()?,
            ),
            JsonValue::Null => Self::Null,
        })
    }
}

impl From<CanonicalJsonValue> for JsonValue {
    fn from(val: CanonicalJsonValue) -> Self {
        match val {
            CanonicalJsonValue::Bool(b) => Self::Bool(b),
            CanonicalJsonValue::Integer(int) => Self::Number(i64::from(int).into()),
            CanonicalJsonValue::String(string) => Self::String(string),
            CanonicalJsonValue::Array(vec) => {
                Self::Array(vec.into_iter().map(Into::into).collect())
            }
            CanonicalJsonValue::Object(obj) => {
                Self::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            CanonicalJsonValue::Null => Self::Null,
        }
    }
}

macro_rules! variant_impls {
    ($variant:ident($ty:ty)) => {
        impl From<$ty> for CanonicalJsonValue {
            fn from(val: $ty) -> Self {
                Self::$variant(val)
            }
        }

        impl PartialEq<$ty> for CanonicalJsonValue {
            fn eq(&self, other: &$ty) -> bool {
                match self {
                    Self::$variant(val) => val == other,
                    _ => false,
                }
            }
        }

        impl PartialEq<CanonicalJsonValue> for $ty {
            fn eq(&self, other: &CanonicalJsonValue) -> bool {
                match other {
                    CanonicalJsonValue::$variant(val) => self == val,
                    _ => false,
                }
            }
        }
    };
}

variant_impls!(Bool(bool));
variant_impls!(Integer(Int));
variant_impls!(String(String));
variant_impls!(Array(Vec<CanonicalJsonValue>));
variant_impls!(Object(CanonicalJsonObject));

impl Serialize for CanonicalJsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Integer(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            Self::Array(v) => v.serialize(serializer),
            Self::Object(m) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for CanonicalJsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<CanonicalJsonValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = JsonValue::deserialize(deserializer)?;
        val.try_into().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::CanonicalJsonValue;

    #[test]
    fn to_string() {
        const CANONICAL_STR: &str = r#"{"city":"London","street":"10 Downing Street"}"#;

        let json: CanonicalJsonValue =
            json!({ "city": "London", "street": "10 Downing Street" }).try_into().unwrap();

        assert_eq!(format!("{json}"), CANONICAL_STR);
        assert_eq!(format!("{:#}", json), CANONICAL_STR);
    }
}
