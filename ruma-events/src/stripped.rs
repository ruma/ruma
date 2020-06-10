//! "Stripped-down" versions of the core state events.
//!
//! Each "stripped" event includes only the `content`, `type`, and `state_key` fields of its full
//! version. These stripped types are useful for APIs where the user is providing the content of a
//! state event to be created, when the other fields can be inferred from a larger context, or where
//! the other fields are otherwise inapplicable.

use ruma_identifiers::UserId;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    room::{
        aliases::AliasesEventContent, avatar::AvatarEventContent,
        canonical_alias::CanonicalAliasEventContent, create::CreateEventContent,
        guest_access::GuestAccessEventContent, history_visibility::HistoryVisibilityEventContent,
        join_rules::JoinRulesEventContent, member::MemberEventContent, name::NameEventContent,
        power_levels::PowerLevelsEventContent, third_party_invite::ThirdPartyInviteEventContent,
        topic::TopicEventContent,
    },
    util::get_field,
    EventType, TryFromRaw,
};

/// A stripped-down version of a state event that is included along with some other events.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AnyStrippedStateEvent {
    /// A stripped-down version of the *m.room.aliases* event.
    RoomAliases(StrippedRoomAliases),

    /// A stripped-down version of the *m.room.avatar* event.
    RoomAvatar(StrippedRoomAvatar),

    /// A stripped-down version of the *m.room.canonical_alias* event.
    RoomCanonicalAlias(StrippedRoomCanonicalAlias),

    /// A striped-down version of the *m.room.create* event.
    RoomCreate(StrippedRoomCreate),

    /// A stripped-down version of the *m.room.guest_access* event.
    RoomGuestAccess(StrippedRoomGuestAccess),

    /// A stripped-down version of the *m.room.history_visibility* event.
    RoomHistoryVisibility(StrippedRoomHistoryVisibility),

    /// A stripped-down version of the *m.room.join_rules* event.
    RoomJoinRules(StrippedRoomJoinRules),

    /// A stripped-down version of the *m.room.member* event.
    RoomMember(StrippedRoomMember),

    /// A stripped-down version of the *m.room.name* event.
    RoomName(StrippedRoomName),

    /// A stripped-down version of the *m.room.power_levels* event.
    RoomPowerLevels(StrippedRoomPowerLevels),

    /// A stripped-down version of the *m.room.third_party_invite* event.
    RoomThirdPartyInvite(StrippedRoomThirdPartyInvite),

    /// A stripped-down version of the *m.room.topic* event.
    RoomTopic(StrippedRoomTopic),
}

/// A "stripped-down" version of a core state event.
#[derive(Clone, Debug, Serialize)]
pub struct StrippedStateEvent<C> {
    /// Data specific to the event type.
    pub content: C,

    // FIXME(jplatte): It's unclear to me why this is stored
    /// The type of the event.
    #[serde(rename = "type")]
    pub event_type: EventType,

    /// A key that determines which piece of room state the event represents.
    pub state_key: String,

    /// The unique identifier for the user who sent this event.
    pub sender: UserId,
}

/// A stripped-down version of the *m.room.aliases* event.
pub type StrippedRoomAliases = StrippedStateEvent<AliasesEventContent>;

/// A stripped-down version of the *m.room.avatar* event.
pub type StrippedRoomAvatar = StrippedStateEvent<AvatarEventContent>;

/// A stripped-down version of the *m.room.canonical_alias* event.
pub type StrippedRoomCanonicalAlias = StrippedStateEvent<CanonicalAliasEventContent>;

/// A stripped-down version of the *m.room.create* event.
pub type StrippedRoomCreate = StrippedStateEvent<CreateEventContent>;

/// A stripped-down version of the *m.room.guest_access* event.
pub type StrippedRoomGuestAccess = StrippedStateEvent<GuestAccessEventContent>;

/// A stripped-down version of the *m.room.history_visibility* event.
pub type StrippedRoomHistoryVisibility = StrippedStateEvent<HistoryVisibilityEventContent>;

/// A stripped-down version of the *m.room.join_rules* event.
pub type StrippedRoomJoinRules = StrippedStateEvent<JoinRulesEventContent>;

/// A stripped-down version of the *m.room.member* event.
pub type StrippedRoomMember = StrippedStateEvent<MemberEventContent>;

/// A stripped-down version of the *m.room.name* event.
pub type StrippedRoomName = StrippedStateEvent<NameEventContent>;

/// A stripped-down version of the *m.room.power_levels* event.
pub type StrippedRoomPowerLevels = StrippedStateEvent<PowerLevelsEventContent>;

/// A stripped-down version of the *m.room.third_party_invite* event.
pub type StrippedRoomThirdPartyInvite = StrippedStateEvent<ThirdPartyInviteEventContent>;

/// A stripped-down version of the *m.room.topic* event.
pub type StrippedRoomTopic = StrippedStateEvent<TopicEventContent>;

impl TryFromRaw for AnyStrippedStateEvent {
    type Raw = raw::StrippedState;
    type Err = String;

    fn try_from_raw(raw: raw::StrippedState) -> Result<Self, Self::Err> {
        use crate::util::try_convert_variant as conv;
        use raw::StrippedState::*;

        match raw {
            RoomAliases(c) => conv(AnyStrippedStateEvent::RoomAliases, c),
            RoomAvatar(c) => conv(AnyStrippedStateEvent::RoomAvatar, c),
            RoomCanonicalAlias(c) => conv(AnyStrippedStateEvent::RoomCanonicalAlias, c),
            RoomCreate(c) => conv(AnyStrippedStateEvent::RoomCreate, c),
            RoomGuestAccess(c) => conv(AnyStrippedStateEvent::RoomGuestAccess, c),
            RoomHistoryVisibility(c) => conv(AnyStrippedStateEvent::RoomHistoryVisibility, c),
            RoomJoinRules(c) => conv(AnyStrippedStateEvent::RoomJoinRules, c),
            RoomMember(c) => conv(AnyStrippedStateEvent::RoomMember, c),
            RoomName(c) => conv(AnyStrippedStateEvent::RoomName, c),
            RoomPowerLevels(c) => conv(AnyStrippedStateEvent::RoomPowerLevels, c),
            RoomThirdPartyInvite(c) => conv(AnyStrippedStateEvent::RoomThirdPartyInvite, c),
            RoomTopic(c) => conv(AnyStrippedStateEvent::RoomTopic, c),
        }
    }
}

impl<C> TryFromRaw for StrippedStateEvent<C>
where
    C: TryFromRaw,
{
    type Raw = StrippedStateEvent<C::Raw>;
    type Err = C::Err;

    fn try_from_raw(raw: StrippedStateEvent<C::Raw>) -> Result<Self, Self::Err> {
        Ok(Self {
            content: C::try_from_raw(raw.content)?,
            event_type: raw.event_type,
            state_key: raw.state_key,
            sender: raw.sender,
        })
    }
}

impl<'de, C> Deserialize<'de> for StrippedStateEvent<C>
where
    C: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Optimize
        let value = JsonValue::deserialize(deserializer)?;

        Ok(Self {
            content: get_field(&value, "content")?,
            event_type: get_field(&value, "type")?,
            state_key: get_field(&value, "state_key")?,
            sender: get_field(&value, "sender")?,
        })
    }
}

mod raw {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value as JsonValue;

    use super::StrippedStateEvent;
    use crate::{
        room::{
            aliases::raw::AliasesEventContent, avatar::raw::AvatarEventContent,
            canonical_alias::raw::CanonicalAliasEventContent, create::raw::CreateEventContent,
            guest_access::raw::GuestAccessEventContent,
            history_visibility::raw::HistoryVisibilityEventContent,
            join_rules::raw::JoinRulesEventContent, member::raw::MemberEventContent,
            name::raw::NameEventContent, power_levels::raw::PowerLevelsEventContent,
            third_party_invite::raw::ThirdPartyInviteEventContent, topic::raw::TopicEventContent,
        },
        util::get_field,
    };

    /// A stripped-down version of the *m.room.aliases* event.
    pub type StrippedRoomAliases = StrippedStateEvent<AliasesEventContent>;

    /// A stripped-down version of the *m.room.avatar* event.
    pub type StrippedRoomAvatar = StrippedStateEvent<AvatarEventContent>;

    /// A stripped-down version of the *m.room.canonical_alias* event.
    pub type StrippedRoomCanonicalAlias = StrippedStateEvent<CanonicalAliasEventContent>;

    /// A stripped-down version of the *m.room.create* event.
    pub type StrippedRoomCreate = StrippedStateEvent<CreateEventContent>;

    /// A stripped-down version of the *m.room.guest_access* event.
    pub type StrippedRoomGuestAccess = StrippedStateEvent<GuestAccessEventContent>;

    /// A stripped-down version of the *m.room.history_visibility* event.
    pub type StrippedRoomHistoryVisibility = StrippedStateEvent<HistoryVisibilityEventContent>;

    /// A stripped-down version of the *m.room.join_rules* event.
    pub type StrippedRoomJoinRules = StrippedStateEvent<JoinRulesEventContent>;

    /// A stripped-down version of the *m.room.member* event.
    pub type StrippedRoomMember = StrippedStateEvent<MemberEventContent>;

    /// A stripped-down version of the *m.room.name* event.
    pub type StrippedRoomName = StrippedStateEvent<NameEventContent>;

    /// A stripped-down version of the *m.room.power_levels* event.
    pub type StrippedRoomPowerLevels = StrippedStateEvent<PowerLevelsEventContent>;

    /// A stripped-down version of the *m.room.third_party_invite* event.
    pub type StrippedRoomThirdPartyInvite = StrippedStateEvent<ThirdPartyInviteEventContent>;

    /// A stripped-down version of the *m.room.topic* event.
    pub type StrippedRoomTopic = StrippedStateEvent<TopicEventContent>;

    /// A stripped-down version of a state event that is included along with some other events.
    #[derive(Clone, Debug)]
    #[allow(clippy::large_enum_variant)]
    pub enum StrippedState {
        /// A stripped-down version of the *m.room.aliases* event.
        RoomAliases(StrippedRoomAliases),

        /// A stripped-down version of the *m.room.avatar* event.
        RoomAvatar(StrippedRoomAvatar),

        /// A stripped-down version of the *m.room.canonical_alias* event.
        RoomCanonicalAlias(StrippedRoomCanonicalAlias),

        /// A striped-down version of the *m.room.create* event.
        RoomCreate(StrippedRoomCreate),

        /// A stripped-down version of the *m.room.guest_access* event.
        RoomGuestAccess(StrippedRoomGuestAccess),

        /// A stripped-down version of the *m.room.history_visibility* event.
        RoomHistoryVisibility(StrippedRoomHistoryVisibility),

        /// A stripped-down version of the *m.room.join_rules* event.
        RoomJoinRules(StrippedRoomJoinRules),

        /// A stripped-down version of the *m.room.member* event.
        RoomMember(StrippedRoomMember),

        /// A stripped-down version of the *m.room.name* event.
        RoomName(StrippedRoomName),

        /// A stripped-down version of the *m.room.power_levels* event.
        RoomPowerLevels(StrippedRoomPowerLevels),

        /// A stripped-down version of the *m.room.third_party_invite* event.
        RoomThirdPartyInvite(StrippedRoomThirdPartyInvite),

        /// A stripped-down version of the *m.room.topic* event.
        RoomTopic(StrippedRoomTopic),
    }

    impl<'de> Deserialize<'de> for StrippedState {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use crate::{util::try_variant_from_value as from_value, EventType::*};
            use serde::de::Error as _;

            // TODO: Optimize
            let value = JsonValue::deserialize(deserializer)?;
            let event_type = get_field(&value, "type")?;

            match event_type {
                RoomAliases => from_value(value, StrippedState::RoomAliases),
                RoomAvatar => from_value(value, StrippedState::RoomAvatar),
                RoomCanonicalAlias => from_value(value, StrippedState::RoomCanonicalAlias),
                RoomCreate => from_value(value, StrippedState::RoomCreate),
                RoomGuestAccess => from_value(value, StrippedState::RoomGuestAccess),
                RoomHistoryVisibility => from_value(value, StrippedState::RoomHistoryVisibility),
                RoomJoinRules => from_value(value, StrippedState::RoomJoinRules),
                RoomMember => from_value(value, StrippedState::RoomMember),
                RoomName => from_value(value, StrippedState::RoomName),
                RoomPowerLevels => from_value(value, StrippedState::RoomPowerLevels),
                RoomThirdPartyInvite => from_value(value, StrippedState::RoomThirdPartyInvite),
                RoomTopic => from_value(value, StrippedState::RoomTopic),
                _ => Err(D::Error::custom("not a state event")),
            }
        }
    }
}

#[cfg(test)]
mod tests {}
