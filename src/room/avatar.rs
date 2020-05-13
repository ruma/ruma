//! Types for the *m.room.avatar* event.

use ruma_events_macros::ruma_event;
use serde_json::value::RawValue as RawJsonValue;

use super::ImageInfo;
use crate::{
    error::{InvalidEvent, InvalidEventKind},
    EventContent, RoomEventContent, StateEventContent,
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

impl EventContent for AvatarEventContent {
    fn event_type(&self) -> &str {
        "m.room.avatar"
    }

    fn from_parts(event_type: &str, content: &RawJsonValue) -> Result<Self, InvalidEvent> {
        if event_type != "m.room.avatar" {
            return Err(InvalidEvent {
                kind: InvalidEventKind::Deserialization,
                message: format!("expected `m.room.avatar` found {}", event_type),
            });
        }
        serde_json::from_str::<AvatarEventContent>(content.get()).map_err(|e| InvalidEvent {
            kind: InvalidEventKind::Deserialization,
            message: e.to_string(),
        })
    }
}

impl RoomEventContent for AvatarEventContent {}

impl StateEventContent for AvatarEventContent {}
