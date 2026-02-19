//! Types to deserialize `m.room.member` events.

use std::ops::Deref;

use ruma_common::{CanonicalJsonObject, UserId, serde::from_raw_json_value};
use ruma_events::room::member::MembershipState;
use ruma_signatures::canonical_json;
use serde::Deserialize;
use serde_json::value::RawValue as RawJsonValue;

use super::Event;

/// A helper type for an [`Event`] of type `m.room.member`.
///
/// This is a type that deserializes each field lazily, as requested.
#[derive(Debug, Clone)]
pub struct RoomMemberEvent<E: Event>(E);

impl<E: Event> RoomMemberEvent<E> {
    /// Construct a new `RoomMemberEvent` around the given event.
    pub fn new(event: E) -> Self {
        Self(event)
    }

    /// The membership of the user.
    pub fn membership(&self) -> Result<MembershipState, String> {
        RoomMemberEventContent(self.content()).membership()
    }

    /// If this is a `join` event, the ID of a user on the homeserver that authorized it.
    pub fn join_authorised_via_users_server(&self) -> Result<Option<UserId>, String> {
        RoomMemberEventContent(self.content()).join_authorised_via_users_server()
    }

    /// If this is an `invite` event, details about the third-party invite that resulted in this
    /// event.
    pub(crate) fn third_party_invite(&self) -> Result<Option<ThirdPartyInvite>, String> {
        RoomMemberEventContent(self.content()).third_party_invite()
    }
}

impl<E: Event> Deref for RoomMemberEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Helper trait for `Option<RoomMemberEvent<E>>`.
pub(crate) trait RoomMemberEventOptionExt {
    /// The membership of the user.
    ///
    /// Defaults to `leave` if there is no `m.room.member` event.
    fn membership(&self) -> Result<MembershipState, String>;
}

impl<E: Event> RoomMemberEventOptionExt for Option<RoomMemberEvent<E>> {
    fn membership(&self) -> Result<MembershipState, String> {
        match self {
            Some(event) => event.membership(),
            None => Ok(MembershipState::Leave),
        }
    }
}

/// A helper type for the raw JSON content of an event of type `m.room.member`.
pub(crate) struct RoomMemberEventContent<'a>(&'a RawJsonValue);

impl<'a> RoomMemberEventContent<'a> {
    /// Construct a new `RoomMemberEventContent` around the given raw JSON content.
    pub(crate) fn new(content: &'a RawJsonValue) -> Self {
        Self(content)
    }
}

impl RoomMemberEventContent<'_> {
    /// The membership of the user.
    pub(crate) fn membership(&self) -> Result<MembershipState, String> {
        #[derive(Deserialize)]
        struct RoomMemberContentMembership {
            membership: MembershipState,
        }

        let content: RoomMemberContentMembership =
            from_raw_json_value(self.0).map_err(|err: serde_json::Error| {
                format!("missing or invalid `membership` field in `m.room.member` event: {err}")
            })?;
        Ok(content.membership)
    }

    /// If this is a `join` event, the ID of a user on the homeserver that authorized it.
    pub(crate) fn join_authorised_via_users_server(&self) -> Result<Option<UserId>, String> {
        #[derive(Deserialize)]
        struct RoomMemberContentJoinAuthorizedViaUsersServer {
            join_authorised_via_users_server: Option<UserId>,
        }

        let content: RoomMemberContentJoinAuthorizedViaUsersServer = from_raw_json_value(self.0)
            .map_err(|err: serde_json::Error| {
                format!(
                "invalid `join_authorised_via_users_server` field in `m.room.member` event: {err}"
            )
            })?;
        Ok(content.join_authorised_via_users_server)
    }

    /// If this is an `invite` event, details about the third-party invite that resulted in this
    /// event.
    pub(crate) fn third_party_invite(&self) -> Result<Option<ThirdPartyInvite>, String> {
        #[derive(Deserialize)]
        struct RoomMemberContentThirdPartyInvite {
            third_party_invite: Option<ThirdPartyInvite>,
        }

        let content: RoomMemberContentThirdPartyInvite =
            from_raw_json_value(self.0).map_err(|err: serde_json::Error| {
                format!("invalid `third_party_invite` field in `m.room.member` event: {err}")
            })?;
        Ok(content.third_party_invite)
    }
}

/// Details about a third-party invite.
#[derive(Deserialize)]
pub(crate) struct ThirdPartyInvite {
    /// Signed details about the third-party invite.
    signed: CanonicalJsonObject,
}

impl ThirdPartyInvite {
    /// The unique identifier for the third-party invite.
    pub(crate) fn token(&self) -> Result<&str, String> {
        let Some(token_value) = self.signed.get("token") else {
            return Err("missing `token` field in `third_party_invite.signed` \
                        of `m.room.member` event"
                .into());
        };

        token_value.as_str().ok_or_else(|| {
            format!(
                "unexpected format of `token` field in `third_party_invite.signed` \
                 of `m.room.member` event: expected string, got {token_value:?}"
            )
        })
    }

    /// The Matrix ID of the user that was invited.
    pub(crate) fn mxid(&self) -> Result<&str, String> {
        let Some(mxid_value) = self.signed.get("mxid") else {
            return Err("missing `mxid` field in `third_party_invite.signed` \
                        of `m.room.member` event"
                .into());
        };

        mxid_value.as_str().ok_or_else(|| {
            format!(
                "unexpected format of `mxid` field in `third_party_invite.signed` \
                 of `m.room.member` event: expected string, got {mxid_value:?}"
            )
        })
    }

    /// The signatures of the event.
    pub(crate) fn signatures(&self) -> Result<&CanonicalJsonObject, String> {
        let Some(signatures_value) = self.signed.get("signatures") else {
            return Err("missing `signatures` field in `third_party_invite.signed` \
                        of `m.room.member` event"
                .into());
        };

        signatures_value.as_object().ok_or_else(|| {
            format!(
                "unexpected format of `signatures` field in `third_party_invite.signed` \
                 of `m.room.member` event: expected object, got {signatures_value:?}"
            )
        })
    }

    /// The `signed` object as canonical JSON string to verify the signatures.
    pub(crate) fn signed_canonical_json(&self) -> Result<String, String> {
        canonical_json(&self.signed).map_err(|error| {
            format!("invalid `third_party_invite.signed` field in `m.room.member` event: {error}")
        })
    }
}
