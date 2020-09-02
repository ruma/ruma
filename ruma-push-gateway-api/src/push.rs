pub mod v1;

use std::{collections::BTreeMap, time::SystemTime};

use js_int::UInt;
use ruma_api::Outgoing;
use ruma_common::push::PusherData;
use ruma_events::EventType;
use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;
use strum::{Display, EnumString};

#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[non_exhaustive]
pub struct Notification<'a> {
    /// The event ID of the event being notified about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<&'a EventId>,

    /// The room ID in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<&'a RoomId>,

    /// The value `m.room.member`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<EventType>,

    /// The matrix ID of the user who sent the original `m.room.third_party_invite`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<&'a UserId>,

    /// The current display name of the sender in the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_display_name: Option<&'a str>,

    /// The name of the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<&'a str>,

    /// An alias to display for the room in which the event occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_alias: Option<&'a RoomAliasId>,

    /// This is true if the user receiving the notification is the subject of a member
    /// event (i.e. the state_key of the member event is equal to the user's Matrix ID).
    #[serde(default)]
    pub user_is_target: bool,

    /// The priority of the notification, low or high. If omitted, high is assumed.
    #[serde(default)]
    pub prio: Priority,

    /// The content field from the event, if present.
    ///
    /// To create a `RawJsonValue`, use `serde_json::value::to_raw_value`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Box<RawJsonValue>>,

    /// This is a dictionary of the current number of unacknowledged communications for
    /// the recipient user.
    #[serde(skip_serializing_if = "Counts::is_default")]
    pub counts: Counts,

    /// This is an array of devices that the notification should be sent to.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub devices: &'a [Device],
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Counts {
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub unread: UInt,
    #[serde(skip_serializing_if = "ruma_serde::is_default")]
    pub missed_calls: UInt,
}

impl Counts {
    pub fn new(unread: UInt, missed_calls: UInt) -> Self {
        Self { unread, missed_calls }
    }

    pub fn is_default(&self) -> bool {
        self.missed_calls == js_int::uint!(0) && self.unread == js_int::uint!(0)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum Priority {
    High,
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Self::High
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Device {
    /// The app_id given when the pusher was created.
    ///
    /// Max length, 64 chars.
    pub app_id: String,

    /// The pushkey given when the pusher was created.
    ///
    /// Max length, 512 bytes.
    pub pushkey: String,

    /// The unix timestamp (in seconds) when the pushkey was last updated.
    #[serde(with = "ruma_serde::time::opt_ms_since_unix_epoch")]
    pub pushkey_ts: Option<SystemTime>,

    /// A dictionary of additional pusher-specific data. For 'http' pushers, this is the data
    /// dictionary passed in at pusher creation minus the url key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<PusherData>,

    /// A dictionary of customizations made to the way this notification is to be presented.
    /// These are added by push rules.
    #[serde(with = "tweak")]
    pub tweaks: BTreeMap<String, Tweak>,
}

impl Device {
    pub fn new(app_id: String, pushkey: String) -> Self {
        Self { app_id, pushkey, pushkey_ts: None, data: None, tweaks: BTreeMap::new() }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Tweak {
    Sound(String),
    Highlight(bool),
    Custom(String),
}

mod tweak {
    use std::{collections::BTreeMap, fmt};

    use serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserializer, Serializer,
    };

    use super::Tweak;

    pub fn serialize<S>(tweak: &BTreeMap<String, Tweak>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(tweak.len()))?;
        for (k, item) in tweak {
            match item {
                Tweak::Highlight(b) => map.serialize_entry("highlight", b)?,
                Tweak::Sound(value) => map.serialize_entry("sound", value)?,
                Tweak::Custom(value) => map.serialize_entry(k, value)?,
            }
        }
        map.end()
    }

    struct TweaksVisitor;

    impl<'de> Visitor<'de> for TweaksVisitor {
        type Value = BTreeMap<String, Tweak>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Lazy load options")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut tweaks = BTreeMap::new();
            while let Some(key) = access.next_key::<String>()? {
                match &*key {
                    "sound" => tweaks.insert(key, Tweak::Sound(access.next_value()?)),
                    // If a highlight tweak is given with no value, its value is defined to be true.
                    "highlight" => {
                        let highlight =
                            if let Ok(highlight) = access.next_value() { highlight } else { true };

                        tweaks.insert(key, Tweak::Highlight(highlight))
                    }
                    // TODO should this be an error?
                    _ => tweaks.insert(key, Tweak::Custom(access.next_value()?)),
                };
            }

            // If no highlight tweak is given at all then the value of highlight is defined to be false.
            if !tweaks.contains_key("highlight") {
                tweaks.insert("highlight".into(), Tweak::Highlight(false));
            }

            Ok(tweaks)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BTreeMap<String, Tweak>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TweaksVisitor)
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime};

    use maplit::btreemap;
    use ruma_events::EventType;
    use ruma_identifiers::{event_id, room_alias_id, room_id, user_id};
    use serde_json::{json, to_value as to_json_value};

    use super::{Counts, Device, Notification, Priority, Tweak};

    #[test]
    fn serialize_request() {
        let expected = json!({
            "event_id": "$3957tyerfgewrf384",
            "room_id": "!slw48wfj34rtnrf:example.com",
            "type": "m.room.message",
            "sender": "@exampleuser:matrix.org",
            "sender_display_name": "Major Tom",
            "room_alias": "#exampleroom:matrix.org",
            "user_is_target": false,
            "prio": "low",
            "content": {},
            "counts": {
              "unread": 2,
            },
            "devices": [
              {
                "app_id": "org.matrix.matrixConsole.ios",
                "pushkey": "V2h5IG9uIGVhcnRoIGRpZCB5b3UgZGVjb2RlIHRoaXM/",
                "pushkey_ts": 123,
                "tweaks": {
                  "sound": "silence",
                  "highlight": true,
                  "custom": "go wild"
                }
              }
            ]
        });

        let eid = event_id!("$3957tyerfgewrf384");
        let rid = room_id!("!slw48wfj34rtnrf:example.com");
        let uid = user_id!("@exampleuser:matrix.org");
        let alias = room_alias_id!("#exampleroom:matrix.org");

        let mut count = Counts::default();
        count.unread = js_int::uint!(2);
        // test default values are ignored
        count.missed_calls = js_int::uint!(0);

        let mut device = Device::new(
            "org.matrix.matrixConsole.ios".into(),
            "V2h5IG9uIGVhcnRoIGRpZCB5b3UgZGVjb2RlIHRoaXM/".into(),
        );
        device.pushkey_ts = Some(SystemTime::UNIX_EPOCH + Duration::from_millis(123));
        device.tweaks = btreemap! {
            "highlight".into() => Tweak::Highlight(true),
            "sound".into() => Tweak::Sound("silence".into()),
            "custom".into() => Tweak::Custom("go wild".into()),
        };

        let devices = vec![device];

        let mut notice = Notification::default();
        notice.event_id = Some(&eid);
        notice.room_id = Some(&rid);
        notice.kind = Some(EventType::RoomMessage);
        notice.sender = Some(&uid);
        notice.sender_display_name = Some("Major Tom");
        notice.room_alias = Some(&alias);
        notice.content = Some(serde_json::from_str("{}").unwrap());
        notice.counts = count;
        notice.prio = Priority::Low;
        notice.devices = &devices;

        assert_eq!(expected, to_json_value(notice).unwrap())
    }
}
