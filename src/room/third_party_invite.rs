//! Types for the *m.room.third_party_invite* event.

use core::{Event, EventType, RoomEvent, StateEvent};

/// An invitation to a room issued to a third party identifier, rather than a matrix user ID.
///
/// Acts as an *m.room.member* invite event, where there isn't a target user_id to invite. This
/// event contains a token and a public key whose private key must be used to sign the token. Any
/// user who can present that signature may use this invitation to join the target room.
pub struct ThirdPartyInviteEvent<'a, 'b> {
    content: ThirdPartyInviteEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<ThirdPartyInviteEventContent<'b>>,
    room_id: &'a str,
    state_key: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, ThirdPartyInviteEventContent<'a>> for ThirdPartyInviteEvent<'a, 'b> {
    fn content(&'a self) -> &'a ThirdPartyInviteEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> EventType {
        EventType::RoomThirdPartyInvite
    }
}

impl<'a, 'b> RoomEvent<'a, ThirdPartyInviteEventContent<'a>> for ThirdPartyInviteEvent<'a, 'b> {
    fn event_id(&'a self) -> &'a str {
        &self.event_id
    }

    fn room_id(&'a self) -> &'a str {
        &self.room_id
    }

    fn user_id(&'a self) -> &'a str {
        &self.user_id
    }
}

impl<'a, 'b> StateEvent<'a, 'b, ThirdPartyInviteEventContent<'a>> for ThirdPartyInviteEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b ThirdPartyInviteEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        &self.state_key
    }
}

/// The payload of a `ThirdPartyInviteEvent`.
pub struct ThirdPartyInviteEventContent<'a> {
    display_name: &'a str,
    key_validity_url: &'a str,
    public_key: &'a str,
}
