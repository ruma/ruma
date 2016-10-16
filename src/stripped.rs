//! Stripped-down versions of certain state events.

use serde::{Deserialize, Serialize};

use EventType;
use room::avatar::AvatarEventContent;
use room::canonical_alias::CanonicalAliasEventContent;
use room::join_rules::JoinRulesEventContent;
use room::name::NameEventContent;

/// A stripped-down version of a state event that is included along with some other events.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StrippedState {
    /// A stripped-down version of the *m.room.avatar* event.
    RoomAvatar(StrippedRoomAvatar),

    /// A stripped-down version of the *m.room.canonical_alias* event.
    RoomCanonicalAlias(StrippedRoomCanonicalAlias),

    /// A stripped-down version of the *m.room.join_rules* event.
    RoomJoinRules(StrippedRoomJoinRules),

    /// A stripped-down version of the *m.room.name* event.
    RoomName(StrippedRoomName),
}

/// The general form of a `StrippedState`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StrippedStateContent<C> where C: Deserialize + Serialize {
    /// Data specific to the event type.
    pub content: C,
    /// The type of the event.
    #[serde(rename="type")]
    pub event_type: EventType,
    /// A key that determines which piece of room state the event represents.
    pub state_key: String,
}

/// A stripped-down version of the *m.room.avatar* event.
pub type StrippedRoomAvatar = StrippedStateContent<AvatarEventContent>;

/// A stripped-down version of the *m.room.canonical_alias* event.
pub type StrippedRoomCanonicalAlias = StrippedStateContent<CanonicalAliasEventContent>;

/// A stripped-down version of the *m.room.join_rules* event.
pub type StrippedRoomJoinRules = StrippedStateContent<JoinRulesEventContent>;

/// A stripped-down version of the *m.room.name* event.
pub type StrippedRoomName = StrippedStateContent<NameEventContent>;
