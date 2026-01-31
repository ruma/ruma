//! Types for the [`m.room.third_party_invite`] event.
//!
//! [`m.room.third_party_invite`]: https://spec.matrix.org/latest/client-server-api/#mroomthird_party_invite

use ruma_common::third_party_invite::IdentityServerBase64PublicKey;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.room.third_party_invite` event.
///
/// An invitation to a room issued to a third party identifier, rather than a matrix user ID.
///
/// Acts as an `m.room.member` invite event, where there isn't a target user_id to invite. This
/// event contains a token and a public key whose private key must be used to sign the token.
/// Any user who can present that signature may use this invitation to join the target room.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.third_party_invite", kind = State, state_key_type = String)]
pub struct RoomThirdPartyInviteEventContent {
    /// A user-readable string which represents the user who has been invited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A URL which can be fetched to validate whether the key has been revoked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_validity_url: Option<String>,

    /// A base64-encoded Ed25519 key with which the token must be signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<IdentityServerBase64PublicKey>,

    /// Keys with which the token may be signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_keys: Option<Vec<PublicKey>>,
}

impl RoomThirdPartyInviteEventContent {
    /// Creates a new `RoomThirdPartyInviteEventContent` with the given display name, key validity
    /// url and public key.
    pub fn new(
        display_name: String,
        key_validity_url: String,
        public_key: IdentityServerBase64PublicKey,
    ) -> Self {
        Self {
            display_name: Some(display_name),
            key_validity_url: Some(key_validity_url),
            public_key: Some(public_key),
            public_keys: None,
        }
    }
}

/// A public key for signing a third party invite token.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PublicKey {
    /// An optional URL which can be fetched to validate whether the key has been revoked.
    ///
    /// The URL must return a JSON object containing a boolean property named 'valid'.
    /// If this URL is absent, the key must be considered valid indefinitely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_validity_url: Option<String>,

    /// A base64-encoded Ed25519 key with which the token must be signed.
    pub public_key: IdentityServerBase64PublicKey,
}

impl PublicKey {
    /// Creates a new `PublicKey` with the given base64-encoded ed25519 key.
    pub fn new(public_key: IdentityServerBase64PublicKey) -> Self {
        Self { key_validity_url: None, public_key }
    }
}
