//! Types for the *m.room.server_acl* event.

use std::{convert::TryFrom, str::FromStr};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::{default_true, Event, EventType, InvalidEvent, RoomEvent, StateEvent};

/// An event to indicate which servers are permitted to participate in the room.
#[derive(Clone, Debug, PartialEq)]
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
    pub unsigned: Option<Value>,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
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

impl FromStr for ServerAclEvent {
    type Err = crate::InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn from_str(json: &str) -> Result<Self, Self::Err> {
        let raw = serde_json::from_str::<raw::ServerAclEvent>(json)?;

        Ok(Self {
            content: ServerAclEventContent {
                allow_ip_literals: raw.content.allow_ip_literals,
                allow: raw.content.allow,
                deny: raw.content.deny,
            },
            event_id: raw.event_id,
            origin_server_ts: raw.origin_server_ts,
            prev_content: raw.prev_content.map(|prev| ServerAclEventContent {
                allow_ip_literals: prev.allow_ip_literals,
                allow: prev.allow,
                deny: prev.deny,
            }),
            room_id: raw.room_id,
            unsigned: raw.unsigned,
            sender: raw.sender,
            state_key: raw.state_key,
        })
    }
}

impl<'a> TryFrom<&'a str> for ServerAclEvent {
    type Error = crate::InvalidEvent;

    /// Attempt to create `Self` from parsing a string of JSON data.
    fn try_from(json: &'a str) -> Result<Self, Self::Error> {
        FromStr::from_str(json)
    }
}

impl Serialize for ServerAclEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ServerAclEvent", 2)?;

        state.serialize_field("content", &self.content)?;
        state.serialize_field("type", &self.event_type())?;

        state.end()
    }
}

impl Event for ServerAclEvent {
    /// The type of this event's `content` field.
    type Content = ServerAclEventContent;

    /// The event's content.
    fn content(&self) -> &Self::Content {
        &self.content
    }

    /// The type of the event.
    fn event_type(&self) -> EventType {
        EventType::RoomServerAcl
    }
}

impl RoomEvent for ServerAclEvent {
    /// The unique identifier for the event.
    fn event_id(&self) -> &EventId {
        &self.event_id
    }

    /// Timestamp (milliseconds since the UNIX epoch) on originating homeserver when this event was
    /// sent.
    fn origin_server_ts(&self) -> UInt {
        self.origin_server_ts
    }

    /// The unique identifier for the room associated with this event.
    ///
    /// This can be `None` if the event came from a context where there is
    /// no ambiguity which room it belongs to, like a `/sync` response for example.
    fn room_id(&self) -> Option<&RoomId> {
        self.room_id.as_ref()
    }

    /// The unique identifier for the user who sent this event.
    fn sender(&self) -> &UserId {
        &self.sender
    }

    /// Additional key-value pairs not signed by the homeserver.
    fn unsigned(&self) -> Option<&Value> {
        self.unsigned.as_ref()
    }
}

impl StateEvent for ServerAclEvent {
    /// The previous content for this state key, if any.
    fn prev_content(&self) -> Option<&Self::Content> {
        self.prev_content.as_ref()
    }

    /// A key that determines which piece of room state the event represents.
    fn state_key(&self) -> &str {
        &self.state_key
    }
}

mod raw {
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
        pub unsigned: Option<Value>,

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
    use super::ServerAclEvent;

    #[test]
    fn default_values() {
        let server_acl_event: ServerAclEvent =
            r#"{"content":{},"event_id":"$h29iv0s8:example.com","origin_server_ts":1,"sender":"@carl:example.com","state_key":"","type":"m.room.server_acl"}"#
            .parse().unwrap();

        assert_eq!(server_acl_event.content.allow_ip_literals, true);
        assert!(server_acl_event.content.allow.is_empty());
        assert!(server_acl_event.content.deny.is_empty());
    }
}
