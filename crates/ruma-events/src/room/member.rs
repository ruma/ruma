//! Types for the *m.room.member* event.

use std::collections::BTreeMap;

use ruma_events_macros::EventContent;
use ruma_identifiers::{MxcUri, ServerNameBox, ServerSigningKeyId, UserId};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::{StateEvent, StrippedStateEvent, SyncStateEvent};

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
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.room.member", kind = State)]
pub struct MemberEventContent {
    /// The avatar URL for this user, if any. This is added by the homeserver.
    ///
    /// If you activate the `compat` feature, this field being an empty string in JSON will give
    /// you `None` here.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat",
        serde(default, deserialize_with = "ruma_serde::empty_string_as_none")
    )]
    pub avatar_url: Option<MxcUri>,

    /// The display name for this user, if any. This is added by the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayname: Option<String>,

    /// Flag indicating whether the room containing this event was created with the intention of
    /// being a direct chat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_direct: Option<bool>,

    /// The membership state of this user.
    #[ruma_event(skip_redaction)]
    pub membership: MembershipState,

    /// If this member event is the successor to a third party invitation, this field will
    /// contain information about that invitation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_invite: Option<ThirdPartyInvite>,

    /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-doc/pull/2448).
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    #[serde(rename = "xyz.amorgan.blurhash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,

    /// The reason for leaving a room.
    #[cfg(feature = "unstable-pre-spec")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl MemberEventContent {
    /// Creates a new `MemberEventContent` with the given membership state.
    pub fn new(membership: MembershipState) -> Self {
        Self {
            membership,
            avatar_url: None,
            displayname: None,
            is_direct: None,
            third_party_invite: None,
            #[cfg(feature = "unstable-pre-spec")]
            blurhash: None,
            #[cfg(feature = "unstable-pre-spec")]
            reason: None,
        }
    }
}

/// The membership state of a user.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
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

    #[doc(hidden)]
    _Custom(String),
}

/// Information about a third party invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct ThirdPartyInvite {
    /// A name which can be displayed to represent the user instead of their third party
    /// identifier.
    pub display_name: String,

    /// A block of content which has been signed, which servers can use to verify the event.
    ///
    /// Clients should ignore this.
    pub signed: SignedContent,
}

impl ThirdPartyInvite {
    /// Creates a new `ThirdPartyInvite` with the given display name and signed content.
    pub fn new(display_name: String, signed: SignedContent) -> Self {
        Self { display_name, signed }
    }
}

/// A block of content which has been signed, which servers can use to verify a third party
/// invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SignedContent {
    /// The invited Matrix user ID.
    ///
    /// Must be equal to the user_id property of the event.
    pub mxid: UserId,

    /// A single signature from the verifying server, in the format specified by the Signing Events
    /// section of the server-server API.
    pub signatures: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, String>>,

    /// The token property of the containing `third_party_invite` object.
    pub token: String,
}

impl SignedContent {
    /// Creates a new `SignedContent` with the given mxid, signature and token.
    pub fn new(
        mxid: UserId,
        signatures: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, String>>,
        token: String,
    ) -> Self {
        Self { mxid, signatures, token }
    }
}

/// Translation of the membership change in `m.room.member` event.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

/// Internal function so all `MemberEventContent` state event kinds can share the same
/// implementation.
fn membership_change(
    content: &MemberEventContent,
    prev_content: Option<&MemberEventContent>,
    sender: &UserId,
    state_key: &str,
) -> MembershipChange {
    use MembershipChange as Ch;
    use MembershipState as St;

    let prev_content = if let Some(prev_content) = &prev_content {
        prev_content
    } else {
        &MemberEventContent {
            avatar_url: None,
            displayname: None,
            is_direct: None,
            membership: St::Leave,
            third_party_invite: None,
            #[cfg(feature = "unstable-pre-spec")]
            blurhash: None,
            #[cfg(feature = "unstable-pre-spec")]
            reason: None,
        }
    };

    match (&prev_content.membership, &content.membership) {
        (St::Invite, St::Invite) | (St::Leave, St::Leave) | (St::Ban, St::Ban) => Ch::None,
        (St::Invite, St::Join) | (St::Leave, St::Join) => Ch::Joined,
        (St::Invite, St::Leave) => {
            if sender == state_key {
                Ch::InvitationRevoked
            } else {
                Ch::InvitationRejected
            }
        }
        (St::Invite, St::Ban) | (St::Leave, St::Ban) => Ch::Banned,
        (St::Join, St::Invite) | (St::Ban, St::Invite) | (St::Ban, St::Join) => Ch::Error,
        (St::Join, St::Join) => Ch::ProfileChanged {
            displayname_changed: prev_content.displayname != content.displayname,
            avatar_url_changed: prev_content.avatar_url != content.avatar_url,
        },
        (St::Join, St::Leave) => {
            if sender == state_key {
                Ch::Left
            } else {
                Ch::Kicked
            }
        }
        (St::Join, St::Ban) => Ch::KickedAndBanned,
        (St::Leave, St::Invite) => Ch::Invited,
        (St::Ban, St::Leave) => Ch::Unbanned,
        _ => Ch::NotImplemented,
    }
}

impl MemberEvent {
    /// Helper function for membership change. Check [the specification][spec] for details.
    ///
    /// [spec]: https://matrix.org/docs/spec/client_server/r0.6.1#m-room-member
    pub fn membership_change(&self) -> MembershipChange {
        membership_change(&self.content, self.prev_content.as_ref(), &self.sender, &self.state_key)
    }
}

impl SyncStateEvent<MemberEventContent> {
    /// Helper function for membership change. Check [the specification][spec] for details.
    ///
    /// [spec]: https://matrix.org/docs/spec/client_server/r0.6.1#m-room-member
    pub fn membership_change(&self) -> MembershipChange {
        membership_change(&self.content, self.prev_content.as_ref(), &self.sender, &self.state_key)
    }
}

impl StrippedStateEvent<MemberEventContent> {
    /// Helper function for membership change. Check [the specification][spec] for details.
    ///
    /// [spec]: https://matrix.org/docs/spec/client_server/r0.6.1#m-room-member
    pub fn membership_change(&self) -> MembershipChange {
        membership_change(&self.content, None, &self.sender, &self.state_key)
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use maplit::btreemap;
    use matches::assert_matches;
    use ruma_common::MilliSecondsSinceUnixEpoch;
    use ruma_identifiers::{server_name, server_signing_key_id};
    use ruma_serde::Raw;
    use serde_json::{from_value as from_json_value, json};

    use super::{MemberEventContent, MembershipState, SignedContent, ThirdPartyInvite};
    use crate::StateEvent;

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
            from_json_value::<Raw<StateEvent<MemberEventContent>>>(json)
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
                    ..
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: None,
            } if event_id == "$h29iv0s8:example.com"
                && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
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
            from_json_value::<Raw<StateEvent<MemberEventContent>>>(json)
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
                    ..
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
                    ..
                }),
            } if event_id == "$h29iv0s8:example.com"
                && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
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
            from_json_value::<Raw<StateEvent<MemberEventContent>>>(json)
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
                    ..
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: None,
            } if avatar_url.to_string() == "mxc://example.org/SEsfnsuifSDFSSEF"
                && displayname == "Alice Margatroid"
                && third_party_displayname == "alice"
                && mxid == "@alice:example.org"
                && signatures == btreemap! {
                    server_name!("magic.forest") => btreemap! {
                        server_signing_key_id!("ed25519:3") => "foobar".to_owned()
                    }
                }
                && token == "abc123"
                && event_id == "$143273582443PhrSn:example.org"
                && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(233))
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
            from_json_value::<Raw<StateEvent<MemberEventContent>>>(json)
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
                    ..
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
                    ..
                }),
            } if event_id == "$143273582443PhrSn:example.org"
                && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(233))
                && room_id == "!jEsUZKDJdhlrceRyVU:example.org"
                && sender == "@alice:example.org"
                && state_key == "@alice:example.org"
                && unsigned.is_empty()
                && avatar_url.to_string() == "mxc://example.org/SEsfnsuifSDFSSEF"
                && displayname == "Alice Margatroid"
                && third_party_displayname == "alice"
                && mxid == "@alice:example.org"
                && signatures == btreemap! {
                    server_name!("magic.forest") => btreemap! {
                        server_signing_key_id!("ed25519:3") => "foobar".to_owned()
                    }
                }
                && token == "abc123"
        );

        #[cfg(feature = "compat")]
        assert_matches!(
            from_json_value::<Raw<StateEvent<MemberEventContent>>>(json!({
                "type": "m.room.member",
                "content": {
                    "membership": "join"
                },
                "event_id": "$143273582443PhrSn:example.org",
                "origin_server_ts": 233,
                "prev_content": {
                    "avatar_url": "",
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
            }))
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
                    ..
                },
                event_id,
                origin_server_ts,
                room_id,
                sender,
                state_key,
                unsigned,
                prev_content: Some(MemberEventContent {
                    avatar_url: None,
                    displayname: Some(displayname),
                    is_direct: Some(true),
                    membership: MembershipState::Invite,
                    third_party_invite: Some(ThirdPartyInvite {
                        display_name: third_party_displayname,
                        signed: SignedContent { mxid, signatures, token },
                    }),
                    ..
                }),
            } if event_id == "$143273582443PhrSn:example.org"
                && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(233))
                && room_id == "!jEsUZKDJdhlrceRyVU:example.org"
                && sender == "@alice:example.org"
                && state_key == "@alice:example.org"
                && unsigned.is_empty()
                && displayname == "Alice Margatroid"
                && third_party_displayname == "alice"
                && mxid == "@alice:example.org"
                && signatures == btreemap! {
                    server_name!("magic.forest") => btreemap! {
                        server_signing_key_id!("ed25519:3") => "foobar".to_owned()
                    }
                }
                && token == "abc123"
        );
    }
}
