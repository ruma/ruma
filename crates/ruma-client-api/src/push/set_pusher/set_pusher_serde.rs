use js_option::JsOption;
use ruma_common::serde::from_raw_json_value;
use serde::{de, ser::SerializeStruct, Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use super::v3::{PusherAction, PusherPostData};

#[derive(Debug, Deserialize)]
struct PusherPostDataDeHelper {
    #[serde(default)]
    append: bool,
}

impl<'de> Deserialize<'de> for PusherPostData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        let PusherPostDataDeHelper { append } = from_raw_json_value(&json)?;
        let pusher = from_raw_json_value(&json)?;

        Ok(Self { pusher, append })
    }
}

impl Serialize for PusherAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PusherAction::Post(pusher) => pusher.serialize(serializer),
            PusherAction::Delete(ids) => {
                let mut st = serializer.serialize_struct("PusherAction", 3)?;
                st.serialize_field("pushkey", &ids.pushkey)?;
                st.serialize_field("app_id", &ids.app_id)?;
                st.serialize_field("kind", &None::<&str>)?;
                st.end()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct PusherActionDeHelper {
    kind: JsOption<String>,
}

impl<'de> Deserialize<'de> for PusherAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let PusherActionDeHelper { kind } = from_raw_json_value(&json)?;

        match kind {
            JsOption::Some(_) => Ok(Self::Post(from_raw_json_value(&json)?)),
            JsOption::Null => Ok(Self::Delete(from_raw_json_value(&json)?)),
            // This is unreachable because we don't use `#[serde(default)]` on the field.
            JsOption::Undefined => Err(de::Error::missing_field("kind")),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::PusherAction;
    use crate::push::{
        set_pusher::v3::PusherPostData, EmailPusherData, Pusher, PusherIds, PusherKind,
    };

    #[test]
    fn serialize_post() {
        let action = PusherAction::Post(PusherPostData {
            pusher: Pusher {
                ids: PusherIds::new("abcdef".to_owned(), "my.matrix.app".to_owned()),
                kind: PusherKind::Email(EmailPusherData::new()),
                app_display_name: "My Matrix App".to_owned(),
                device_display_name: "My Phone".to_owned(),
                profile_tag: None,
                lang: "en".to_owned(),
            },
            append: false,
        });

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "pushkey": "abcdef",
                "app_id": "my.matrix.app",
                "kind": "email",
                "app_display_name": "My Matrix App",
                "device_display_name": "My Phone",
                "lang": "en",
                "data": {}
            })
        );
    }

    #[test]
    fn serialize_delete() {
        let action =
            PusherAction::Delete(PusherIds::new("abcdef".to_owned(), "my.matrix.app".to_owned()));

        assert_eq!(
            to_json_value(action).unwrap(),
            json!({
                "pushkey": "abcdef",
                "app_id": "my.matrix.app",
                "kind": null,
            })
        );
    }

    #[test]
    fn deserialize_post() {
        let json = json!({
            "pushkey": "abcdef",
            "app_id": "my.matrix.app",
            "kind": "email",
            "app_display_name": "My Matrix App",
            "device_display_name": "My Phone",
            "lang": "en",
            "data": {}
        });

        assert_matches!(from_json_value(json).unwrap(), PusherAction::Post(post_data));
        assert!(!post_data.append);

        let pusher = post_data.pusher;
        assert_eq!(pusher.ids.pushkey, "abcdef");
        assert_eq!(pusher.ids.app_id, "my.matrix.app");
        assert_matches!(pusher.kind, PusherKind::Email(_));
        assert_eq!(pusher.app_display_name, "My Matrix App");
        assert_eq!(pusher.device_display_name, "My Phone");
        assert_eq!(pusher.profile_tag, None);
        assert_eq!(pusher.lang, "en");
    }

    #[test]
    fn deserialize_delete() {
        let json = json!({
            "pushkey": "abcdef",
            "app_id": "my.matrix.app",
            "kind": null,
        });

        assert_matches!(from_json_value(json).unwrap(), PusherAction::Delete(ids));
        assert_eq!(ids.pushkey, "abcdef");
        assert_eq!(ids.app_id, "my.matrix.app");
    }
}
