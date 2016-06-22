//! Types for the *m.room.third_party_invite* event.

use StateEvent;

/// An invitation to a room issued to a third party identifier, rather than a matrix user ID.
///
/// Acts as an *m.room.member* invite event, where there isn't a target user_id to invite. This
/// event contains a token and a public key whose private key must be used to sign the token. Any
/// user who can present that signature may use this invitation to join the target room.
pub type ThirdPartyInviteEvent = StateEvent<ThirdPartyInviteEventContent>;

/// The payload of a `ThirdPartyInviteEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ThirdPartyInviteEventContent {
    pub display_name: String,
    pub key_validity_url: String,
    pub public_key: String,
}
