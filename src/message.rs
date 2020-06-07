//! An enum that represents any message event. A message event is represented by
//! a parameterized struct allowing more flexibility in whats being sent.

use std::{
    convert::TryFrom,
    time::{SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};

use crate::{MessageEventContent, RoomEventContent, UnsignedData};
use ruma_events_macros::{event_content_collection, Event};

event_content_collection! {
    /// A message event.
    name: AnyMessageEventContent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.sticker",
    ]
}

/// Message event.
#[derive(Clone, Debug, Event)]
pub struct MessageEvent<C: MessageEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
}
