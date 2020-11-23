//! Types for the *m.room.third_party_invite* event.

use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::StateEvent;

/// An invitation to a room issued to a third party identifier, rather than a matrix user ID.
///
/// Acts as an *m.room.member* invite event, where there isn't a target user_id to invite. This
/// event contains a token and a public key whose private key must be used to sign the token.
/// Any user who can present that signature may use this invitation to join the target room.
pub type ThirdPartyInviteEvent = StateEvent<ThirdPartyInviteEventContent>;

/// The payload for `ThirdPartyInviteEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[ruma_event(type = "m.room.third_party_invite")]
pub struct ThirdPartyInviteEventContent {
    /// A user-readable string which represents the user who has been invited.
    #[cfg_attr(feature = "unstable-synapse-quirks", serde(default))]
    pub display_name: String,

    /// A URL which can be fetched to validate whether the key has been revoked.
    #[cfg_attr(feature = "unstable-synapse-quirks", serde(default))]
    pub key_validity_url: String,

    /// A Base64-encoded Ed25519 key with which the token must be signed.
    #[cfg_attr(feature = "unstable-synapse-quirks", serde(default))]
    pub public_key: String,

    /// Keys with which the token may be signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_keys: Option<Vec<PublicKey>>,
}

/// A public key for signing a third party invite token.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey {
    /// An optional URL which can be fetched to validate whether the key has been revoked.
    ///
    /// The URL must return a JSON object containing a boolean property named 'valid'.
    /// If this URL is absent, the key must be considered valid indefinitely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_validity_url: Option<String>,

    /// A Base64-encoded Ed25519 key with which the token must be signed.
    pub public_key: String,
}
