//! The names of the `Any*Event` enums. The event_enum! macro uses these names to generate
//! certain code for certain enums. If the names change this is the one source of truth,
//! most comparisons and branching uses these constants.

// State events
pub const ANY_STATE_EVENT: &str = "AnyStateEvent";

pub const ANY_SYNC_STATE_EVENT: &str = "AnyStateEventStub";

pub const ANY_STRIPPED_STATE_EVENT: &str = "AnyStrippedStateEventStub";

// Message events
pub const ANY_MESSAGE_EVENT: &str = "AnyMessageEvent";

pub const ANY_SYNC_MESSAGE_EVENT: &str = "AnyMessageEventStub";

// Ephemeral events
pub const ANY_EPHEMERAL_EVENT: &str = "AnyEphemeralRoomEvent";

#[allow(dead_code)]
// This is currently not used but, left for completeness sake.
pub const ANY_SYNC_EPHEMERAL_EVENT: &str = "AnyEphemeralRoomEventStub";

// Basic event
pub const ANY_BASIC_EVENT: &str = "AnyBasicEvent";

// To device event
pub const ANY_TO_DEVICE_EVENT: &str = "AnyToDeviceEvent";
