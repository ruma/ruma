//! The names of the `Any*Event` enums. The event_enum! macro uses these names to generate
//! certain code for certain enums. If the names change this is the one source of truth,
//! most comparisons and branching uses these constants.

#![allow(dead_code)]

// Those marked with (UNUSED) are not used but, left for completeness sake.
// If you change this please remove the (UNUSED) comment.

// State events
pub const ANY_STATE_EVENT: &str = "AnyStateEvent";

pub const ANY_SYNC_STATE_EVENT: &str = "AnyStateEventStub";

pub const ANY_STRIPPED_STATE_EVENT: &str = "AnyStrippedStateEventStub";

// Redacted state events
pub const REDACTED_STATE_EVENT: &str = "AnyRedactedStateEvent"; // (UNUSED)

pub const REDACTED_SYNC_STATE_EVENT: &str = "AnyRedactedStateEventStub"; // (UNUSED)

pub const REDACTED_STRIPPED_STATE_EVENT: &str = "AnyRedactedStrippedStateEventStub"; // (UNUSED)

// Message events
pub const ANY_MESSAGE_EVENT: &str = "AnyMessageEvent";

pub const ANY_SYNC_MESSAGE_EVENT: &str = "AnyMessageEventStub";

// Redacted message events
pub const REDACTED_MESSAGE_EVENT: &str = "AnyRedactedMessageEvent"; // (UNUSED)

pub const REDACTED_SYNC_MESSAGE_EVENT: &str = "AnyRedactedMessageEventStub"; // (UNUSED)

// Ephemeral events
pub const ANY_EPHEMERAL_EVENT: &str = "AnyEphemeralRoomEvent";

pub const ANY_SYNC_EPHEMERAL_EVENT: &str = "AnyEphemeralRoomEventStub"; // (UNUSED)

// Basic event
pub const ANY_BASIC_EVENT: &str = "AnyBasicEvent";

// To device event
pub const ANY_TO_DEVICE_EVENT: &str = "AnyToDeviceEvent";
