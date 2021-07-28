//! Types for the *m.secret.request* event.

use ruma_events_macros::EventContent;
use ruma_identifiers::DeviceIdBox;
use ruma_serde::StringEnum;
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};

use crate::ToDeviceEvent;

/// Event sent by a client to request a secret from another device or to cancel a previous request.
///
/// It is sent as an unencrypted to-device event.
pub type SecretRequestEvent = ToDeviceEvent<SecretRequestEventContent>;

/// The payload for SecretRequestEvent.
#[derive(Clone, Debug, Serialize, EventContent)]
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

impl<'de> Deserialize<'de> for SecretRequestEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Name,
            Action,
            RequestingDeviceId,
            RequestId,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter
                            .write_str("`name`, `action`, `requesting_device_id` or `request_id`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "action" => Ok(Field::Action),
                            "requesting_device_id" => Ok(Field::RequestingDeviceId),
                            "request_id" => Ok(Field::RequestId),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SecretRequestEventContentVisitor;

        impl<'de> Visitor<'de> for SecretRequestEventContentVisitor {
            type Value = SecretRequestEventContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct SecretRequestEventContent")
            }

            fn visit_map<V>(self, mut map: V) -> Result<SecretRequestEventContent, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut action: Option<String> = None;
                let mut requesting_device_id: Option<String> = None;
                let mut request_id: Option<String> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }
                            action = Some(map.next_value()?);
                        }
                        Field::RequestingDeviceId => {
                            if requesting_device_id.is_some() {
                                return Err(de::Error::duplicate_field("requesting_device_id"));
                            }
                            requesting_device_id = Some(map.next_value()?);
                        }
                        Field::RequestId => {
                            if request_id.is_some() {
                                return Err(de::Error::duplicate_field("request_id"));
                            }
                            request_id = Some(map.next_value()?);
                        }
                    }
                }

                let request_action = match action {
                    None => {
                        return Err(de::Error::missing_field("action"));
                    }
                    // If the action is "request", bundle the name in the `RequestAction`, else
                    // discard the name.
                    Some(act) => match act.as_ref() {
                        "request" => RequestAction::Request(match name {
                            Some(secret_name) => SecretName::from(secret_name),
                            None => {
                                return Err(de::Error::missing_field("name"));
                            }
                        }),
                        "request_cancellation" => RequestAction::RequestCancellation,
                        other_action => RequestAction::_Custom(other_action.to_owned()),
                    },
                };

                let requesting_device_id: DeviceIdBox = match requesting_device_id {
                    None => {
                        return Err(de::Error::missing_field("requesting_device_id"));
                    }
                    Some(id) => id.into(),
                };

                let request_id = match request_id {
                    None => {
                        return Err(de::Error::missing_field("request_id"));
                    }
                    Some(id) => id,
                };

                Ok(SecretRequestEventContent::new(request_action, requesting_device_id, request_id))
            }
        }

        const FIELDS: &[&str] = &["name", "action", "requesting_device_id", "request_id"];
        deserializer.deserialize_struct(
            "SecretRequestEventContent",
            FIELDS,
            SecretRequestEventContentVisitor,
        )
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

    /// Recovery key (m.megolm_backup.v1).
    #[ruma_enum(rename = "m.megolm_backup.v1")]
    RecoveryKey,

    /// Custom secret name.
    Custom(String),
}

#[cfg(test)]
mod test {
    use super::{RequestAction, SecretName, SecretRequestEventContent};
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

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

    #[test]
    fn secret_request_deserialization() {
        let json = json!({
            "name": "org.example.some.secret",
            "action": "request",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            SecretRequestEventContent {
                action: RequestAction::Request(
                    SecretName::Custom(secret)
                ),
                requesting_device_id,
                request_id,
            }
            if secret == "org.example.some.secret"
            && requesting_device_id == "ABCDEFG"
            && request_id == "randomly_generated_id_9573"
        )
    }

    #[test]
    fn secret_request_cancellation_deserialisation() {
        let json = json!({
            "action": "request_cancellation",
            "requesting_device_id": "ABCDEFG",
            "request_id": "randomly_generated_id_9573"
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            SecretRequestEventContent {
                action: RequestAction::RequestCancellation,
                requesting_device_id,
                request_id,
            }
            if requesting_device_id.as_str() == "ABCDEFG"
            && request_id == "randomly_generated_id_9573"
        )
    }

    #[test]
    fn secret_request_recovery_key_deserialization() {
        let json = json!({
            "name": "m.megolm_backup.v1",
            "action": "request",
            "requesting_device_id": "XYZxyz",
            "request_id": "this_is_a_request_id"
        });

        assert_matches!(
            from_json_value(json).unwrap(),
            SecretRequestEventContent {
                action: RequestAction::Request(
                    SecretName::RecoveryKey
                ),
                requesting_device_id,
                request_id,
            }
            if requesting_device_id == "XYZxyz"
            && request_id == "this_is_a_request_id"
        )
    }
}
