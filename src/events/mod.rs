//! Event types.

use std::fmt::{Display, Formatter, Error as FmtError};

pub mod call;
pub mod presence;
pub mod receipt;
pub mod room;
pub mod tag;
pub mod typing;

use self::room::avatar::AvatarEventContent;
use self::room::canonical_alias::CanonicalAliasEventContent;
use self::room::join_rules::JoinRulesEventContent;
use self::room::name::NameEventContent;

/// The type of an event.
#[derive(Debug, Deserialize, Serialize)]
pub enum EventType {
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
    RoomName,
    RoomPowerLevels,
    RoomRedaction,
    RoomThirdPartyInvite,
    RoomTopic,
    Tag,
    Typing,
}

/// A stripped-down version of a state event that is included along with some other events.
#[derive(Debug, Deserialize, Serialize)]
pub enum StrippedState {
    RoomAvatar(StrippedRoomAvatar),
    RoomCanonicalAlias(StrippedRoomCanonicalAlias),
    RoomJoinRules(StrippedRoomJoinRules),
    RoomName(StrippedRoomName),
}

/// The general form of a `StrippedState`.
#[derive(Debug, Deserialize, Serialize)]
pub struct StrippedStateContent<T> {
    pub content: T,
    pub event_type: EventType,
    pub state_key: String,
}

pub type StrippedRoomAvatar = StrippedStateContent<AvatarEventContent>;
pub type StrippedRoomCanonicalAlias = StrippedStateContent<CanonicalAliasEventContent>;
pub type StrippedRoomJoinRules = StrippedStateContent<JoinRulesEventContent>;
pub type StrippedRoomName = StrippedStateContent<NameEventContent>;

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let event_type_str = match *self {
            EventType::CallAnswer => "m.call.answer",
            EventType::CallCandidates => "m.call.candidates",
            EventType::CallHangup => "m.call.hangup",
            EventType::CallInvite => "m.call.invite",
            EventType::Presence => "m.presence",
            EventType::Receipt => "m.receipt",
            EventType::RoomAliases => "m.room.aliases",
            EventType::RoomAvatar => "m.room.avatar",
            EventType::RoomCanonicalAlias => "m.room.canonical_alias",
            EventType::RoomCreate => "m.room.create",
            EventType::RoomGuestAccess => "m.room.guest_access",
            EventType::RoomHistoryVisibility => "m.room.history_visibility",
            EventType::RoomJoinRules => "m.room.join_rules",
            EventType::RoomMember => "m.room.member",
            EventType::RoomMessage => "m.room.message",
            EventType::RoomName => "m.room.name",
            EventType::RoomPowerLevels => "m.room.power_levels",
            EventType::RoomRedaction => "m.room.redaction",
            EventType::RoomThirdPartyInvite => "m.room.third_party_invite",
            EventType::RoomTopic => "m.room.topic",
            EventType::Tag => "m.tag",
            EventType::Typing => "m.typing",
        };

        write!(f, "{}", event_type_str)
    }
}
