use ruma_common::serde::{from_raw_json_value, JsonObject};
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{EmailPusherData, Pusher, PusherIds, PusherKind};

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
            PusherKind::Email(_) => {
                st.serialize_field("kind", &"email")?;
                st.serialize_field("data", &JsonObject::new())?;
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
            "email" => Ok(Self::Email(EmailPusherData)),
            _ => from_raw_json_value(&json).map(Self::_Custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{push::HttpPusherData, serde::JsonObject};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::push::{CustomPusherData, EmailPusherData, PusherKind};

    #[test]
    fn serialize_email() {
        let action = PusherKind::Email(EmailPusherData::new());

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "email",
                "data": {},
            })
        );
    }

    #[test]
    fn serialize_http() {
        let action = PusherKind::Http(HttpPusherData::new("http://localhost".to_owned()));

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "kind": "http",
                "data": {
                    "url": "http://localhost",
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
        let json = json!({
            "kind": "email",
            "data": {},
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Email(_));
    }

    #[test]
    fn deserialize_http() {
        let json = json!({
            "kind": "http",
            "data": {
                "url": "http://localhost",
            },
        });

        assert_matches!(from_json_value(json).unwrap(), PusherKind::Http(data));
        assert_eq!(data.url, "http://localhost");
        assert_eq!(data.format, None);
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
