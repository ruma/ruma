//! A presence event is represented by a struct with a set content field.
//!
//! The only content valid for this event is `PresenceEventContent`.

use js_int::UInt;
use ruma_common::presence::PresenceState;
use ruma_events_macros::{Event, EventContent};
use ruma_identifiers::{MxcUri, UserId};
use serde::{Deserialize, Serialize};

/// Presence event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct PresenceEvent {
    /// Data specific to the event type.
    pub content: PresenceEventContent,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: UserId,
}

/// Informs the room of members presence.
///
/// This is the only type a `PresenceEvent` can contain as its `content` field.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.presence")]
pub struct PresenceEventContent {
    /// The current avatar URL for this user.
    ///
    /// If you activate the `compat` feature, this field being an empty string in JSON will give
    /// you `None` here.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
    )]
    pub avatar_url: Option<MxcUri>,

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
}

impl PresenceEventContent {
    /// Creates a new `PresenceEventContent` with the given state.
    pub fn new(presence: PresenceState) -> Self {
        Self {
            avatar_url: None,
            currently_active: None,
            displayname: None,
            last_active_ago: None,
            presence,
            status_msg: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use matches::assert_matches;
    use ruma_common::presence::PresenceState;
    use ruma_identifiers::{mxc_uri, user_id};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{PresenceEvent, PresenceEventContent};

    #[test]
    fn serialization() {
        let event = PresenceEvent {
            content: PresenceEventContent {
                avatar_url: Some(mxc_uri!("mxc://localhost/wefuiwegh8742w")),
                currently_active: Some(false),
                displayname: None,
                last_active_ago: Some(uint!(2_478_593)),
                presence: PresenceState::Online,
                status_msg: Some("Making cupcakes".into()),
            },
            sender: user_id!("@example:localhost"),
        };

        let json = json!({
            "content": {
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
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
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes"
            },
            "sender": "@example:localhost",
            "type": "m.presence"
        });

        assert_matches!(
            from_json_value::<PresenceEvent>(json).unwrap(),
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
            } if avatar_url.to_string() == "mxc://localhost/wefuiwegh8742w"
                && status_msg == "Making cupcakes"
                && sender == "@example:localhost"
                && last_active_ago == uint!(2_478_593)
        );

        #[cfg(feature = "compat")]
        assert_matches!(
            from_json_value::<PresenceEvent>(json!({
                "content": {
                    "avatar_url": "",
                    "currently_active": false,
                    "last_active_ago": 2_478_593,
                    "presence": "online",
                    "status_msg": "Making cupcakes"
                },
                "sender": "@example:localhost",
                "type": "m.presence"
            })).unwrap(),
            PresenceEvent {
                content: PresenceEventContent {
                    avatar_url: None,
                    currently_active: Some(false),
                    displayname: None,
                    last_active_ago: Some(last_active_ago),
                    presence: PresenceState::Online,
                    status_msg: Some(status_msg),
                },
                sender,
            } if status_msg == "Making cupcakes"
                && sender == "@example:localhost"
                && last_active_ago == uint!(2_478_593)
        );
    }
}
