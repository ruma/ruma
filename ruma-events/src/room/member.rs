//! Types for the *m.room.member* event.

use std::collections::BTreeMap;

use ruma_events_macros::StateEventContent;
use ruma_identifiers::UserId;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::StateEvent;

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
pub type MemberEvent = StateEvent<MemberEventContent>;

/// The payload for `MemberEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.member")]
pub struct MemberEventContent {
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
}

/// The membership state of a user.
#[derive(Clone, Copy, Debug, PartialEq, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MembershipState {
    /// The user is banned.
    Ban,

    /// The user has been invited.
    Invite,

    /// The user has joined.
    Join,

    /// The user has requested to join.
    Knock,

    /// The user has left.
    Leave,
}

/// Information about a third party invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedContent {
    /// The invited Matrix user ID.
    ///
    /// Must be equal to the user_id property of the event.
    pub mxid: UserId,

    /// A single signature from the verifying server, in the format specified by the Signing Events
    /// section of the server-server API.
    pub signatures: BTreeMap<String, BTreeMap<String, String>>,

    /// The token property of the containing third_party_invite object.
    pub token: String,
}

/// Translation of the membership change in `m.room.member` event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
    ProfileChanged {
        /// Whether the `displayname` changed.
        displayname_changed: bool,
        /// Whether the `avatar_url` changed.
        avatar_url_changed: bool,
    },

    /// Not implemented.
    NotImplemented,
}

impl MemberEvent {
    /// Helper function for membership change. Check [the specification][spec] for details.
    ///
    /// [spec]: https://matrix.org/docs/spec/client_server/latest#m-room-member
    pub fn membership_change(&self) -> MembershipChange {
        use MembershipState::*;
        let prev_content = if let Some(prev_content) = &self.prev_content {
            prev_content
        } else {
            &MemberEventContent {
                avatar_url: None,
                displayname: None,
                is_direct: None,
                membership: Leave,
                third_party_invite: None,
            }
        };

        match (prev_content.membership, &self.content.membership) {
            (Invite, Invite) | (Leave, Leave) | (Ban, Ban) => MembershipChange::None,
            (Invite, Join) | (Leave, Join) => MembershipChange::Joined,
            (Invite, Leave) => {
                if self.sender == self.state_key {
                    MembershipChange::InvitationRevoked
                } else {
                    MembershipChange::InvitationRejected
                }
            }
            (Invite, Ban) | (Leave, Ban) => MembershipChange::Banned,
            (Join, Invite) | (Ban, Invite) | (Ban, Join) => MembershipChange::Error,
            (Join, Join) => MembershipChange::ProfileChanged {
                displayname_changed: prev_content.displayname != self.content.displayname,
                avatar_url_changed: prev_content.avatar_url != self.content.avatar_url,
            },
            (Join, Leave) => {
                if self.sender == self.state_key {
                    MembershipChange::Left
                } else {
                    MembershipChange::Kicked
                }
            }
            (Join, Ban) => MembershipChange::KickedAndBanned,
            (Leave, Invite) => MembershipChange::Invited,
            (Ban, Leave) => MembershipChange::Unbanned,
            (Knock, _) | (_, Knock) => MembershipChange::NotImplemented,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use maplit::btreemap;
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json};

    use super::{MemberEventContent, MembershipState, SignedContent, ThirdPartyInvite};
    use crate::{EventJson, StateEvent};

    #[test]
    fn serde_with_no_prev_content() {
        let json = json!({
            "type": "m.room.member",
            "content": {
                "membership": "join"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "example.com"
        });

        assert_matches!(
            from_json_value::<EventJson<StateEvent<MemberEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent::<MemberEventContent> {
                content: MemberEventContent {
                    avatar_url: None,
                    displayname: None,
                    is_direct: None,
                    membership: MembershipState::Join,
                    third_party_invite: None,
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: None,
            } if event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == "!n8f893n9:example.com"
                && sender == "@carl:example.com"
                && state_key == "example.com"
                && unsigned.is_empty()
        );
    }

    #[test]
    fn serde_with_prev_content() {
        let json = json!({
            "type": "m.room.member",
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
            "state_key": "example.com"
        });

        assert_matches!(
            from_json_value::<EventJson<StateEvent<MemberEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent::<MemberEventContent> {
                content: MemberEventContent {
                    avatar_url: None,
                    displayname: None,
                    is_direct: None,
                    membership: MembershipState::Join,
                    third_party_invite: None,
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: Some(MemberEventContent {
                    avatar_url: None,
                    displayname: None,
                    is_direct: None,
                    membership: MembershipState::Join,
                    third_party_invite: None,
                }),
            } if event_id == "$h29iv0s8:example.com"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == "!n8f893n9:example.com"
                && sender == "@carl:example.com"
                && state_key == "example.com"
                && unsigned.is_empty()
        );
    }

    #[test]
    fn serde_with_content_full() {
        let json = json!({
            "type": "m.room.member",
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
            "origin_server_ts": 233,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org"
        });

        assert_matches!(
            from_json_value::<EventJson<StateEvent<MemberEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent::<MemberEventContent> {
                content: MemberEventContent {
                    avatar_url: Some(avatar_url),
                    displayname: Some(displayname),
                    is_direct: Some(true),
                    membership: MembershipState::Invite,
                    third_party_invite: Some(ThirdPartyInvite {
                        display_name: third_party_displayname,
                        signed: SignedContent { mxid, signatures, token },
                    }),
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: None,
            } if avatar_url == "mxc://example.org/SEsfnsuifSDFSSEF"
                && displayname == "Alice Margatroid"
                && third_party_displayname == "alice"
                && mxid == "@alice:example.org"
                && signatures == btreemap! {
                    "magic.forest".to_owned() => btreemap! {
                        "ed25519:3".to_owned() => "foobar".to_owned()
                    }
                }
                && token == "abc123"
                && event_id == "$143273582443PhrSn:example.org"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(233)
                && room_id == "!jEsUZKDJdhlrceRyVU:example.org"
                && sender == "@alice:example.org"
                && state_key == "@alice:example.org"
                && unsigned.is_empty()
        )
    }

    #[test]
    fn serde_with_prev_content_full() {
        let json = json!({
            "type": "m.room.member",
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
            "state_key": "@alice:example.org"
        });

        assert_matches!(
            from_json_value::<EventJson<StateEvent<MemberEventContent>>>(json)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent::<MemberEventContent> {
                content: MemberEventContent {
                    avatar_url: None,
                    displayname: None,
                    is_direct: None,
                    membership: MembershipState::Join,
                    third_party_invite: None,
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: Some(MemberEventContent {
                    avatar_url: Some(avatar_url),
                    displayname: Some(displayname),
                    is_direct: Some(true),
                    membership: MembershipState::Invite,
                    third_party_invite: Some(ThirdPartyInvite {
                        display_name: third_party_displayname,
                        signed: SignedContent { mxid, signatures, token },
                    }),
                }),
            } if event_id == "$143273582443PhrSn:example.org"
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(233)
                && room_id == "!jEsUZKDJdhlrceRyVU:example.org"
                && sender == "@alice:example.org"
                && state_key == "@alice:example.org"
                && unsigned.is_empty()
                && avatar_url == "mxc://example.org/SEsfnsuifSDFSSEF"
                && displayname == "Alice Margatroid"
                && third_party_displayname == "alice"
                && mxid == "@alice:example.org"
                && signatures == btreemap! {
                    "magic.forest".to_owned() => btreemap! {
                        "ed25519:3".to_owned() => "foobar".to_owned()
                    }
                }
                && token == "abc123"
        );
    }
}
