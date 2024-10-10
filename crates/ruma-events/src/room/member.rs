//! Types for the [`m.room.member`] event.
//!
//! [`m.room.member`]: https://spec.matrix.org/latest/client-server-api/#mroommember

use std::collections::BTreeMap;

use js_int::Int;
use ruma_common::{
    serde::{CanBeEmpty, Raw, StringEnum},
    OwnedMxcUri, OwnedServerName, OwnedServerSigningKeyId, OwnedTransactionId, OwnedUserId,
    RoomVersionId, UserId,
};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    AnyStrippedStateEvent, BundledStateRelations, EventContent, PossiblyRedactedStateEventContent,
    PrivOwnedStr, RedactContent, RedactedStateEventContent, StateEventType,
};

mod change;

use self::change::membership_change;
pub use self::change::{Change, MembershipChange, MembershipDetails};

/// The content of an `m.room.member` event.
///
/// The current membership state of a user in the room.
///
/// Adjusts the membership state for a user in a room. It is preferable to use the membership
/// APIs (`/rooms/<room id>/invite` etc) when performing membership actions rather than
/// adjusting the state directly as there are a restricted set of valid transformations. For
/// example, user A cannot force user B to join a room, and trying to force this state change
/// directly will fail.
///
/// This event may also include an `invite_room_state` key inside the event's unsigned data, but
/// Ruma doesn't currently expose this; see [#998](https://github.com/ruma/ruma/issues/998).
///
/// The user for which a membership applies is represented by the `state_key`. Under some
/// conditions, the `sender` and `state_key` may not match - this may be interpreted as the
/// `sender` affecting the membership state of the `state_key` user.
///
/// The membership for a given user can change over time. Previous membership can be retrieved
/// from the `prev_content` object on an event. If not present, the user's previous membership
/// must be assumed as leave.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(
    type = "m.room.member",
    kind = State,
    state_key_type = OwnedUserId,
    unsigned_type = RoomMemberUnsigned,
    custom_redacted,
    custom_possibly_redacted,
)]
pub struct RoomMemberEventContent {
    /// The avatar URL for this user, if any.
    ///
    /// This is added by the homeserver. If you activate the `compat-empty-string-null` feature,
    /// this field being an empty string in JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// The display name for this user, if any.
    ///
    /// This is added by the homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayname: Option<String>,

    /// Flag indicating whether the room containing this event was created with the intention of
    /// being a direct chat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_direct: Option<bool>,

    /// The membership state of this user.
    pub membership: MembershipState,

    /// If this member event is the successor to a third party invitation, this field will
    /// contain information about that invitation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_invite: Option<ThirdPartyInvite>,

    /// The [BlurHash](https://blurha.sh) for the avatar pointed to by `avatar_url`.
    ///
    /// This uses the unstable prefix in
    /// [MSC2448](https://github.com/matrix-org/matrix-spec-proposals/pull/2448).
    #[cfg(feature = "unstable-msc2448")]
    #[serde(rename = "xyz.amorgan.blurhash", skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,

    /// User-supplied text for why their membership has changed.
    ///
    /// For kicks and bans, this is typically the reason for the kick or ban. For other membership
    /// changes, this is a way for the user to communicate their intent without having to send a
    /// message to the room, such as in a case where Bob rejects an invite from Alice about an
    /// upcoming concert, but can't make it that day.
    ///
    /// Clients are not recommended to show this reason to users when receiving an invite due to
    /// the potential for spam and abuse. Hiding the reason behind a button or other component
    /// is recommended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Arbitrarily chosen `UserId` (MxID) of a local user who can send an invite.
    #[serde(rename = "join_authorised_via_users_server")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_authorized_via_users_server: Option<OwnedUserId>,
}

impl RoomMemberEventContent {
    /// Creates a new `RoomMemberEventContent` with the given membership state.
    pub fn new(membership: MembershipState) -> Self {
        Self {
            membership,
            avatar_url: None,
            displayname: None,
            is_direct: None,
            third_party_invite: None,
            #[cfg(feature = "unstable-msc2448")]
            blurhash: None,
            reason: None,
            join_authorized_via_users_server: None,
        }
    }

    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        MembershipDetails {
            avatar_url: self.avatar_url.as_deref(),
            displayname: self.displayname.as_deref(),
            membership: &self.membership,
        }
    }

    /// Helper function for membership change.
    ///
    /// This requires data from the full event:
    ///
    /// * The previous details computed from `event.unsigned.prev_content`,
    /// * The sender of the event,
    /// * The state key of the event.
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change<'a>(
        &'a self,
        prev_details: Option<MembershipDetails<'a>>,
        sender: &UserId,
        state_key: &UserId,
    ) -> MembershipChange<'a> {
        membership_change(self.details(), prev_details, sender, state_key)
    }
}

impl RedactContent for RoomMemberEventContent {
    type Redacted = RedactedRoomMemberEventContent;

    fn redact(self, version: &RoomVersionId) -> RedactedRoomMemberEventContent {
        RedactedRoomMemberEventContent {
            membership: self.membership,
            third_party_invite: self.third_party_invite.and_then(|i| i.redact(version)),
            join_authorized_via_users_server: match version {
                RoomVersionId::V1
                | RoomVersionId::V2
                | RoomVersionId::V3
                | RoomVersionId::V4
                | RoomVersionId::V5
                | RoomVersionId::V6
                | RoomVersionId::V7
                | RoomVersionId::V8 => None,
                _ => self.join_authorized_via_users_server,
            },
        }
    }
}

/// The possibly redacted form of [`RoomMemberEventContent`].
///
/// This type is used when it's not obvious whether the content is redacted or not.
pub type PossiblyRedactedRoomMemberEventContent = RoomMemberEventContent;

impl PossiblyRedactedStateEventContent for RoomMemberEventContent {
    type StateKey = OwnedUserId;
}

/// A member event that has been redacted.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedRoomMemberEventContent {
    /// The membership state of this user.
    pub membership: MembershipState,

    /// If this member event is the successor to a third party invitation, this field will
    /// contain information about that invitation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_invite: Option<RedactedThirdPartyInvite>,

    /// An arbitrary user who has the power to issue invites.
    ///
    /// This is redacted in room versions 8 and below. It is used for validating
    /// joins when the join rule is restricted.
    #[serde(rename = "join_authorised_via_users_server", skip_serializing_if = "Option::is_none")]
    pub join_authorized_via_users_server: Option<OwnedUserId>,
}

impl RedactedRoomMemberEventContent {
    /// Create a `RedactedRoomMemberEventContent` with the given membership.
    pub fn new(membership: MembershipState) -> Self {
        Self { membership, third_party_invite: None, join_authorized_via_users_server: None }
    }

    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        MembershipDetails { avatar_url: None, displayname: None, membership: &self.membership }
    }

    /// Helper function for membership change.
    ///
    /// Since redacted events don't have `unsigned.prev_content`, you have to pass the `.details()`
    /// of the previous `m.room.member` event manually (if there is a previous `m.room.member`
    /// event).
    ///
    /// This also requires data from the full event:
    ///
    /// * The sender of the event,
    /// * The state key of the event.
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change<'a>(
        &'a self,
        prev_details: Option<MembershipDetails<'a>>,
        sender: &UserId,
        state_key: &UserId,
    ) -> MembershipChange<'a> {
        membership_change(self.details(), prev_details, sender, state_key)
    }
}

impl EventContent for RedactedRoomMemberEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomMember
    }
}

impl RedactedStateEventContent for RedactedRoomMemberEventContent {
    type StateKey = OwnedUserId;
}

impl RoomMemberEvent {
    /// Obtain the membership state, regardless of whether this event is redacted.
    pub fn membership(&self) -> &MembershipState {
        match self {
            Self::Original(ev) => &ev.content.membership,
            Self::Redacted(ev) => &ev.content.membership,
        }
    }
}

impl SyncRoomMemberEvent {
    /// Obtain the membership state, regardless of whether this event is redacted.
    pub fn membership(&self) -> &MembershipState {
        match self {
            Self::Original(ev) => &ev.content.membership,
            Self::Redacted(ev) => &ev.content.membership,
        }
    }
}

/// The membership state of a user.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
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
    _Custom(PrivOwnedStr),
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

    /// Transform `self` into a redacted form (removing most or all fields) according to the spec.
    ///
    /// Returns `None` if the field for this object was redacted in the given room version,
    /// otherwise returns the redacted form.
    fn redact(self, version: &RoomVersionId) -> Option<RedactedThirdPartyInvite> {
        match version {
            RoomVersionId::V1
            | RoomVersionId::V2
            | RoomVersionId::V3
            | RoomVersionId::V4
            | RoomVersionId::V5
            | RoomVersionId::V6
            | RoomVersionId::V7
            | RoomVersionId::V8
            | RoomVersionId::V9
            | RoomVersionId::V10 => None,
            _ => Some(RedactedThirdPartyInvite { signed: self.signed }),
        }
    }
}

/// Redacted information about a third party invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RedactedThirdPartyInvite {
    /// A block of content which has been signed, which servers can use to verify the event.
    ///
    /// Clients should ignore this.
    pub signed: SignedContent,
}

/// A block of content which has been signed, which servers can use to verify a third party
/// invitation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SignedContent {
    /// The invited Matrix user ID.
    ///
    /// Must be equal to the user_id property of the event.
    pub mxid: OwnedUserId,

    /// A single signature from the verifying server, in the format specified by the Signing Events
    /// section of the server-server API.
    pub signatures: BTreeMap<OwnedServerName, BTreeMap<OwnedServerSigningKeyId, String>>,

    /// The token property of the containing `third_party_invite` object.
    pub token: String,
}

impl SignedContent {
    /// Creates a new `SignedContent` with the given mxid, signature and token.
    pub fn new(
        signatures: BTreeMap<OwnedServerName, BTreeMap<OwnedServerSigningKeyId, String>>,
        mxid: OwnedUserId,
        token: String,
    ) -> Self {
        Self { mxid, signatures, token }
    }
}

impl OriginalRoomMemberEvent {
    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        self.content.details()
    }

    /// Get a reference to the `prev_content` in unsigned, if it exists.
    ///
    /// Shorthand for `event.unsigned.prev_content.as_ref()`
    pub fn prev_content(&self) -> Option<&RoomMemberEventContent> {
        self.unsigned.prev_content.as_ref()
    }

    fn prev_details(&self) -> Option<MembershipDetails<'_>> {
        self.prev_content().map(|c| c.details())
    }

    /// Helper function for membership change.
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change(&self) -> MembershipChange<'_> {
        membership_change(self.details(), self.prev_details(), &self.sender, &self.state_key)
    }
}

impl RedactedRoomMemberEvent {
    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        self.content.details()
    }

    /// Helper function for membership change.
    ///
    /// Since redacted events don't have `unsigned.prev_content`, you have to pass the `.details()`
    /// of the previous `m.room.member` event manually (if there is a previous `m.room.member`
    /// event).
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change<'a>(
        &'a self,
        prev_details: Option<MembershipDetails<'a>>,
    ) -> MembershipChange<'a> {
        membership_change(self.details(), prev_details, &self.sender, &self.state_key)
    }
}

impl OriginalSyncRoomMemberEvent {
    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        self.content.details()
    }

    /// Get a reference to the `prev_content` in unsigned, if it exists.
    ///
    /// Shorthand for `event.unsigned.prev_content.as_ref()`
    pub fn prev_content(&self) -> Option<&RoomMemberEventContent> {
        self.unsigned.prev_content.as_ref()
    }

    fn prev_details(&self) -> Option<MembershipDetails<'_>> {
        self.prev_content().map(|c| c.details())
    }

    /// Helper function for membership change.
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change(&self) -> MembershipChange<'_> {
        membership_change(self.details(), self.prev_details(), &self.sender, &self.state_key)
    }
}

impl RedactedSyncRoomMemberEvent {
    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        self.content.details()
    }

    /// Helper function for membership change.
    ///
    /// Since redacted events don't have `unsigned.prev_content`, you have to pass the `.details()`
    /// of the previous `m.room.member` event manually (if there is a previous `m.room.member`
    /// event).
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change<'a>(
        &'a self,
        prev_details: Option<MembershipDetails<'a>>,
    ) -> MembershipChange<'a> {
        membership_change(self.details(), prev_details, &self.sender, &self.state_key)
    }
}

impl StrippedRoomMemberEvent {
    /// Obtain the details about this event that are required to calculate a membership change.
    ///
    /// This is required when you want to calculate the change a redacted `m.room.member` event
    /// made.
    pub fn details(&self) -> MembershipDetails<'_> {
        self.content.details()
    }

    /// Helper function for membership change.
    ///
    /// Since stripped events don't have `unsigned.prev_content`, you have to pass the `.details()`
    /// of the previous `m.room.member` event manually (if there is a previous `m.room.member`
    /// event).
    ///
    /// Check [the specification][spec] for details.
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommember
    pub fn membership_change<'a>(
        &'a self,
        prev_details: Option<MembershipDetails<'a>>,
    ) -> MembershipChange<'a> {
        membership_change(self.details(), prev_details, &self.sender, &self.state_key)
    }
}

/// Extra information about a message event that is not incorporated into the event's hash.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomMemberUnsigned {
    /// The time in milliseconds that has elapsed since the event was sent.
    ///
    /// This field is generated by the local homeserver, and may be incorrect if the local time on
    /// at least one of the two servers is out of sync, which can cause the age to either be
    /// negative or greater than it actually is.
    pub age: Option<Int>,

    /// The client-supplied transaction ID, if the client being given the event is the same one
    /// which sent it.
    pub transaction_id: Option<OwnedTransactionId>,

    /// Optional previous content of the event.
    pub prev_content: Option<PossiblyRedactedRoomMemberEventContent>,

    /// State events to assist the receiver in identifying the room.
    #[serde(default)]
    pub invite_room_state: Vec<Raw<AnyStrippedStateEvent>>,

    /// [Bundled aggregations] of related child events.
    ///
    /// [Bundled aggregations]: https://spec.matrix.org/latest/client-server-api/#aggregations-of-child-events
    #[serde(rename = "m.relations", default)]
    pub relations: BundledStateRelations,
}

impl RoomMemberUnsigned {
    /// Create a new `Unsigned` with fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CanBeEmpty for RoomMemberUnsigned {
    /// Whether this unsigned data is empty (all fields are `None`).
    ///
    /// This method is used to determine whether to skip serializing the `unsigned` field in room
    /// events. Do not use it to determine whether an incoming `unsigned` field was present - it
    /// could still have been present but contained none of the known fields.
    fn is_empty(&self) -> bool {
        self.age.is_none()
            && self.transaction_id.is_none()
            && self.prev_content.is_none()
            && self.invite_room_state.is_empty()
            && self.relations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use maplit::btreemap;
    use ruma_common::{
        mxc_uri, serde::CanBeEmpty, server_name, server_signing_key_version, user_id,
        MilliSecondsSinceUnixEpoch, ServerSigningKeyId, SigningKeyAlgorithm,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{MembershipState, RoomMemberEventContent};
    use crate::OriginalStateEvent;

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
            "state_key": "@carl:example.com"
        });

        let ev = from_json_value::<OriginalStateEvent<RoomMemberEventContent>>(json).unwrap();
        assert_eq!(ev.event_id, "$h29iv0s8:example.com");
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
        assert_eq!(ev.room_id, "!n8f893n9:example.com");
        assert_eq!(ev.sender, "@carl:example.com");
        assert_eq!(ev.state_key, "@carl:example.com");
        assert!(ev.unsigned.is_empty());

        assert_eq!(ev.content.avatar_url, None);
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.is_direct, None);
        assert_eq!(ev.content.membership, MembershipState::Join);
        assert_matches!(ev.content.third_party_invite, None);
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
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "@carl:example.com",
            "unsigned": {
                "prev_content": {
                    "membership": "join"
                },
            },
        });

        let ev = from_json_value::<OriginalStateEvent<RoomMemberEventContent>>(json).unwrap();
        assert_eq!(ev.event_id, "$h29iv0s8:example.com");
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
        assert_eq!(ev.room_id, "!n8f893n9:example.com");
        assert_eq!(ev.sender, "@carl:example.com");
        assert_eq!(ev.state_key, "@carl:example.com");

        assert_eq!(ev.content.avatar_url, None);
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.is_direct, None);
        assert_eq!(ev.content.membership, MembershipState::Join);
        assert_matches!(ev.content.third_party_invite, None);

        let prev_content = ev.unsigned.prev_content.unwrap();
        assert_eq!(prev_content.avatar_url, None);
        assert_eq!(prev_content.displayname, None);
        assert_eq!(prev_content.is_direct, None);
        assert_eq!(prev_content.membership, MembershipState::Join);
        assert_matches!(prev_content.third_party_invite, None);
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

        let ev = from_json_value::<OriginalStateEvent<RoomMemberEventContent>>(json).unwrap();
        assert_eq!(ev.event_id, "$143273582443PhrSn:example.org");
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(233)));
        assert_eq!(ev.room_id, "!jEsUZKDJdhlrceRyVU:example.org");
        assert_eq!(ev.sender, "@alice:example.org");
        assert_eq!(ev.state_key, "@alice:example.org");
        assert!(ev.unsigned.is_empty());

        assert_eq!(
            ev.content.avatar_url.as_deref(),
            Some(mxc_uri!("mxc://example.org/SEsfnsuifSDFSSEF"))
        );
        assert_eq!(ev.content.displayname.as_deref(), Some("Alice Margatroid"));
        assert_eq!(ev.content.is_direct, Some(true));
        assert_eq!(ev.content.membership, MembershipState::Invite);

        let third_party_invite = ev.content.third_party_invite.unwrap();
        assert_eq!(third_party_invite.display_name, "alice");
        assert_eq!(third_party_invite.signed.mxid, "@alice:example.org");
        assert_eq!(
            third_party_invite.signed.signatures,
            btreemap! {
                server_name!("magic.forest").to_owned() => btreemap! {
                    ServerSigningKeyId::from_parts(
                        SigningKeyAlgorithm::Ed25519,
                        server_signing_key_version!("3")
                    ) => "foobar".to_owned()
                }
            }
        );
        assert_eq!(third_party_invite.signed.token, "abc123");
    }

    #[test]
    fn serde_with_prev_content_full() {
        let json = json!({
            "type": "m.room.member",
            "content": {
                "membership": "join",
            },
            "event_id": "$143273582443PhrSn:example.org",
            "origin_server_ts": 233,
            "room_id": "!jEsUZKDJdhlrceRyVU:example.org",
            "sender": "@alice:example.org",
            "state_key": "@alice:example.org",
            "unsigned": {
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
                                    "ed25519:3": "foobar",
                                },
                            },
                            "token": "abc123"
                        },
                    },
                },
            },
        });

        let ev = from_json_value::<OriginalStateEvent<RoomMemberEventContent>>(json).unwrap();
        assert_eq!(ev.event_id, "$143273582443PhrSn:example.org");
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(233)));
        assert_eq!(ev.room_id, "!jEsUZKDJdhlrceRyVU:example.org");
        assert_eq!(ev.sender, "@alice:example.org");
        assert_eq!(ev.state_key, "@alice:example.org");

        assert_eq!(ev.content.avatar_url, None);
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.is_direct, None);
        assert_eq!(ev.content.membership, MembershipState::Join);
        assert_matches!(ev.content.third_party_invite, None);

        let prev_content = ev.unsigned.prev_content.unwrap();
        assert_eq!(
            prev_content.avatar_url.as_deref(),
            Some(mxc_uri!("mxc://example.org/SEsfnsuifSDFSSEF"))
        );
        assert_eq!(prev_content.displayname.as_deref(), Some("Alice Margatroid"));
        assert_eq!(prev_content.is_direct, Some(true));
        assert_eq!(prev_content.membership, MembershipState::Invite);

        let third_party_invite = prev_content.third_party_invite.unwrap();
        assert_eq!(third_party_invite.display_name, "alice");
        assert_eq!(third_party_invite.signed.mxid, "@alice:example.org");
        assert_eq!(
            third_party_invite.signed.signatures,
            btreemap! {
                server_name!("magic.forest").to_owned() => btreemap! {
                    ServerSigningKeyId::from_parts(
                        SigningKeyAlgorithm::Ed25519,
                        server_signing_key_version!("3")
                    ) => "foobar".to_owned()
                }
            }
        );
        assert_eq!(third_party_invite.signed.token, "abc123");
    }

    #[test]
    fn serde_with_join_authorized() {
        let json = json!({
            "type": "m.room.member",
            "content": {
                "membership": "join",
                "join_authorised_via_users_server": "@notcarl:example.com"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!n8f893n9:example.com",
            "sender": "@carl:example.com",
            "state_key": "@carl:example.com"
        });

        let ev = from_json_value::<OriginalStateEvent<RoomMemberEventContent>>(json).unwrap();
        assert_eq!(ev.event_id, "$h29iv0s8:example.com");
        assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
        assert_eq!(ev.room_id, "!n8f893n9:example.com");
        assert_eq!(ev.sender, "@carl:example.com");
        assert_eq!(ev.state_key, "@carl:example.com");
        assert!(ev.unsigned.is_empty());

        assert_eq!(ev.content.avatar_url, None);
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.is_direct, None);
        assert_eq!(ev.content.membership, MembershipState::Join);
        assert_matches!(ev.content.third_party_invite, None);
        assert_eq!(
            ev.content.join_authorized_via_users_server.as_deref(),
            Some(user_id!("@notcarl:example.com"))
        );
    }
}
