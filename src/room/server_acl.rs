//! Types for the *m.room.server_acl* event.

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{
    default_true, Event, EventResult, EventType, InnerInvalidEvent, InvalidEvent, RoomEvent,
    StateEvent,
};

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

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: Option<Value>,
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

impl<'de> Deserialize<'de> for EventResult<ServerAclEvent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::ServerAclEvent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(ServerAclEvent {
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
        }))
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

impl_state_event!(
    ServerAclEvent,
    ServerAclEventContent,
    EventType::RoomServerAcl
);

impl<'de> Deserialize<'de> for EventResult<ServerAclEventContent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = serde_json::Value::deserialize(deserializer)?;

        let raw: raw::ServerAclEventContent = match serde_json::from_value(json.clone()) {
            Ok(raw) => raw,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(
                    InnerInvalidEvent::Validation {
                        json,
                        message: error.to_string(),
                    },
                )));
            }
        };

        Ok(EventResult::Ok(ServerAclEventContent {
            allow_ip_literals: raw.allow_ip_literals,
            allow: raw.allow,
            deny: raw.deny,
        }))
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
