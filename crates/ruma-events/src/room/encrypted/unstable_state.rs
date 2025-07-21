use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    room::encrypted::EncryptedEventScheme, EventContent, PossiblyRedactedStateEventContent,
    RedactContent, RedactedStateEventContent, StateEventType, StaticEventContent,
};

/// The content of an `m.room.encrypted` state event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = State, state_key_type = String, custom_redacted, custom_possibly_redacted)]
pub struct StateRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,
}

/// Redacted form of [StateRoomEncryptedEventContent].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RedactedStateRoomEncryptedEventContent;

impl RedactedStateEventContent for RedactedStateRoomEncryptedEventContent {
    type StateKey = String;
}

impl EventContent for RedactedStateRoomEncryptedEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::StateRoomEncrypted
    }
}

impl RedactContent for StateRoomEncryptedEventContent {
    type Redacted = RedactedStateRoomEncryptedEventContent;

    fn redact(self, _: &ruma_common::RoomVersionId) -> Self::Redacted {
        todo!("Redaction of encrypted state events")
    }
}

/// The PossiblyRedacted form of [StateRoomEncryptedEventContent].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PossiblyRedactedStateRoomEncryptedEventContent;

impl EventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    type EventType = StateEventType;

    fn event_type(&self) -> Self::EventType {
        StateEventType::StateRoomEncrypted
    }
}

impl StaticEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    const TYPE: &'static str = "m.room.encrypted";
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    type StateKey = String;
}
