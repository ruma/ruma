use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use crate::{
    room::encrypted::EncryptedEventScheme, False, PossiblyRedactedStateEventContent,
    StateEventType, StaticEventContent,
};

/// The content of an `m.room.encrypted` state event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.room.encrypted", kind = State, state_key_type = String, custom_possibly_redacted)]
pub struct StateRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten)]
    pub scheme: EncryptedEventScheme,
}

/// The PossiblyRedacted form of [StateRoomEncryptedEventContent].
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PossiblyRedactedStateRoomEncryptedEventContent {
    /// Algorithm-specific fields.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<EncryptedEventScheme>,
}

impl StaticEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    const TYPE: &'static str = "m.room.encrypted";
    type IsPrefix = False;
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedStateRoomEncryptedEventContent {
    type StateKey = String;

    fn event_type(&self) -> StateEventType {
        StateEventType::RoomEncrypted
    }
}
