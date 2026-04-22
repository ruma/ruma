//! Types for the [`m.secret.request`] event.
//!
//! [`m.secret.request`]: https://spec.matrix.org/v1.18/client-server-api/#msecretrequest

use as_variant::as_variant;
use ruma_common::{
    OwnedDeviceId, OwnedTransactionId,
    serde::{JsonObject, StringEnum},
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize, de};
use serde_json::{Value as JsonValue, from_value as from_json_value};

use crate::{GlobalAccountDataEventType, PrivOwnedStr};

/// The content of an `m.secret.request` event.
///
/// Event sent by a client to request a secret from another device or to cancel a previous request.
///
/// It is sent as an unencrypted to-device event.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RequestAction {
    /// Request a secret by its name.
    Request(SecretRequestAction),

    /// Cancel a request for a secret.
    RequestCancellation,

    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomRequestAction),
}

impl RequestAction {
    /// Access the `action` field of this action.
    pub fn action(&self) -> &str {
        match self {
            Self::Request(_) => "request",
            Self::RequestCancellation => "request_cancellation",
            Self::_Custom(custom) => &custom.action,
        }
    }
}

impl<'de> Deserialize<'de> for RequestAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut json = JsonObject::deserialize(deserializer)?;

        let action = json
            .remove("action")
            .and_then(|value| as_variant!(value, JsonValue::String))
            .ok_or_else(|| de::Error::missing_field("action"))?;

        match action.as_ref() {
            "request" => from_json_value(json.into()).map(Self::Request),
            "request_cancellation" => Ok(Self::RequestCancellation),
            _ => Ok(Self::_Custom(CustomRequestAction { action })),
        }
        .map_err(de::Error::custom)
    }
}

impl From<SecretRequestAction> for RequestAction {
    fn from(value: SecretRequestAction) -> Self {
        Self::Request(value)
    }
}

/// Details about a secret to request in a [`RequestAction`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct SecretRequestAction {
    /// The name of the requested secret.
    pub name: SecretName,
}

impl SecretRequestAction {
    /// Construct a new `SecretRequestAction` for the given secret name.
    pub fn new(name: SecretName) -> Self {
        Self { name }
    }
}

/// The name of a secret.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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

/// A custom [`RequestAction`].
#[doc(hidden)]
#[derive(Clone, Debug, Serialize)]
pub struct CustomRequestAction {
    /// The action of the request.
    action: String,
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::{
        RequestAction, SecretName, SecretRequestAction, ToDeviceSecretRequestEventContent,
    };

    #[test]
    fn secret_request_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::Request(SecretRequestAction::new("org.example.some.secret".into())),
            "ABCDEFG".into(),
            "randomly_generated_id_9573".into(),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "name": "org.example.some.secret",
                "action": "request",
                "requesting_device_id": "ABCDEFG",
                "request_id": "randomly_generated_id_9573",
            }),
        );
    }

    #[test]
    fn secret_request_recovery_key_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::Request(SecretRequestAction::new(SecretName::RecoveryKey)),
            "XYZxyz".into(),
            "this_is_a_request_id".into(),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "name": "m.megolm_backup.v1",
                "action": "request",
                "requesting_device_id": "XYZxyz",
                "request_id": "this_is_a_request_id",
            }),
        );
    }

    #[test]
    fn secret_request_cancellation_serialization() {
        let content = ToDeviceSecretRequestEventContent::new(
            RequestAction::RequestCancellation,
            "ABCDEFG".into(),
            "randomly_generated_id_9573".into(),
        );

        assert_to_canonical_json_eq!(
            content,
            json!({
                "action": "request_cancellation",
                "requesting_device_id": "ABCDEFG",
                "request_id": "randomly_generated_id_9573",
            }),
        );
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
        assert_eq!(secret.name.as_str(), "org.example.some.secret");
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
        assert_eq!(secret.name, SecretName::RecoveryKey);
    }

    #[test]
    fn secret_custom_action_serialization_roundtrip() {
        let json = json!({
            "action": "my_custom_action",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        let content = from_json_value::<ToDeviceSecretRequestEventContent>(json.clone()).unwrap();
        assert_eq!(content.requesting_device_id, "XYZxyz");
        assert_eq!(content.request_id, "this_is_a_request_id");
        assert_eq!(content.action.action(), "my_custom_action");

        assert_to_canonical_json_eq!(content, json);
    }
}
