//! Types for the *m.room.third_party_invite* event.

use core::EventType;

/// An invitation to a room issued to a third party identifier, rather than a matrix user ID.
///
/// Acts as an *m.room.member* invite event, where there isn't a target user_id to invite. This
/// event contains a token and a public key whose private key must be used to sign the token. Any
/// user who can present that signature may use this invitation to join the target room.
pub struct ThirdPartyInviteEvent {
    content: ThirdPartyInviteEventContent,
    event_id: String,
    event_type: EventType,
    prev_content: Option<ThirdPartyInviteEventContent>,
    room_id: String,
    state_key: String,
    user_id: String,
}

/// The payload of a `ThirdPartyInviteEvent`.
pub struct ThirdPartyInviteEventContent {
    display_name: String,
    key_validity_url: String,
    public_key: String,
}
