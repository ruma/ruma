//! Crate ruma_events contains serializable types for the events in the [Matrix](https://matrix.org)
//! specification that can be shared by client and server code.

pub mod call;
pub mod core;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod typing;

/// The type of an event.
pub enum EventTypes {
    CallAnswer,
    CallCandidates,
    CallHangup,
    CallInvite,
    Presence,
    Receipt,
    RoomAliases,
    RoomAvatar,
    RoomCanonicalAlias,
    RoomCreate,
    RoomGuestAccess,
    RoomHistoryVisibility,
    RoomJoinRules,
    RoomMember,
    RoomMessage,
    RoomMessageFeedback,
    RoomName,
    RoomPowerLevels,
    RoomRedaction,
    RoomThirdPartyInvite,
    RoomTopic,
    Typing,
}
