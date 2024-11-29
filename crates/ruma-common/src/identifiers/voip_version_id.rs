//! Matrix VoIP version identifier.

use std::fmt;

use js_int::UInt;
use ruma_macros::DisplayAsRefStr;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{IdParseError, PrivOwnedStr};

/// A Matrix VoIP version ID.
///
/// A `VoipVersionId` representing VoIP version 0 can be converted or deserialized from a `UInt`,
/// and can be converted or serialized back into a `UInt` as needed.
///
/// Custom room versions or ones that were introduced into the specification after this code was
/// written are represented by a hidden enum variant. They can be converted or deserialized from a
/// string slice, and can be converted or serialized back into a string as needed.
///
/// ```
/// # use ruma_common::VoipVersionId;
/// assert_eq!(VoipVersionId::try_from("1").unwrap().as_ref(), "1");
/// ```
///
/// For simplicity, version 0 has a string representation, but trying to construct a `VoipVersionId`
/// from a `"0"` string will not result in the `V0` variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash, DisplayAsRefStr)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum VoipVersionId {
    /// A version 0 VoIP call.
    V0,

    /// A version 1 VoIP call.
    V1,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl VoipVersionId {
    /// Creates a string slice from this `VoipVersionId`.
    pub fn as_str(&self) -> &str {
        match &self {
            Self::V0 => "0",
            Self::V1 => "1",
            Self::_Custom(PrivOwnedStr(s)) => s,
        }
    }

    /// Creates a byte slice from this `VoipVersionId`.
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl From<VoipVersionId> for String {
    fn from(id: VoipVersionId) -> Self {
        match id {
            VoipVersionId::_Custom(PrivOwnedStr(version)) => version.into(),
            _ => id.as_str().to_owned(),
        }
    }
}

impl AsRef<str> for VoipVersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for VoipVersionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CallVersionVisitor;

        impl Visitor<'_> for CallVersionVisitor {
            type Value = VoipVersionId;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("0 or string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value.into())
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let uint = UInt::try_from(value).map_err(de::Error::custom)?;
                Self::Value::try_from(uint).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(CallVersionVisitor)
    }
}

impl Serialize for VoipVersionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::V0 => serializer.serialize_u64(0),
            _ => serializer.serialize_str(self.as_str()),
        }
    }
}

impl TryFrom<UInt> for VoipVersionId {
    type Error = IdParseError;

    fn try_from(u: UInt) -> Result<Self, Self::Error> {
        ruma_identifiers_validation::voip_version_id::validate(u)?;
        Ok(Self::V0)
    }
}

fn from<T>(s: T) -> VoipVersionId
where
    T: AsRef<str> + Into<Box<str>>,
{
    match s.as_ref() {
        "1" => VoipVersionId::V1,
        _ => VoipVersionId::_Custom(PrivOwnedStr(s.into())),
    }
}

impl From<&str> for VoipVersionId {
    fn from(s: &str) -> Self {
        from(s)
    }
}

impl From<String> for VoipVersionId {
    fn from(s: String) -> Self {
        from(s)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::VoipVersionId;
    use crate::IdParseError;

    #[test]
    fn valid_version_0() {
        assert_eq!(VoipVersionId::try_from(uint!(0)), Ok(VoipVersionId::V0));
    }

    #[test]
    fn invalid_uint_version() {
        assert_matches!(
            VoipVersionId::try_from(uint!(1)),
            Err(IdParseError::InvalidVoipVersionId(_))
        );
    }

    #[test]
    fn valid_version_1() {
        assert_eq!(VoipVersionId::from("1"), VoipVersionId::V1);
    }

    #[test]
    fn valid_custom_string_version() {
        assert_matches!(VoipVersionId::from("io.ruma.2"), version);
        assert_eq!(version.as_ref(), "io.ruma.2");
    }

    #[test]
    fn serialize_version_0() {
        assert_eq!(to_json_value(&VoipVersionId::V0).unwrap(), json!(0));
    }

    #[test]
    fn deserialize_version_0() {
        assert_eq!(from_json_value::<VoipVersionId>(json!(0)).unwrap(), VoipVersionId::V0);
    }

    #[test]
    fn serialize_version_1() {
        assert_eq!(to_json_value(&VoipVersionId::V1).unwrap(), json!("1"));
    }

    #[test]
    fn deserialize_version_1() {
        assert_eq!(from_json_value::<VoipVersionId>(json!("1")).unwrap(), VoipVersionId::V1);
    }

    #[test]
    fn serialize_custom_string() {
        let version = VoipVersionId::from("io.ruma.1");
        assert_eq!(to_json_value(&version).unwrap(), json!("io.ruma.1"));
    }

    #[test]
    fn deserialize_custom_string() {
        let version = VoipVersionId::from("io.ruma.1");
        assert_eq!(from_json_value::<VoipVersionId>(json!("io.ruma.1")).unwrap(), version);
    }
}
