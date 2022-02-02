use ruma_serde::from_raw_json_value;
use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use super::LoginType;

/// Helper struct to determine the type from a `serde_json::value::RawValue`
#[derive(Debug, Deserialize)]
struct LoginTypeDeHelper {
    /// The login type field
    #[serde(rename = "type")]
    type_: String,
}

impl<'de> Deserialize<'de> for LoginType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let LoginTypeDeHelper { type_ } = from_raw_json_value(&json)?;

        Ok(match type_.as_ref() {
            "m.login.password" => Self::Password(from_raw_json_value(&json)?),
            "m.login.token" => Self::Token(from_raw_json_value(&json)?),
            "m.login.sso" => Self::Sso(from_raw_json_value(&json)?),
            _ => Self::_Custom(from_raw_json_value(&json)?),
        })
    }
}
