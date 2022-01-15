//! Types for the [`m.room.server_acl`] event.
//!
//! [`m.room.server_acl`]: https://spec.matrix.org/v1.1/client-server-api/#mroomserver_acl

use ruma_events_macros::EventContent;
use ruma_identifiers::ServerName;
use serde::{Deserialize, Serialize};

/// The content of an `m.room.server_acl` event.
///
/// An event to indicate which servers are permitted to participate in the room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.server_acl", kind = State)]
pub struct RoomServerAclEventContent {
    /// Whether to allow server names that are IP address literals.
    ///
    /// This is strongly recommended to be set to false as servers running with IP literal names
    /// are strongly discouraged in order to require legitimate homeservers to be backed by a
    /// valid registered domain name.
    #[serde(default = "ruma_serde::default_true", skip_serializing_if = "ruma_serde::is_true")]
    pub allow_ip_literals: bool,

    /// The server names to allow in the room, excluding any port information.
    ///
    /// Wildcards may be used to cover a wider range of hosts, where `*` matches zero or more
    /// characters and `?` matches exactly one character.
    ///
    /// **Defaults to an empty list when not provided, effectively disallowing every server.**
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,

    /// The server names to disallow in the room, excluding any port information.
    ///
    /// Wildcards may be used to cover a wider range of hosts, where * matches zero or more
    /// characters and `?` matches exactly one character.
    ///
    /// Defaults to an empty list when not provided.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

impl RoomServerAclEventContent {
    /// Creates a new `RoomServerAclEventContent` with the given IP literal allowance flag, allowed
    /// and denied servers.
    pub fn new(allow_ip_literals: bool, allow: Vec<String>, deny: Vec<String>) -> Self {
        Self { allow_ip_literals, allow, deny }
    }

    /// Returns true if and only if the server is allowed by the ACL rules.
    pub fn is_allowed(&self, server_name: &ServerName) -> bool {
        if !self.allow_ip_literals && server_name.is_ip_literal() {
            return false;
        }

        for deny in &self.deny {
            if let Ok(d) = glob::Pattern::new(deny) {
                if d.matches(server_name.as_str()) {
                    return false;
                }
            }
        }

        for allow in &self.allow {
            if let Ok(a) = glob::Pattern::new(allow) {
                if a.matches(server_name.as_str()) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json};

    use super::*;
    use crate::StateEvent;
    use ruma_identifiers::server_name;

    #[test]
    fn default_values() {
        let json_data = json!({
            "content": {},
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.server_acl"
        });

        let server_acl_event: StateEvent<RoomServerAclEventContent> =
            from_json_value(json_data).unwrap();

        assert!(server_acl_event.content.allow_ip_literals);
        assert!(server_acl_event.content.allow.is_empty());
        assert!(server_acl_event.content.deny.is_empty());
    }

    #[test]
    fn acl_allow_ip_literal() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: true,
            allow: vec!["*".to_owned()],
            deny: Vec::new(),
        };
        assert!(acl_event.is_allowed(server_name!("1.1.1.1")));
    }

    #[test]
    fn acl_deny_ip_literal() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("1.1.1.1")));
    }

    #[test]
    fn acl_deny() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*".to_owned()],
            deny: vec!["matrix.org".to_owned()],
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("conduit.rs")));
    }

    #[test]
    fn acl_explicit_allow() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["conduit.rs".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("conduit.rs")));
    }

    #[test]
    fn acl_explicit_glob_1() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["*.matrix.org".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix.org")));
        assert!(acl_event.is_allowed(server_name!("server.matrix.org")));
    }

    #[test]
    fn acl_explicit_glob_2() {
        let acl_event = RoomServerAclEventContent {
            allow_ip_literals: false,
            allow: vec!["matrix??.org".to_owned()],
            deny: Vec::new(),
        };
        assert!(!acl_event.is_allowed(server_name!("matrix1.org")));
        assert!(acl_event.is_allowed(server_name!("matrix02.org")));
    }
}
