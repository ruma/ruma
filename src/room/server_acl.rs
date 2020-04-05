//! Types for the *m.room.server_acl* event.

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{util::default_true, EventType, FromRaw};

/// An event to indicate which servers are permitted to participate in the room.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename = "m.room.server_acl", tag = "type")]
pub struct ServerAclEvent {
    /// The event's content.
    pub content: ServerAclEventContent,

    /// The unique identifier for the event.
    pub event_id: EventId,

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
    /// event was sent.
    pub origin_server_ts: UInt,

    /// The previous content for this state key, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_content: Option<ServerAclEventContent>,

    /// The unique identifier for the room associated with this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<RoomId>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub unsigned: Map<String, Value>,
}

/// The payload for `ServerAclEvent`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ServerAclEventContent {
    /// True to allow server names that are IP address literals. False to deny. Defaults to true if
    /// missing or otherwise not a boolean.
    ///
    /// This is strongly recommended to be set to false as servers running with IP literal names are
    /// strongly discouraged in order to require legitimate homeservers to be backed by a valid
    /// registered domain name.
    #[serde(default = "default_true")]
    pub allow_ip_literals: bool,

    /// The server names to allow in the room, excluding any port information. Wildcards may be used
    /// to cover a wider range of hosts, where * matches zero or more characters and ? matches
    /// exactly one character.
    ///
    /// **This defaults to an empty list when not provided, effectively disallowing every server.**
    #[serde(default)]
    pub allow: Vec<String>,

    /// The server names to disallow in the room, excluding any port information. Wildcards may be
    /// used to cover a wider range of hosts, where * matches zero or more characters and ? matches
    /// exactly one character.
    ///
    /// This defaults to an empty list when not provided.
    #[serde(default)]
    pub deny: Vec<String>,
}

impl FromRaw for ServerAclEvent {
    type Raw = raw::ServerAclEvent;

    fn from_raw(raw: raw::ServerAclEvent) -> Self {
        Self {
            content: FromRaw::from_raw(raw.content),
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw.prev_content.map(FromRaw::from_raw),
            room_id: raw.room_id,
            sender: raw.sender,
            state_key: raw.state_key,
            unsigned: raw.unsigned,
        }
    }
}

impl FromRaw for ServerAclEventContent {
    type Raw = raw::ServerAclEventContent;

    fn from_raw(raw: raw::ServerAclEventContent) -> Self {
        Self {
            allow_ip_literals: raw.allow_ip_literals,
            allow: raw.allow,
            deny: raw.deny,
        }
    }
}

impl_state_event!(
    ServerAclEvent,
    ServerAclEventContent,
    EventType::RoomServerAcl
);

pub(crate) mod raw {
    use super::*;

    /// An event to indicate which servers are permitted to participate in the room.
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    pub struct ServerAclEvent {
        /// The event's content.
        pub content: ServerAclEventContent,

        /// The unique identifier for the event.
        pub event_id: EventId,

        /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this
        /// event was sent.
        pub origin_server_ts: UInt,

        /// The previous content for this state key, if any.
        pub prev_content: Option<ServerAclEventContent>,

        /// The unique identifier for the room associated with this event.
        pub room_id: Option<RoomId>,

        /// Additional key-value pairs not signed by the homeserver.
        #[serde(default)]
        pub unsigned: Map<String, Value>,

        /// The unique identifier for the user who sent this event.
        pub sender: UserId,

        /// A key that determines which piece of room state the event represents.
        pub state_key: String,
    }

    /// The payload for `ServerAclEvent`.
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    pub struct ServerAclEventContent {
        /// True to allow server names that are IP address literals. False to deny. Defaults to true
        /// if missing or otherwise not a boolean.
        ///
        /// This is strongly recommended to be set to false as servers running with IP literal names
        /// are strongly discouraged in order to require legitimate homeservers to be backed by a
        /// valid registered domain name.
        #[serde(default = "default_true")]
        pub allow_ip_literals: bool,

        /// The server names to allow in the room, excluding any port information. Wildcards may be
        /// used to cover a wider range of hosts, where * matches zero or more characters and ?
        /// matches exactly one character.
        ///
        /// **This defaults to an empty list when not provided, effectively disallowing every
        /// server.**
        #[serde(default)]
        pub allow: Vec<String>,

        /// The server names to disallow in the room, excluding any port information. Wildcards may
        /// be used to cover a wider range of hosts, where * matches zero or more characters and ?
        /// matches exactly one character.
        ///
        /// This defaults to an empty list when not provided.
        #[serde(default)]
        pub deny: Vec<String>,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::ServerAclEvent;
    use crate::EventResult;

    #[test]
    fn default_values() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com","origin_server_ts":1,
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.server_acl"
        });
        let server_acl_event: ServerAclEvent = from_json_value::<EventResult<_>>(json_data)
            .unwrap()
            .into_result()
            .unwrap();

        assert_eq!(server_acl_event.content.allow_ip_literals, true);
        assert!(server_acl_event.content.allow.is_empty());
        assert!(server_acl_event.content.deny.is_empty());
    }
}
