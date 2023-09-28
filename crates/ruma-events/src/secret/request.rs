//! Types for the [`m.secret.request`] event.
//!
//! [`m.secret.request`]: https://spec.matrix.org/latest/client-server-api/#msecretrequest

use ruma_common::{serde::StringEnum, OwnedDeviceId, OwnedTransactionId};
use ruma_macros::EventContent;
use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::{GlobalAccountDataEventType, PrivOwnedStr};

/// The content of an `m.secret.request` event.
///
/// Event sent by a client to request a secret from another device or to cancel a previous request.
///
/// It is sent as an unencrypted to-device event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret.request", kind = ToDevice)]
pub struct ToDeviceSecretRequestEventContent {
    /// The action for the request.
    #[serde(flatten)]
    pub action: RequestAction,

    /// The ID of the device requesting the event.
    pub requesting_device_id: OwnedDeviceId,

    /// A random string uniquely identifying (with respect to the requester and the target) the
    /// target for a secret.
    ///
    /// If the secret is requested from multiple devices at the same time, the same ID may be used
    /// for every target. The same ID is also used in order to cancel a previous request.
    pub request_id: OwnedTransactionId,
}

impl ToDeviceSecretRequestEventContent {
    /// Creates a new `ToDeviceRequestEventContent` with the given action, requesting device ID and
    /// request ID.
    pub fn new(
        action: RequestAction,
        requesting_device_id: OwnedDeviceId,
        request_id: OwnedTransactionId,
    ) -> Self {
        Self { action, requesting_device_id, request_id }
    }
}

/// Action for an `m.secret.request` event.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(try_from = "RequestActionJsonRepr")]
pub enum RequestAction {
    /// Request a secret by its name.
    Request(SecretName),

    /// Cancel a request for a secret.
    RequestCancellation,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Serialize for RequestAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = serializer.serialize_struct("request_action", 2)?;

        match self {
            Self::Request(name) => {
                st.serialize_field("name", name)?;
                st.serialize_field("action", "request")?;
                st.end()
            }
            Self::RequestCancellation => {
                st.serialize_field("action", "request_cancellation")?;
                st.end()
            }
            RequestAction::_Custom(custom) => {
                st.serialize_field("action", &custom.0)?;
                st.end()
            }
        }
    }
}

#[derive(Deserialize)]
struct RequestActionJsonRepr {
    action: String,
    name: Option<SecretName>,
}

impl TryFrom<RequestActionJsonRepr> for RequestAction {
    type Error = &'static str;

    fn try_from(value: RequestActionJsonRepr) -> Result<Self, Self::Error> {
        match value.action.as_str() {
            "request" => {
                if let Some(name) = value.name {
                    Ok(RequestAction::Request(name))
                } else {
                    Err("A secret name is required when the action is \"request\".")
                }
            }
            "request_cancellation" => Ok(RequestAction::RequestCancellation),
            _ => Ok(RequestAction::_Custom(PrivOwnedStr(value.action.into()))),
        }
    }
}

/// The name of a secret.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum SecretName {
    /// Cross-signing master key (m.cross_signing.master).
    #[ruma_enum(rename = "m.cross_signing.master")]
    CrossSigningMasterKey,

    /// Cross-signing user-signing key (m.cross_signing.user_signing).
    #[ruma_enum(rename = "m.cross_signing.user_signing")]
    CrossSigningUserSigningKey,

    /// Cross-signing self-signing key (m.cross_signing.self_signing).
    #[ruma_enum(rename = "m.cross_signing.self_signing")]
    CrossSigningSelfSigningKey,

    /// Recovery key (m.megolm_backup.v1).
    #[ruma_enum(rename = "m.megolm_backup.v1")]
    RecoveryKey,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl From<SecretName> for GlobalAccountDataEventType {
    fn from(value: SecretName) -> Self {
        GlobalAccountDataEventType::from(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{RequestAction, SecretName, ToDeviceSecretRequestEventContent};
    use crate::PrivOwnedStr;

    #[test]
    fn secret_request_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::Request("org.example.some.secret".into()),
            "ABCDEFG".into(),
            "randomly_generated_id_9573".into(),
        );

        let json = json!({
            "name": "org.example.some.secret",
            "action": "request",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn secret_request_recovery_key_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::Request(SecretName::RecoveryKey),
            "XYZxyz".into(),
            "this_is_a_request_id".into(),
        );

        let json = json!({
            "name": "m.megolm_backup.v1",
            "action": "request",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn secret_custom_action_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::_Custom(PrivOwnedStr("my_custom_action".into())),
            "XYZxyz".into(),
            "this_is_a_request_id".into(),
        );

        let json = json!({
            "action": "my_custom_action",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn secret_request_cancellation_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::RequestCancellation,
            "ABCDEFG".into(),
            "randomly_generated_id_9573".into(),
        );

        let json = json!({
            "action": "request_cancellation",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn secret_request_deserialization() {
        let json = json!({
            "name": "org.example.some.secret",
            "action": "request",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        let content = from_json_value::<ToDeviceSecretRequestEventContent>(json).unwrap();
        assert_eq!(content.requesting_device_id, "ABCDEFG");
        assert_eq!(content.request_id, "randomly_generated_id_9573");
        assert_matches!(content.action, RequestAction::Request(secret));
        assert_eq!(secret.as_str(), "org.example.some.secret");
    }

    #[test]
    fn secret_request_cancellation_deserialization() {
        let json = json!({
            "action": "request_cancellation",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        let content = from_json_value::<ToDeviceSecretRequestEventContent>(json).unwrap();
        assert_eq!(content.requesting_device_id, "ABCDEFG");
        assert_eq!(content.request_id, "randomly_generated_id_9573");
        assert_matches!(content.action, RequestAction::RequestCancellation);
    }

    #[test]
    fn secret_request_recovery_key_deserialization() {
        let json = json!({
            "name": "m.megolm_backup.v1",
            "action": "request",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        let content = from_json_value::<ToDeviceSecretRequestEventContent>(json).unwrap();
        assert_eq!(content.requesting_device_id, "XYZxyz");
        assert_eq!(content.request_id, "this_is_a_request_id");
        assert_matches!(content.action, RequestAction::Request(secret));
        assert_eq!(secret, SecretName::RecoveryKey);
    }

    #[test]
    fn secret_custom_action_deserialization() {
        let json = json!({
            "action": "my_custom_action",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        let content = from_json_value::<ToDeviceSecretRequestEventContent>(json).unwrap();
        assert_eq!(content.requesting_device_id, "XYZxyz");
        assert_eq!(content.request_id, "this_is_a_request_id");
        assert_eq!(content.action, RequestAction::_Custom(PrivOwnedStr("my_custom_action".into())));
    }
}
