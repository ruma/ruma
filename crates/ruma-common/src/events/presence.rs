//! A presence event is represented by a struct with a set content field.
//!
//! The only content valid for this event is `PresenceEventContent`.

use js_int::UInt;
use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};

use super::{EventKind, StaticEventContent};
use crate::{presence::PresenceState, OwnedMxcUri, OwnedUserId};

/// Presence event.
#[derive(Clone, Debug, Event)]
#[allow(clippy::exhaustive_structs)]
pub struct PresenceEvent {
    /// Data specific to the event type.
    pub content: PresenceEventContent,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,
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
    /// If you activate the `compat` feature, this field being an empty string in JSON will result
    /// in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "crate::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

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

impl StaticEventContent for PresenceEventContent {
    const KIND: EventKind = EventKind::Presence;
    const TYPE: &'static str = "m.presence";
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{PresenceEvent, PresenceEventContent};
    use crate::{mxc_uri, presence::PresenceState, user_id};

    #[test]
    fn serialization() {
        let event = PresenceEvent {
            content: PresenceEventContent {
                avatar_url: Some(mxc_uri!("mxc://localhost/wefuiwegh8742w").to_owned()),
                currently_active: Some(false),
                displayname: None,
                last_active_ago: Some(uint!(2_478_593)),
                presence: PresenceState::Online,
                status_msg: Some("Making cupcakes".into()),
            },
            sender: user_id!("@example:localhost").to_owned(),
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

        let ev = from_json_value::<PresenceEvent>(json).unwrap();
        assert_eq!(
            ev.content.avatar_url.as_deref(),
            Some(mxc_uri!("mxc://localhost/wefuiwegh8742w"))
        );
        assert_eq!(ev.content.currently_active, Some(false));
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
        assert_eq!(ev.content.presence, PresenceState::Online);
        assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
        assert_eq!(ev.sender, "@example:localhost");

        #[cfg(feature = "compat")]
        {
            let json = json!({
                "content": {
                    "avatar_url": "",
                    "currently_active": false,
                    "last_active_ago": 2_478_593,
                    "presence": "online",
                    "status_msg": "Making cupcakes"
                },
                "sender": "@example:localhost",
                "type": "m.presence"
            });

            let ev = from_json_value::<PresenceEvent>(json).unwrap();
            assert_eq!(ev.content.avatar_url, None);
            assert_eq!(ev.content.currently_active, Some(false));
            assert_eq!(ev.content.displayname, None);
            assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
            assert_eq!(ev.content.presence, PresenceState::Online);
            assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
            assert_eq!(ev.sender, "@example:localhost");
        }
    }
}
