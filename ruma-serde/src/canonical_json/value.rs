use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fmt,
};

use js_int::Int;
use serde::de::Error;
use serde_json::Value as JsonValue;

#[derive(Clone, Eq, PartialEq)]
pub enum CanonicalJsonValue {
    /// Represents a JSON null value.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(null).try_into().unwrap();
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(true).try_into().unwrap();
    /// ```
    Bool(bool),

    /// Represents a JSON integer.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(12).try_into().unwrap();
    /// ```
    Integer(Int),

    /// Represents a JSON string.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!("a string").try_into().unwrap();
    /// ```
    String(String),

    /// Represents a JSON array.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!(["an", "array"]).try_into().unwrap();
    /// ```
    Array(Vec<CanonicalJsonValue>),

    /// Represents a JSON object.
    ///
    /// The map is backed by a BTreeMap to guarantee the sorting of keys.
    ///
    /// ```
    /// # use serde_json::json;
    /// # use std::convert::TryInto;
    /// # use ruma_serde::CanonicalJsonValue;
    /// let v: CanonicalJsonValue = json!({ "an": "object" }).try_into().unwrap();
    /// ```
    Object(BTreeMap<String, CanonicalJsonValue>),
}

impl CanonicalJsonValue {
    /// Returns a canonical JSON string according to Matrix specification.
    ///
    /// The method should be preferred over `serde_json::to_string` since it
    /// checks the size of the canonical string. Matrix canonical JSON enforces
    /// a size limit of less than 65,535 when sending PDU's for the server-server protocol.
    pub fn to_canonical_string(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string(self).and_then(|s| {
            if s.len() > 65_535 {
                Err(serde_json::Error::custom("TOO LARGE"))
            } else {
                Ok(s)
            }
        })?)
    }
}

impl fmt::Debug for CanonicalJsonValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CanonicalJsonValue::Null => formatter.debug_tuple("Null").finish(),
            CanonicalJsonValue::Bool(v) => formatter.debug_tuple("Bool").field(&v).finish(),
            CanonicalJsonValue::Integer(ref v) => fmt::Debug::fmt(v, formatter),
            CanonicalJsonValue::String(ref v) => formatter.debug_tuple("String").field(v).finish(),
            CanonicalJsonValue::Array(ref v) => {
                formatter.write_str("Array(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
            CanonicalJsonValue::Object(ref v) => {
                formatter.write_str("Object(")?;
                fmt::Debug::fmt(v, formatter)?;
                formatter.write_str(")")
            }
        }
    }
}

impl fmt::Display for CanonicalJsonValue {
    /// Display a JSON value as a string.
    ///
    /// ```
    /// # use serde_json::json;
    /// #
    /// let json = json!({ "city": "London", "street": "10 Downing Street" });
    ///
    /// // Canonical format:
    /// //
    /// // {"city":"London","street":"10 Downing Street"}
    /// let compact = format!("{}", json);
    /// assert_eq!(compact,
    ///     "{\"city\":\"London\",\"street\":\"10 Downing Street\"}");
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).map_err(|_| fmt::Error)?)
    }
}

impl TryFrom<JsonValue> for CanonicalJsonValue {
    type Error = serde_json::Error;
    fn try_from(json: JsonValue) -> Result<Self, Self::Error> {
        Ok(match json {
            JsonValue::Bool(b) => Self::Bool(b),
            JsonValue::Number(num) => Self::Integer(
                Int::try_from(num.as_i64().ok_or_else(|| {
                    serde_json::Error::custom("Invalid number found, expected i64")
                })?)
                .map_err(serde_json::Error::custom)?,
            ),
            JsonValue::Array(vec) => {
                Self::Array(vec.into_iter().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?)
            }
            JsonValue::String(string) => Self::String(string),
            JsonValue::Object(obj) => Self::Object(
                obj.into_iter()
                    .map(|(k, v)| Ok((k, v.try_into()?)))
                    .collect::<Result<BTreeMap<_, _>, _>>()?,
            ),
            JsonValue::Null => Self::Null,
        })
    }
}

impl serde::ser::Serialize for CanonicalJsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match *self {
            CanonicalJsonValue::Null => serializer.serialize_unit(),
            CanonicalJsonValue::Bool(b) => serializer.serialize_bool(b),
            CanonicalJsonValue::Integer(ref n) => n.serialize(serializer),
            CanonicalJsonValue::String(ref s) => serializer.serialize_str(s),
            CanonicalJsonValue::Array(ref v) => v.serialize(serializer),
            CanonicalJsonValue::Object(ref m) => {
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

impl<'de> serde::Deserialize<'de> for CanonicalJsonValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<CanonicalJsonValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = serde_json::Value::deserialize(deserializer)?;
        Ok(val.try_into().map_err(serde::de::Error::custom)?)
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use super::CanonicalJsonValue;

    #[test]
    fn serialize_canon() {
        let json: CanonicalJsonValue = serde_json::json!({
            "a": [1, 2, 3],
            "other": { "stuff": "hello" },
            "string": "Thing"
        })
        .try_into()
        .unwrap();

        let ser = json.to_canonical_string().unwrap();
        let back = serde_json::from_str::<CanonicalJsonValue>(&ser).unwrap();

        assert_eq!(json, back);
    }

    #[test]
    fn check_canonical_sorts_keys() {
        let json: CanonicalJsonValue = serde_json::json!({
            "auth": {
                "success": true,
                "mxid": "@john.doe:example.com",
                "profile": {
                    "display_name": "John Doe",
                    "three_pids": [
                        {
                            "medium": "email",
                            "address": "john.doe@example.org"
                        },
                        {
                            "medium": "msisdn",
                            "address": "123456789"
                        }
                    ]
                }
            }
        })
        .try_into()
        .unwrap();

        assert_eq!(
            serde_json::to_string(&json).unwrap(),
            r#"{"auth":{"mxid":"@john.doe:example.com","profile":{"display_name":"John Doe","three_pids":[{"address":"john.doe@example.org","medium":"email"},{"address":"123456789","medium":"msisdn"}]},"success":true}}"#
        )
    }
}
