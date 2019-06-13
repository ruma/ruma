//! Types for the *m.room.server_acl* event.

use serde::{Deserialize, Serialize};

use crate::default_true;

state_event! {
    /// An event to indicate which servers are permitted to participate in the room.
    pub struct ServerAclEvent(ServerAclEventContent) {}
}

/// The payload of an *m.room.server_acl* event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::ServerAclEventContent;

    #[test]
    fn default_values() {
        let content: ServerAclEventContent = serde_json::from_str("{}").unwrap();

        assert_eq!(content.allow_ip_literals, true);
        assert!(content.allow.is_empty());
        assert!(content.deny.is_empty());
    }
}
