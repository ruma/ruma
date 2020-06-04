//! Types for the *m.presence* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

ruma_event! {
    /// Informs the client of a user's presence state change.
    PresenceEvent {
        kind: Event,
        event_type: "m.presence",
        fields: {
            /// The unique identifier for the user associated with this event.
            pub sender: UserId,
        },
        content: {
            /// The current avatar URL for this user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub avatar_url: Option<String>,

            /// Whether or not the user is currently active.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub currently_active: Option<bool>,

            /// The current display name for this user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>,

            /// The last time since this user performed some action, in milliseconds.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub last_active_ago: Option<UInt>,

            /// The presence state for this user.
            pub presence: PresenceState,

            /// An optional description to accompany the presence.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub status_msg: Option<String>,
        },
    }
}

/// A description of a user's connectivity and availability for chat.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PresenceState {
    /// Disconnected from the service.
    Offline,

    /// Connected to the service.
    Online,

    /// Connected to the service but not available for chat.
    Unavailable,
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use matches::assert_matches;
    use ruma_identifiers::UserId;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{PresenceEvent, PresenceEventContent, PresenceState};
    use crate::EventJson;

    #[test]
    fn serialization() {
        let event = PresenceEvent {
            content: PresenceEventContent {
                avatar_url: Some("mxc://localhost:wefuiwegh8742w".to_string()),
                currently_active: Some(false),
                displayname: None,
                last_active_ago: Some(UInt::try_from(2_478_593).unwrap()),
                presence: PresenceState::Online,
                status_msg: Some("Making cupcakes".to_string()),
            },
            sender: UserId::try_from("@example:localhost").unwrap(),
        };

        let json = json!({
            "content": {
                "avatar_url": "mxc://localhost:wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes"
            },
            "sender": "@example:localhost",
            "type": "m.presence"
        });

        assert_eq!(to_json_value(&event).unwrap(), json);
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "avatar_url": "mxc://localhost:wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes"
            },
            "sender": "@example:localhost",
            "type": "m.presence"
        });

        assert_matches!(
            from_json_value::<EventJson<PresenceEvent>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            PresenceEvent {
                content: PresenceEventContent {
                    avatar_url: Some(avatar_url),
                    currently_active: Some(false),
                    displayname: None,
                    last_active_ago: Some(last_active_ago),
                    presence: PresenceState::Online,
                    status_msg: Some(status_msg),
                },
                sender,
            } if avatar_url == "mxc://localhost:wefuiwegh8742w"
                && status_msg == "Making cupcakes"
                && sender == "@example:localhost"
                && last_active_ago == UInt::from(2_478_593u32)
        );
    }
}
