//! Types for the *m.secret.request* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use ruma_serde::StringEnum;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::ToDeviceEvent;

/// Event sent by a client to request a secret from another device or to cancel a previous request.
///
/// It is sent as an unencrypted to-device event.
pub type SecretRequestEvent = ToDeviceEvent<SecretRequestEventContent>;

/// The payload for SecretRequestEvent.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.secret.request", kind = ToDevice)]
pub struct SecretRequestEventContent {
    /// The action for the request, one of `["request", "request_cancellation"]`.
    ///
    /// If the action is "request", the name of the secret must also be provided.
    #[serde(flatten)]
    pub action: RequestAction,

    /// The ID of the device requesting the event.
    pub requesting_device_id: DeviceIdBox,

    /// A random string uniquely identifying (with respect to the requester and the target) the
    /// target for a secret.
    ///
    /// If the secret is requested from multiple devices at the same time, the same ID may be used
    /// for every target. The same ID is also used in order to cancel a previous request.
    pub request_id: String,
}

impl SecretRequestEventContent {
    /// Creates a new `SecretRequestEventContent` with the given action, requesting device ID and
    /// request ID.
    pub fn new(
        action: RequestAction,
        requesting_device_id: DeviceIdBox,
        request_id: String,
    ) -> Self {
        Self { action, requesting_device_id, request_id }
    }
}

/// Action for a *m.secret.request* event.
#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RequestAction {
    /// Request a secret by its name.
    Request(SecretName),

    /// Cancel a request for a secret.
    RequestCancellation,

    #[doc(hidden)]
    _Custom(String),
}

impl Serialize for RequestAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Request(name) => {
                let mut st = serializer.serialize_map(Some(2))?;
                st.serialize_entry("name", name)?;
                st.serialize_entry("action", "request")?;
                st.end()
            }
            Self::RequestCancellation => {
                let mut st = serializer.serialize_map(Some(1))?;
                st.serialize_entry("action", "request_cancellation")?;
                st.end()
            }
            RequestAction::_Custom(custom) => serializer.serialize_str(custom),
        }
    }
}

/// The name of a secret.
#[derive(Clone, Debug, StringEnum)]
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

    /// Recovery key (m.megolm_backup.v1.recovery_key).
    #[ruma_enum(rename = "m.megolm_backup.v1.recovery_key")]
    RecoveryKey,

    /// Custom secret name.
    Custom(String),
}

#[cfg(test)]
mod test {
    use super::{RequestAction, SecretName, SecretRequestEventContent};
    use serde_json::{json, to_value as to_json_value};

    #[test]
    fn secret_request_serialization() {
        let content = SecretRequestEventContent::new(
            RequestAction::Request(SecretName::Custom("org.example.some.secret".into())),
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
        let content = SecretRequestEventContent::new(
            RequestAction::Request(SecretName::RecoveryKey),
            "XYZxyz".into(),
            "this_is_a_device_id".into(),
        );

        let json = json!({
            "name": "m.megolm_backup.v1.recovery_key",
            "action": "request",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_device_id"
        });

        assert_eq!(to_json_value(&content).unwrap(), json);
    }

    #[test]
    fn secret_request_cancellation_serialization() {
        let content = SecretRequestEventContent::new(
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
}
