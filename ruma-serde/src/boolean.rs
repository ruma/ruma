//! De-/serialization functions for casting boolean values to int and vise versa.
//! Delegates to `js_int::UInt` to ensure integer size is within bounds.

use js_int::UInt;
use serde::{
    de::{self, Deserialize, Deserializer, Unexpected},
    ser::{Error, Serialize, Serializer},
};
use std::convert::TryFrom;

/// Serialize a boolean to UInt.
pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match UInt::try_from(match value {
        false => 0,
        true => 1,
    }) {
        Ok(uint) => uint.serialize(serializer),
        Err(err) => Err(S::Error::custom(err)),
    }
}

/// Deserializes a boolean from UInt.
///
/// Will fail if integer is greater than 1.
pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u64::from(UInt::deserialize(deserializer)?) {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(Unexpected::Unsigned(other as u64), &"zero or one")),
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct UIntBooleanTest {
        #[serde(with = "super")]
        value: bool,
    }

    #[test]
    fn deserialize_false() {
        let json = json!({ "value": 0 });

        assert_eq!(
            serde_json::from_value::<UIntBooleanTest>(json).unwrap(),
            UIntBooleanTest { value: false },
        );
    }

    #[test]
    fn deserialize_true() {
        let json = json!({ "value": 1 });

        assert_eq!(
            serde_json::from_value::<UIntBooleanTest>(json).unwrap(),
            UIntBooleanTest { value: true },
        );
    }

    #[test]
    fn serialize_false() {
        let request = UIntBooleanTest { value: false };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({ "value": 0 }));
    }

    #[test]
    fn serialize_true() {
        let request = UIntBooleanTest { value: true };
        assert_eq!(serde_json::to_value(&request).unwrap(), json!({ "value": 1 }));
    }
}
