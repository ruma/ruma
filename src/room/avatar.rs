//! Types for the *m.room.avatar* event.

use core::{Event, RoomEvent, StateEvent};
use super::ImageInfo;

/// A picture that is associated with the room.
///
/// This can be displayed alongside the room information.
pub struct AvatarEvent<'a, 'b> {
    content: AvatarEventContent<'a>,
    event_id: &'a str,
    prev_content: Option<AvatarEventContent<'b>>,
    room_id: &'a str,
    user_id: &'a str,
}

impl<'a, 'b> Event<'a, AvatarEventContent<'a>> for AvatarEvent<'a, 'b> {
    fn content(&'a self) -> &'a AvatarEventContent<'a> {
        &self.content
    }

    fn event_type(&self) -> &'static str {
        "m.room.avatar"
    }
}

impl<'a, 'b> RoomEvent<'a, AvatarEventContent<'a>> for AvatarEvent<'a, 'b> {
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

impl<'a, 'b> StateEvent<'a, 'b, AvatarEventContent<'a>> for AvatarEvent<'a, 'b> {
    fn prev_content(&'a self) -> Option<&'b AvatarEventContent> {
        match self.prev_content {
            Some(ref prev_content) => Some(prev_content),
            None => None,
        }
    }

    fn state_key(&self) -> &'a str {
        ""
    }
}
/// The payload of an `AvatarEvent`.
pub struct AvatarEventContent<'a> {
    info: &'a ImageInfo<'a>,
    thumbnail_info: &'a ImageInfo<'a>,
    thumbnail_url: &'a str,
    url: &'a str,
}
