//! Types for the *m.presence* event.

use js_int::UInt;
use ruma_events_macros::ruma_event;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_event! {
    /// Informs the client of a user's presence state change.
    PresenceEvent {
        kind: Event,
        event_type: Presence,
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
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum PresenceState {
    /// Disconnected from the service.
    #[serde(rename = "offline")]
    Offline,

    /// Connected to the service.
    #[serde(rename = "online")]
    Online,

    /// Connected to the service but not available for chat.
    #[serde(rename = "unavailable")]
    Unavailable,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    PresenceState {
        Offline => "offline",
        Online => "online",
        Unavailable => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::UserId;
    use serde_json::to_string;

    use super::{PresenceEvent, PresenceEventContent, PresenceState};

    /// Test serialization and deserialization of example m.presence event from the spec
    /// https://github.com/turt2live/matrix-doc/blob/master/event-schemas/examples/m.presence
    #[test]
    fn test_example_event() {
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
        let serialized_event =
            r#"{"content":{"avatar_url":"mxc://localhost:wefuiwegh8742w","currently_active":false,"last_active_ago":2478593,"presence":"online","status_msg":"Making cupcakes"},"sender":"@example:localhost","type":"m.presence"}"#;

        assert_eq!(to_string(&event).unwrap(), serialized_event);
        let deserialized_event: PresenceEvent = serialized_event.parse().unwrap();
        assert_eq!(deserialized_event.content, event.content);
        assert_eq!(deserialized_event.sender, event.sender);
    }
}
