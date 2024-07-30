//! `POST /_matrix/push/*/notify`
//!
//! Notify a push gateway about an event or update the number of unread notifications a user has.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/push-gateway-api/#post_matrixpushv1notify

    use js_int::{uint, UInt};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        push::{PushFormat, Tweak},
        serde::StringEnum,
        OwnedEventId, OwnedRoomAliasId, OwnedRoomId, OwnedUserId, SecondsSinceUnixEpoch,
    };
    use ruma_events::TimelineEventType;
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;
    #[cfg(feature = "unstable-unspecified")]
    use serde_json::value::Value as JsonValue;

    use crate::PrivOwnedStr;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/push/v1/notify",
        }
    };

    /// Request type for the `send_event_notification` endpoint.
    #[request]
    pub struct Request {
        /// Information about the push notification
        pub notification: Notification,
    }

    /// Response type for the `send_event_notification` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// A list of all pushkeys given in the notification request that are not valid.
        ///
        /// These could have been rejected by an upstream gateway because they have expired or have
        /// never been valid. Homeservers must cease sending notification requests for these
        /// pushkeys and remove the associated pushers. It may not necessarily be the notification
        /// in the request that failed: it could be that a previous notification to the same
        /// pushkey failed. May be empty.
        pub rejected: Vec<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given notification.
        pub fn new(notification: Notification) -> Self {
            Self { notification }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given list of rejected pushkeys.
        pub fn new(rejected: Vec<String>) -> Self {
            Self { rejected }
        }
    }

    /// Type for passing information about a push notification
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Notification {
        /// The Matrix event ID of the event being notified about.
        ///
        /// Required if the notification is about a particular Matrix event. May be omitted for
        /// notifications that only contain updated badge counts. This ID can and should be used to
        /// detect duplicate notification requests.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub event_id: Option<OwnedEventId>,

        /// The ID of the room in which this event occurred.
        ///
        /// Required if the notification relates to a specific Matrix event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_id: Option<OwnedRoomId>,

        /// The type of the event as in the event's `type` field.
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        pub event_type: Option<TimelineEventType>,

        /// The sender of the event as in the corresponding event field.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender: Option<OwnedUserId>,

        /// The current display name of the sender in the room in which the event occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender_display_name: Option<String>,

        /// The name of the room in which the event occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_name: Option<String>,

        /// An alias to display for the room in which the event occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias: Option<OwnedRoomAliasId>,

        /// Whether the user receiving the notification is the subject of a member event (i.e. the
        /// `state_key` of the member event is equal to the user's Matrix ID).
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub user_is_target: bool,

        /// The priority of the notification.
        ///
        /// If omitted, `high` is assumed. This may be used by push gateways to deliver less
        /// time-sensitive notifications in a way that will preserve battery power on mobile
        /// devices.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub prio: NotificationPriority,

        /// The `content` field from the event, if present.
        ///
        /// The pusher may omit this if the event had no content or for any other reason.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<Box<RawJsonValue>>,

        /// Current number of unacknowledged communications for the recipient user.
        ///
        /// Counts whose value is zero should be omitted.
        #[serde(default, skip_serializing_if = "NotificationCounts::is_default")]
        pub counts: NotificationCounts,

        /// An array of devices that the notification should be sent to.
        pub devices: Vec<Device>,
    }

    impl Notification {
        /// Create a new notification for the given devices.
        pub fn new(devices: Vec<Device>) -> Self {
            Notification { devices, ..Default::default() }
        }
    }

    /// Type for passing information about notification priority.
    ///
    /// This may be used by push gateways to deliver less time-sensitive
    /// notifications in a way that will preserve battery power on mobile devices.
    ///
    /// This type can hold an arbitrary string. To build this with a custom value, convert it from a
    /// string with `::from()` / `.into()`. To check for values that are not available as a
    /// documented variant here, use its string representation, obtained through `.as_str()`.
    #[derive(Clone, Default, PartialEq, Eq, StringEnum)]
    #[ruma_enum(rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum NotificationPriority {
        /// A high priority notification
        #[default]
        High,

        /// A low priority notification
        Low,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// Type for passing information about notification counts.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct NotificationCounts {
        /// The number of unread messages a user has across all of the rooms they
        /// are a member of.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub unread: UInt,

        /// The number of unacknowledged missed calls a user has across all rooms of
        /// which they are a member.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub missed_calls: UInt,
    }

    impl NotificationCounts {
        /// Create new notification counts from the given unread and missed call
        /// counts.
        pub fn new(unread: UInt, missed_calls: UInt) -> Self {
            NotificationCounts { unread, missed_calls }
        }

        fn is_default(&self) -> bool {
            self.unread == uint!(0) && self.missed_calls == uint!(0)
        }
    }

    /// Type for passing information about devices.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Device {
        /// The `app_id` given when the pusher was created.
        ///
        /// Max length: 64 chars.
        pub app_id: String,

        /// The `pushkey` given when the pusher was created.
        ///
        /// Max length: 512 bytes.
        pub pushkey: String,

        /// The unix timestamp (in seconds) when the pushkey was last updated.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pushkey_ts: Option<SecondsSinceUnixEpoch>,

        /// A dictionary of additional pusher-specific data.
        #[serde(default, skip_serializing_if = "PusherData::is_empty")]
        pub data: PusherData,

        /// A dictionary of customisations made to the way this notification is to be presented.
        ///
        /// These are added by push rules.
        #[serde(with = "tweak_serde", skip_serializing_if = "Vec::is_empty")]
        pub tweaks: Vec<Tweak>,
    }

    impl Device {
        /// Create a new device with the given app id and pushkey
        pub fn new(app_id: String, pushkey: String) -> Self {
            Device {
                app_id,
                pushkey,
                pushkey_ts: None,
                data: PusherData::new(),
                tweaks: Vec::new(),
            }
        }
    }

    /// Information for the pusher implementation itself.
    ///
    /// This is the data dictionary passed in at pusher creation minus the `url` key.
    ///
    /// It can be constructed from [`ruma_common::push::HttpPusherData`] with `::from()` /
    /// `.into()`.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct PusherData {
        /// The format to use when sending notifications to the Push Gateway.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub format: Option<PushFormat>,

        /// iOS (+ macOS?) specific default payload that will be sent to apple push notification
        /// service.
        ///
        /// For more information, see [Sygnal docs][sygnal].
        ///
        /// [sygnal]: https://github.com/matrix-org/sygnal/blob/main/docs/applications.md#ios-applications-beware
        // Not specified, issue: https://github.com/matrix-org/matrix-spec/issues/921
        #[cfg(feature = "unstable-unspecified")]
        #[serde(default, skip_serializing_if = "JsonValue::is_null")]
        pub default_payload: JsonValue,
    }

    impl PusherData {
        /// Creates an empty `PusherData`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are `None`.
        pub fn is_empty(&self) -> bool {
            #[cfg(not(feature = "unstable-unspecified"))]
            {
                self.format.is_none()
            }

            #[cfg(feature = "unstable-unspecified")]
            {
                self.format.is_none() && self.default_payload.is_null()
            }
        }
    }

    impl From<ruma_common::push::HttpPusherData> for PusherData {
        fn from(data: ruma_common::push::HttpPusherData) -> Self {
            let ruma_common::push::HttpPusherData {
                format,
                #[cfg(feature = "unstable-unspecified")]
                default_payload,
                ..
            } = data;

            Self {
                format,
                #[cfg(feature = "unstable-unspecified")]
                default_payload,
            }
        }
    }

    mod tweak_serde {
        use std::fmt;

        use ruma_common::push::Tweak;
        use serde::{
            de::{MapAccess, Visitor},
            ser::SerializeMap,
            Deserializer, Serializer,
        };

        pub(super) fn serialize<S>(tweak: &[Tweak], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(tweak.len()))?;
            for item in tweak {
                #[allow(unreachable_patterns)]
                match item {
                    Tweak::Highlight(b) => map.serialize_entry("highlight", b)?,
                    Tweak::Sound(value) => map.serialize_entry("sound", value)?,
                    Tweak::Custom { value, name } => map.serialize_entry(name, value)?,
                    _ => unreachable!("variant added to Tweak not covered by Custom"),
                }
            }
            map.end()
        }

        struct TweaksVisitor;

        impl<'de> Visitor<'de> for TweaksVisitor {
            type Value = Vec<Tweak>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("List of tweaks")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut tweaks = vec![];
                while let Some(key) = access.next_key::<String>()? {
                    match &*key {
                        "sound" => tweaks.push(Tweak::Sound(access.next_value()?)),
                        // If a highlight tweak is given with no value, its value is defined to be
                        // true.
                        "highlight" => {
                            let highlight = access.next_value().unwrap_or(true);

                            tweaks.push(Tweak::Highlight(highlight));
                        }
                        _ => tweaks.push(Tweak::Custom { name: key, value: access.next_value()? }),
                    };
                }

                // If no highlight tweak is given at all then the value of highlight is defined to
                // be false.
                if !tweaks.iter().any(|tw| matches!(tw, Tweak::Highlight(_))) {
                    tweaks.push(Tweak::Highlight(false));
                }

                Ok(tweaks)
            }
        }

        pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Tweak>, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(TweaksVisitor)
        }
    }

    #[cfg(test)]
    mod tests {
        use js_int::uint;
        use ruma_common::{
            owned_event_id, owned_room_alias_id, owned_room_id, owned_user_id,
            SecondsSinceUnixEpoch,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{
            from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
        };

        use super::{Device, Notification, NotificationCounts, NotificationPriority, Tweak};

        #[test]
        fn serialize_request() {
            let expected = json!({
                "event_id": "$3957tyerfgewrf384",
                "room_id": "!slw48wfj34rtnrf:example.com",
                "type": "m.room.message",
                "sender": "@exampleuser:matrix.org",
                "sender_display_name": "Major Tom",
                "room_alias": "#exampleroom:matrix.org",
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

            let eid = owned_event_id!("$3957tyerfgewrf384");
            let rid = owned_room_id!("!slw48wfj34rtnrf:example.com");
            let uid = owned_user_id!("@exampleuser:matrix.org");
            let alias = owned_room_alias_id!("#exampleroom:matrix.org");

            let count = NotificationCounts { unread: uint!(2), ..NotificationCounts::default() };

            let device = Device {
                pushkey_ts: Some(SecondsSinceUnixEpoch(uint!(123))),
                tweaks: vec![
                    Tweak::Highlight(true),
                    Tweak::Sound("silence".into()),
                    Tweak::Custom {
                        name: "custom".into(),
                        value: from_json_value(JsonValue::String("go wild".into())).unwrap(),
                    },
                ],
                ..Device::new(
                    "org.matrix.matrixConsole.ios".into(),
                    "V2h5IG9uIGVhcnRoIGRpZCB5b3UgZGVjb2RlIHRoaXM/".into(),
                )
            };
            let devices = vec![device];

            let notice = Notification {
                event_id: Some(eid),
                room_id: Some(rid),
                event_type: Some(TimelineEventType::RoomMessage),
                sender: Some(uid),
                sender_display_name: Some("Major Tom".to_owned()),
                room_alias: Some(alias),
                content: Some(serde_json::from_str("{}").unwrap()),
                counts: count,
                prio: NotificationPriority::Low,
                devices,
                ..Notification::default()
            };

            assert_eq!(expected, to_json_value(notice).unwrap());
        }
    }
}
