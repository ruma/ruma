//! Types for the *m.room.avatar* event.

use ruma_events_macros::ruma_event;
use serde_json::value::RawValue as RawJsonValue;

use super::ImageInfo;
use crate::{
    error::{InvalidEvent, InvalidEventKind},
    EventContent, EventJson, RoomEventContent, StateEventContent,
};

ruma_event! {
    /// A picture that is associated with the room.
    ///
    /// This can be displayed alongside the room information.
    AvatarEvent {
        kind: StateEvent,
        event_type: "m.room.avatar",
        content: {
            /// Information about the avatar image.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub info: Option<Box<ImageInfo>>,

            /// Information about the avatar thumbnail image.
            /// URL of the avatar image.
            pub url: String,
        },
    }
}
