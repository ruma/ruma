//! Types for the *m.room.member* event.

use std::collections::HashMap;

use ruma_events_macros::ruma_event;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};

ruma_event! {
    /// The current membership state of a user in the room.
    ///
    /// Adjusts the membership state for a user in a room. It is preferable to use the membership
    /// APIs (`/rooms/<room id>/invite` etc) when performing membership actions rather than
    /// adjusting the state directly as there are a restricted set of valid transformations. For
    /// example, user A cannot force user B to join a room, and trying to force this state change
    /// directly will fail.
    ///
    /// The `third_party_invite` property will be set if this invite is an *invite* event and is the
    /// successor of an *m.room.third_party_invite* event, and absent otherwise.
    ///
    /// This event may also include an `invite_room_state` key inside the event's unsigned data. If
    /// present, this contains an array of `StrippedState` events. These events provide information
    /// on a subset of state events such as the room name. Note that ruma-events treats unsigned
    /// data on events as arbitrary JSON values, and the ruma-events types for this event don't
    /// provide direct access to `invite_room_state`. If you need this data, you must extract and
    /// convert it from a `serde_json::Value` yourself.
    ///
    /// The user for which a membership applies is represented by the `state_key`. Under some
    /// conditions, the `sender` and `state_key` may not match - this may be interpreted as the
    /// `sender` affecting the membership state of the `state_key` user.
    ///
    /// The membership for a given user can change over time. Previous membership can be retrieved
    /// from the `prev_content` object on an event. If not present, the user's previous membership
    /// must be assumed as leave.
    MemberEvent {
        kind: StateEvent,
        event_type: RoomMember,
        content: {
            /// The avatar URL for this user, if any. This is added by the homeserver.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub avatar_url: Option<String>,

            /// The display name for this user, if any. This is added by the homeserver.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>,

            /// Flag indicating if the room containing this event was created
            /// with the intention of being a direct chat.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub is_direct: Option<bool>,

            /// The membership state of this user.
            pub membership: MembershipState,

            /// If this member event is the successor to a third party invitation, this field will
            /// contain information about that invitation.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub third_party_invite: Option<ThirdPartyInvite>,
        },
    }
}

/// The membership state of a user.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum MembershipState {
    /// The user is banned.
    #[serde(rename = "ban")]
    Ban,

    /// The user has been invited.
    #[serde(rename = "invite")]
    Invite,

    /// The user has joined.
    #[serde(rename = "join")]
    Join,

    /// The user has requested to join.
    #[serde(rename = "knock")]
    Knock,

    /// The user has left.
    #[serde(rename = "leave")]
    Leave,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    #[serde(skip)]
    __Nonexhaustive,
}

impl_enum! {
    MembershipState {
        Ban => "ban",
        Invite => "invite",
        Join => "join",
        Knock => "knock",
        Leave => "leave",
    }
}

/// Information about a third party invitation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ThirdPartyInvite {
    /// A name which can be displayed to represent the user instead of their third party
    /// identifier.
    pub display_name: String,

    /// A block of content which has been signed, which servers can use to verify the event.
    /// Clients should ignore this.
    pub signed: SignedContent,
}

/// A block of content which has been signed, which servers can use to verify a third party
/// invitation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SignedContent {
    /// The invited Matrix user ID.
    ///
    /// Must be equal to the user_id property of the event.
    pub mxid: UserId,

    /// A single signature from the verifying server, in the format specified by the Signing Events
    /// section of the server-server API.
    pub signatures: HashMap<String, HashMap<String, String>>,

    /// The token property of the containing third_party_invite object.
    pub token: String,
}

/// Translation of the membership change in `m.room.member` event.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum MembershipChange {
    /// No change.
    None,

    /// Must never happen.
    Error,

    /// User joined the room.
    Joined,

    /// User left the room.
    Left,

    /// User was banned.
    Banned,

    /// User was unbanned.
    Unbanned,

    /// User was kicked.
    Kicked,

    /// User was invited.
    Invited,

    /// User was kicked and banned.
    KickedAndBanned,

    /// User rejected the invite.
    InvitationRejected,

    /// User had their invite revoked.
    InvitationRevoked,

    /// `displayname` or `avatar_url` changed.
    ProfileChanged,

    /// Not implemented.
    NotImplemented,

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl MemberEvent {
    /// Helper function for memebership change. Check [the specification][spec] for details.
    ///
    /// [spec]: https://matrix.org/docs/spec/client_server/latest#m-room-member
    pub fn membership_change(&self) -> MembershipChange {
        use MembershipState::*;
        let prev_membership = if let Some(prev_content) = &self.prev_content {
            prev_content.membership
        } else {
            Leave
        };
        match (prev_membership, &self.content.membership) {
            (Invite, Invite) | (Leave, Leave) | (Ban, Ban) => MembershipChange::None,
            (Invite, Join) | (Leave, Join) => MembershipChange::Joined,
            (Invite, Leave) => {
                if self.sender.to_string() == self.state_key {
                    MembershipChange::InvitationRevoked
                } else {
                    MembershipChange::InvitationRejected
                }
            }
            (Invite, Ban) | (Leave, Ban) => MembershipChange::Banned,
            (Join, Invite) | (Ban, Invite) | (Ban, Join) => MembershipChange::Error,
            (Join, Join) => MembershipChange::ProfileChanged,
            (Join, Leave) => {
                if self.sender.to_string() == self.state_key {
                    MembershipChange::Left
                } else {
                    MembershipChange::Kicked
                }
            }
            (Join, Ban) => MembershipChange::KickedAndBanned,
            (Leave, Invite) => MembershipChange::Invited,
            (Ban, Leave) => MembershipChange::Unbanned,
            (Knock, _) | (_, Knock) => MembershipChange::NotImplemented,
            (__Nonexhaustive, _) | (_, __Nonexhaustive) => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::{EventId, RoomId, UserId};
    use serde_json::json;

    use super::*;
    use crate::util::serde_json_eq_try_from_raw;

    #[test]
    fn serde_with_no_prev_content() {
        let event = MemberEvent {
            content: MemberEventContent {
                avatar_url: None,
                displayname: None,
                is_direct: None,
                membership: MembershipState::Join,
                third_party_invite: None,
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::new(1).unwrap(),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
            prev_content: None,
        };
        let json = json!({
            "content": {
                "membership": "join"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.member"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn serde_with_prev_content() {
        let event = MemberEvent {
            content: MemberEventContent {
                avatar_url: None,
                displayname: None,
                is_direct: None,
                membership: MembershipState::Join,
                third_party_invite: None,
            },
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UInt::new(1).unwrap(),
            room_id: Some(RoomId::try_from("!n8f893n9:example.com").unwrap()),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "example.com".to_string(),
            unsigned: None,
            prev_content: Some(MemberEventContent {
                avatar_url: None,
                displayname: None,
                is_direct: None,
                membership: MembershipState::Join,
                third_party_invite: None,
            }),
        };
        let json = json!({
            "content": {
                "membership": "join"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "membership": "join"
            },
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com",
            "type": "m.room.member"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn serde_with_content_full() {
        let signatures = vec![(
            "magic.forest".to_owned(),
            vec![("ed25519:3".to_owned(), "foobar".to_owned())]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();
        let event = MemberEvent {
            content: MemberEventContent {
                avatar_url: Some("mxc://example.org/SEsfnsuifSDFSSEF".to_owned()),
                displayname: Some("Alice Margatroid".to_owned()),
                is_direct: Some(true),
                membership: MembershipState::Invite,
                third_party_invite: Some(ThirdPartyInvite {
                    display_name: "alice".to_owned(),
                    signed: SignedContent {
                        mxid: UserId::try_from("@alice:example.org").unwrap(),
                        signatures,
                        token: "abc123".to_owned(),
                    },
                }),
            },
            event_id: EventId::try_from("$143273582443PhrSn:example.org").unwrap(),
            origin_server_ts: UInt::new(233).unwrap(),
            room_id: Some(RoomId::try_from("!jEsUZKDJdhlrceRyVU:example.org").unwrap()),
            sender: UserId::try_from("@alice:example.org").unwrap(),
            state_key: "@alice:example.org".to_string(),
            unsigned: None,
            prev_content: None,
        };
        let json = json!({
            "content": {
                "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
                "displayname": "Alice Margatroid",
                "is_direct": true,
                "membership": "invite",
                "third_party_invite": {
                    "display_name": "alice",
                    "signed": {
                        "mxid": "@alice:example.org",
                        "signatures": {
                            "magic.forest": {
                                "ed25519:3": "foobar"
                            }
                        },
                        "token": "abc123"
                    }
                }
            },
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts":233,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org",
            "type": "m.room.member"
        });
        serde_json_eq_try_from_raw(event, json);
    }

    #[test]
    fn serde_with_prev_content_full() {
        let signatures = vec![(
            "magic.forest".to_owned(),
            vec![("ed25519:3".to_owned(), "foobar".to_owned())]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();
        let event = MemberEvent {
            content: MemberEventContent {
                avatar_url: None,
                displayname: None,
                is_direct: None,
                membership: MembershipState::Join,
                third_party_invite: None,
            },
            event_id: EventId::try_from("$143273582443PhrSn:example.org").unwrap(),
            origin_server_ts: UInt::new(233).unwrap(),
            room_id: Some(RoomId::try_from("!jEsUZKDJdhlrceRyVU:example.org").unwrap()),
            sender: UserId::try_from("@alice:example.org").unwrap(),
            state_key: "@alice:example.org".to_string(),
            unsigned: None,
            prev_content: Some(MemberEventContent {
                avatar_url: Some("mxc://example.org/SEsfnsuifSDFSSEF".to_owned()),
                displayname: Some("Alice Margatroid".to_owned()),
                is_direct: Some(true),
                membership: MembershipState::Invite,
                third_party_invite: Some(ThirdPartyInvite {
                    display_name: "alice".to_owned(),
                    signed: SignedContent {
                        mxid: UserId::try_from("@alice:example.org").unwrap(),
                        signatures,
                        token: "abc123".to_owned(),
                    },
                }),
            }),
        };
        let json = json!({
            "content": {
                "membership": "join"
            },
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 233,
            "prev_content": {
                "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
                "displayname": "Alice Margatroid",
                "is_direct": true,
                "membership": "invite",
                "third_party_invite": {
                    "display_name": "alice",
                    "signed": {
                        "mxid": "@alice:example.org",
                        "signatures": {
                            "magic.forest": {
                                "ed25519:3": "foobar"
                            }
                        },
                        "token": "abc123"
                    }
                }
            },
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org",
            "type": "m.room.member"
        });
        serde_json_eq_try_from_raw(event, json);
    }
}
