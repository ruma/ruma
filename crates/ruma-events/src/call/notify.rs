//! Type for the MatrixRTC notify event ([MSC4075]).
//!
//! [MSC4075]: https://github.com/matrix-org/matrix-spec-proposals/pull/4075

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::member::Application;
use crate::Mentions;

/// The content of an `m.call.notify` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.notify", kind = MessageLike)]
pub struct CallNotifyEventContent {
    /// A unique identifier for the call.
    pub call_id: String,

    /// The application this notify event applies to.
    pub application: ApplicationType,

    /// How this notify event should notify the receiver.
    pub notify_type: NotifyType,

    /// The users that are notified by this event (See [MSC3952] (Intentional Mentions)).
    ///
    /// [MSC3952]: https://github.com/matrix-org/matrix-spec-proposals/pull/3952
    #[serde(rename = "m.mentions")]
    pub mentions: Mentions,
}

impl CallNotifyEventContent {
    /// Creates a new `CallNotifyEventContent` with the given configuration.
    pub fn new(
        call_id: String,
        application: ApplicationType,
        notify_type: NotifyType,
        mentions: Mentions,
    ) -> Self {
        Self { call_id, application, notify_type, mentions }
    }
}

/// How this notify event should notify the receiver.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotifyType {
    /// The receiving client should ring with an audible sound.
    #[serde(rename = "ring")]
    Ring,

    /// The receiving client should display a visual notification.
    #[serde(rename = "notify")]
    Notify,
}

/// The type of matrix RTC application.
///
/// This is different to [`Application`] because application contains all the information from the
/// `m.call.member` event.
///
/// An `Application` can be converted into an `ApplicationType` using `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum ApplicationType {
    /// A VoIP call.
    #[serde(rename = "m.call")]
    Call,
}

impl From<Application> for ApplicationType {
    fn from(val: Application) -> Self {
        match val {
            Application::Call(_) => ApplicationType::Call,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use crate::{
        call::notify::{ApplicationType, CallNotifyEventContent, NotifyType},
        Mentions,
    };

    #[test]
    fn notify_event_serialization() {
        use ruma_common::owned_user_id;

        let content_user_mention = CallNotifyEventContent::new(
            "abcdef".into(),
            ApplicationType::Call,
            NotifyType::Ring,
            Mentions::with_user_ids(vec![
                owned_user_id!("@user:example.com"),
                owned_user_id!("@user2:example.com"),
            ]),
        );

        let content_room_mention = CallNotifyEventContent::new(
            "abcdef".into(),
            ApplicationType::Call,
            NotifyType::Ring,
            Mentions::with_room_mention(),
        );

        assert_eq!(
            to_json_value(&content_user_mention).unwrap(),
            json!({
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": {
                    "user_ids": ["@user2:example.com","@user:example.com"],
                },
                "notify_type": "ring",
            })
        );
        assert_eq!(
            to_json_value(&content_room_mention).unwrap(),
            json!({
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": { "room": true },
                "notify_type": "ring",
            })
        );
    }

    #[test]
    fn notify_event_deserialization() {
        use std::collections::BTreeSet;

        use assert_matches2::assert_matches;
        use ruma_common::owned_user_id;

        use crate::{AnyMessageLikeEvent, MessageLikeEvent};

        let json_data = json!({
            "content": {
                "call_id": "abcdef",
                "application": "m.call",
                "m.mentions": {
                    "room": false,
                    "user_ids": ["@user:example.com", "@user2:example.com"],
                },
                "notify_type": "ring",
            },
            "event_id": "$event:notareal.hs",
            "origin_server_ts": 134_829_848,
            "room_id": "!roomid:notareal.hs",
            "sender": "@user:notareal.hs",
            "type": "m.call.notify",
        });

        let event = from_json_value::<AnyMessageLikeEvent>(json_data).unwrap();
        assert_matches!(
            event,
            AnyMessageLikeEvent::CallNotify(MessageLikeEvent::Original(message_event))
        );
        let content = message_event.content;
        assert_eq!(content.call_id, "abcdef");
        assert!(!content.mentions.room);
        assert_eq!(
            content.mentions.user_ids,
            BTreeSet::from([
                owned_user_id!("@user:example.com"),
                owned_user_id!("@user2:example.com")
            ])
        );
    }
}
