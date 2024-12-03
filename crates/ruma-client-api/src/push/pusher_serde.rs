use ruma_common::serde::from_raw_json_value;
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{Pusher, PusherIds, PusherKind};

#[derive(Debug, Deserialize)]
struct PusherDeHelper {
    #[serde(flatten)]
    ids: PusherIds,
    app_display_name: String,
    device_display_name: String,
    profile_tag: Option<String>,
    lang: String,
}

impl<'de> Deserialize<'de> for Pusher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let PusherDeHelper { ids, app_display_name, device_display_name, profile_tag, lang } =
            from_raw_json_value(&json)?;
        let kind = from_raw_json_value(&json)?;

        Ok(Self { ids, kind, app_display_name, device_display_name, profile_tag, lang })
    }
}

impl Serialize for PusherKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut st = serializer.serialize_struct("PusherAction", 3)?;

        match self {
            PusherKind::Http(data) => {
                st.serialize_field("kind", &"http")?;
                st.serialize_field("data", data)?;
            }
            PusherKind::Email(data) => {
                st.serialize_field("kind", &"email")?;
                st.serialize_field("data", data)?;
            }
            PusherKind::_Custom(custom) => {
                st.serialize_field("kind", &custom.kind)?;
                st.serialize_field("data", &custom.data)?;
            }
        }

        st.end()
    }
}

#[derive(Debug, Deserialize)]
struct PusherKindDeHelper {
    kind: String,
    data: Box<RawJsonValue>,
}

impl<'de> Deserialize<'de> for PusherKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let PusherKindDeHelper { kind, data } = from_raw_json_value(&json)?;

        match kind.as_ref() {
            "http" => from_raw_json_value(&data).map(Self::Http),
            "email" => from_raw_json_value(&data).map(Self::Email),
            _ => from_raw_json_value(&json).map(Self::_Custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{push::HttpPusherData, serde::JsonObject};
    use serde_json::{
        from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
    };

    use crate::push::{CustomPusherData, EmailPusherData, PusherKind};

    #[test]
    fn serialize_email() {
        // With default data fields.
        let mut data = EmailPusherData::new();
        let action = PusherKind::Email(data.clone());

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "email",
                "data": {},
            })
        );

        // With custom data fields.
        data.data.insert("custom_key".to_owned(), "value".into());
        let action = PusherKind::Email(data);

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "email",
                "data": {
                    "custom_key": "value",
                },
            })
        );
    }

    #[test]
    fn serialize_http() {
        // With default data fields.
        let mut data = HttpPusherData::new("http://localhost".to_owned());
        let action = PusherKind::Http(data.clone());

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "http",
                "data": {
                    "url": "http://localhost",
                },
            })
        );

        // With custom data fields.
        data.data.insert("custom_key".to_owned(), "value".into());
        let action = PusherKind::Http(data);

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "http",
                "data": {
                    "url": "http://localhost",
                    "custom_key": "value",
                },
            })
        );
    }

    #[test]
    fn serialize_custom() {
        let action = PusherKind::_Custom(CustomPusherData {
            kind: "my.custom.kind".to_owned(),
            data: JsonObject::new(),
        });

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "my.custom.kind",
                "data": {}
            })
        );
    }

    #[test]
    fn deserialize_email() {
        // With default data fields.
        let json = json!({
            "kind": "email",
            "data": {},
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Email(data));
        assert!(data.data.is_empty());

        // With custom data fields.
        let json = json!({
            "kind": "email",
            "data": {
                "custom_key": "value",
            },
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Email(data));
        assert_eq!(data.data.len(), 1);
        assert_matches!(data.data.get("custom_key"), Some(JsonValue::String(custom_value)));
        assert_eq!(custom_value, "value");
    }

    #[test]
    fn deserialize_http() {
        // With default data fields.
        let json = json!({
            "kind": "http",
            "data": {
                "url": "http://localhost",
            },
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Http(data));
        assert_eq!(data.url, "http://localhost");
        assert_eq!(data.format, None);
        assert!(data.data.is_empty());

        // With custom data fields.
        let json = json!({
            "kind": "http",
            "data": {
                "url": "http://localhost",
                "custom_key": "value",
            },
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Http(data));
        assert_eq!(data.data.len(), 1);
        assert_matches!(data.data.get("custom_key"), Some(JsonValue::String(custom_value)));
        assert_eq!(custom_value, "value");
    }

    #[test]
    fn deserialize_custom() {
        let json = json!({
            "kind": "my.custom.kind",
            "data": {}
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::_Custom(custom));
        assert_eq!(custom.kind, "my.custom.kind");
        assert!(custom.data.is_empty());
    }
}
