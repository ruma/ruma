//! "Stripped-down" versions of the core state events.
//!
//! Each "stripped" event includes only the `content`, `type`, and `state_key` fields of its full
//! version. These stripped types are useful for APIs where the user is providing the content of a
//! state event to be created, when the other fields can be inferred from a larger context, or where
//! the other fields are otherwise inapplicable.

use ruma_identifiers::UserId;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

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

/// A "stripped-down" version of a core state event.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StrippedStateContent<C> {
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
pub type StrippedRoomAliases = StrippedStateContent<AliasesEventContent>;

/// A stripped-down version of the *m.room.avatar* event.
pub type StrippedRoomAvatar = StrippedStateContent<AvatarEventContent>;

/// A stripped-down version of the *m.room.canonical_alias* event.
pub type StrippedRoomCanonicalAlias = StrippedStateContent<CanonicalAliasEventContent>;

/// A stripped-down version of the *m.room.create* event.
pub type StrippedRoomCreate = StrippedStateContent<CreateEventContent>;

/// A stripped-down version of the *m.room.guest_access* event.
pub type StrippedRoomGuestAccess = StrippedStateContent<GuestAccessEventContent>;

/// A stripped-down version of the *m.room.history_visibility* event.
pub type StrippedRoomHistoryVisibility = StrippedStateContent<HistoryVisibilityEventContent>;

/// A stripped-down version of the *m.room.join_rules* event.
pub type StrippedRoomJoinRules = StrippedStateContent<JoinRulesEventContent>;

/// A stripped-down version of the *m.room.member* event.
pub type StrippedRoomMember = StrippedStateContent<MemberEventContent>;

/// A stripped-down version of the *m.room.name* event.
pub type StrippedRoomName = StrippedStateContent<NameEventContent>;

/// A stripped-down version of the *m.room.power_levels* event.
pub type StrippedRoomPowerLevels = StrippedStateContent<PowerLevelsEventContent>;

/// A stripped-down version of the *m.room.third_party_invite* event.
pub type StrippedRoomThirdPartyInvite = StrippedStateContent<ThirdPartyInviteEventContent>;

/// A stripped-down version of the *m.room.topic* event.
pub type StrippedRoomTopic = StrippedStateContent<TopicEventContent>;

impl TryFromRaw for StrippedState {
    type Raw = raw::StrippedState;
    type Err = String;

    fn try_from_raw(raw: raw::StrippedState) -> Result<Self, (Self::Err, Self::Raw)> {
        use crate::util::try_convert_variant as conv;
        use raw::StrippedState::*;

        match raw {
            RoomAliases(c) => conv(RoomAliases, StrippedState::RoomAliases, c),
            RoomAvatar(c) => conv(RoomAvatar, StrippedState::RoomAvatar, c),
            RoomCanonicalAlias(c) => conv(RoomCanonicalAlias, StrippedState::RoomCanonicalAlias, c),
            RoomCreate(c) => conv(RoomCreate, StrippedState::RoomCreate, c),
            RoomGuestAccess(c) => conv(RoomGuestAccess, StrippedState::RoomGuestAccess, c),
            RoomHistoryVisibility(c) => conv(
                RoomHistoryVisibility,
                StrippedState::RoomHistoryVisibility,
                c,
            ),
            RoomJoinRules(c) => conv(RoomJoinRules, StrippedState::RoomJoinRules, c),
            RoomMember(c) => conv(RoomMember, StrippedState::RoomMember, c),
            RoomName(c) => conv(RoomName, StrippedState::RoomName, c),
            RoomPowerLevels(c) => conv(RoomPowerLevels, StrippedState::RoomPowerLevels, c),
            RoomThirdPartyInvite(c) => {
                conv(RoomThirdPartyInvite, StrippedState::RoomThirdPartyInvite, c)
            }
            RoomTopic(c) => conv(RoomTopic, StrippedState::RoomTopic, c),
        }
    }
}

impl<C> TryFromRaw for StrippedStateContent<C>
where
    C: TryFromRaw,
{
    type Raw = StrippedStateContent<C::Raw>;
    type Err = C::Err;

    fn try_from_raw(mut raw: StrippedStateContent<C::Raw>) -> Result<Self, (Self::Err, Self::Raw)> {
        Ok(Self {
            content: match C::try_from_raw(raw.content) {
                Ok(c) => c,
                Err((msg, raw_content)) => {
                    // we moved raw.content, so we need to put it back before returning raw
                    raw.content = raw_content;
                    return Err((msg, raw));
                }
            },
            event_type: raw.event_type,
            state_key: raw.state_key,
            sender: raw.sender,
        })
    }
}

impl Serialize for StrippedState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            StrippedState::RoomAliases(ref event) => event.serialize(serializer),
            StrippedState::RoomAvatar(ref event) => event.serialize(serializer),
            StrippedState::RoomCanonicalAlias(ref event) => event.serialize(serializer),
            StrippedState::RoomCreate(ref event) => event.serialize(serializer),
            StrippedState::RoomGuestAccess(ref event) => event.serialize(serializer),
            StrippedState::RoomHistoryVisibility(ref event) => event.serialize(serializer),
            StrippedState::RoomJoinRules(ref event) => event.serialize(serializer),
            StrippedState::RoomMember(ref event) => event.serialize(serializer),
            StrippedState::RoomName(ref event) => event.serialize(serializer),
            StrippedState::RoomPowerLevels(ref event) => event.serialize(serializer),
            StrippedState::RoomThirdPartyInvite(ref event) => event.serialize(serializer),
            StrippedState::RoomTopic(ref event) => event.serialize(serializer),
        }
    }
}

impl<'de, C> Deserialize<'de> for StrippedStateContent<C>
where
    C: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Optimize
        let value = Value::deserialize(deserializer)?;

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
    use serde_json::Value;

    use super::StrippedStateContent;
    use crate::room::{
        aliases::raw::AliasesEventContent, avatar::raw::AvatarEventContent,
        canonical_alias::raw::CanonicalAliasEventContent, create::raw::CreateEventContent,
        guest_access::raw::GuestAccessEventContent,
        history_visibility::raw::HistoryVisibilityEventContent,
        join_rules::raw::JoinRulesEventContent, member::raw::MemberEventContent,
        name::raw::NameEventContent, power_levels::raw::PowerLevelsEventContent,
        third_party_invite::raw::ThirdPartyInviteEventContent, topic::raw::TopicEventContent,
    };

    /// A stripped-down version of the *m.room.aliases* event.
    pub type StrippedRoomAliases = StrippedStateContent<AliasesEventContent>;

    /// A stripped-down version of the *m.room.avatar* event.
    pub type StrippedRoomAvatar = StrippedStateContent<AvatarEventContent>;

    /// A stripped-down version of the *m.room.canonical_alias* event.
    pub type StrippedRoomCanonicalAlias = StrippedStateContent<CanonicalAliasEventContent>;

    /// A stripped-down version of the *m.room.create* event.
    pub type StrippedRoomCreate = StrippedStateContent<CreateEventContent>;

    /// A stripped-down version of the *m.room.guest_access* event.
    pub type StrippedRoomGuestAccess = StrippedStateContent<GuestAccessEventContent>;

    /// A stripped-down version of the *m.room.history_visibility* event.
    pub type StrippedRoomHistoryVisibility = StrippedStateContent<HistoryVisibilityEventContent>;

    /// A stripped-down version of the *m.room.join_rules* event.
    pub type StrippedRoomJoinRules = StrippedStateContent<JoinRulesEventContent>;

    /// A stripped-down version of the *m.room.member* event.
    pub type StrippedRoomMember = StrippedStateContent<MemberEventContent>;

    /// A stripped-down version of the *m.room.name* event.
    pub type StrippedRoomName = StrippedStateContent<NameEventContent>;

    /// A stripped-down version of the *m.room.power_levels* event.
    pub type StrippedRoomPowerLevels = StrippedStateContent<PowerLevelsEventContent>;

    /// A stripped-down version of the *m.room.third_party_invite* event.
    pub type StrippedRoomThirdPartyInvite = StrippedStateContent<ThirdPartyInviteEventContent>;

    /// A stripped-down version of the *m.room.topic* event.
    pub type StrippedRoomTopic = StrippedStateContent<TopicEventContent>;

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
            use crate::{
                util::{get_field, serde_json_error_to_generic_de_error as conv_err},
                EventType::*,
            };
            use serde::de::Error as _;
            use serde_json::from_value;

            // TODO: Optimize
            let value = Value::deserialize(deserializer)?;
            let event_type = get_field(&value, "type")?;

            match event_type {
                RoomAliases => from_value(value)
                    .map(StrippedState::RoomAliases)
                    .map_err(conv_err),
                RoomAvatar => from_value(value)
                    .map(StrippedState::RoomAvatar)
                    .map_err(conv_err),
                RoomCanonicalAlias => from_value(value)
                    .map(StrippedState::RoomCanonicalAlias)
                    .map_err(conv_err),
                RoomCreate => from_value(value)
                    .map(StrippedState::RoomCreate)
                    .map_err(conv_err),
                RoomGuestAccess => from_value(value)
                    .map(StrippedState::RoomGuestAccess)
                    .map_err(conv_err),
                RoomHistoryVisibility => from_value(value)
                    .map(StrippedState::RoomHistoryVisibility)
                    .map_err(conv_err),
                RoomJoinRules => from_value(value)
                    .map(StrippedState::RoomJoinRules)
                    .map_err(conv_err),
                RoomMember => from_value(value)
                    .map(StrippedState::RoomMember)
                    .map_err(conv_err),
                RoomName => from_value(value)
                    .map(StrippedState::RoomName)
                    .map_err(conv_err),
                RoomPowerLevels => from_value(value)
                    .map(StrippedState::RoomPowerLevels)
                    .map_err(conv_err),
                RoomThirdPartyInvite => from_value(value)
                    .map(StrippedState::RoomThirdPartyInvite)
                    .map_err(conv_err),
                RoomTopic => from_value(value)
                    .map(StrippedState::RoomTopic)
                    .map_err(conv_err),
                _ => Err(D::Error::custom("not a state event")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::UserId;
    use serde_json::to_string;

    use super::{StrippedRoomName, StrippedRoomTopic, StrippedState};
    use crate::{
        room::{join_rules::JoinRule, topic::TopicEventContent},
        EventResult, EventType,
    };

    #[test]
    fn serialize_stripped_state_event() {
        let content = StrippedRoomTopic {
            content: TopicEventContent {
                topic: "Testing room".to_string(),
            },
            state_key: "".to_string(),
            event_type: EventType::RoomTopic,
            sender: UserId::try_from("@example:localhost").unwrap(),
        };

        let event = StrippedState::RoomTopic(content);

        assert_eq!(
            to_string(&event).unwrap(),
            r#"{"content":{"topic":"Testing room"},"type":"m.room.topic","state_key":"","sender":"@example:localhost"}"#
        );
    }

    #[test]
    fn deserialize_stripped_state_events() {
        let name_event = r#"{
            "type": "m.room.name",
            "state_key": "",
            "sender": "@example:localhost",
            "content": {"name": "Ruma"}
        }"#;

        let join_rules_event = r#"{
            "type": "m.room.join_rules",
            "state_key": "",
            "sender": "@example:localhost",
            "content": { "join_rule": "public" }
        }"#;

        let avatar_event = r#"{
            "type": "m.room.avatar",
            "state_key": "",
            "sender": "@example:localhost",
            "content": {
                "info": {
                    "h": 128,
                    "w": 128,
                    "mimetype": "image/jpeg",
                    "size": 1024,
                    "thumbnail_info": {
                        "h": 16,
                        "w": 16,
                        "mimetype": "image/jpeg",
                        "size": 32
                    },
                    "thumbnail_url": "https://domain.com/image-thumbnail.jpg"
                },
                "thumbnail_info": {
                    "h": 16,
                    "w": 16,
                    "mimetype": "image/jpeg",
                    "size": 32
                },
                "thumbnail_url": "https://domain.com/image-thumbnail.jpg",
                "url": "https://domain.com/image.jpg"
            }
        }"#;

        match serde_json::from_str::<EventResult<_>>(name_event)
            .unwrap()
            .into_result()
            .unwrap()
        {
            StrippedState::RoomName(event) => {
                assert_eq!(event.content.name, Some("Ruma".to_string()));
                assert_eq!(event.event_type, EventType::RoomName);
                assert_eq!(event.state_key, "");
                assert_eq!(event.sender.to_string(), "@example:localhost");
            }
            _ => unreachable!(),
        };

        // Ensure `StrippedStateContent` can be parsed, not just `StrippedState`.
        assert!(
            serde_json::from_str::<EventResult<StrippedRoomName>>(name_event)
                .unwrap()
                .into_result()
                .is_ok()
        );

        match serde_json::from_str::<EventResult<_>>(join_rules_event)
            .unwrap()
            .into_result()
            .unwrap()
        {
            StrippedState::RoomJoinRules(event) => {
                assert_eq!(event.content.join_rule, JoinRule::Public);
                assert_eq!(event.event_type, EventType::RoomJoinRules);
                assert_eq!(event.state_key, "");
                assert_eq!(event.sender.to_string(), "@example:localhost");
            }
            _ => unreachable!(),
        };

        match serde_json::from_str::<EventResult<_>>(avatar_event)
            .unwrap()
            .into_result()
            .unwrap()
        {
            StrippedState::RoomAvatar(event) => {
                let image_info = event.content.info.unwrap();

                assert_eq!(image_info.height.unwrap(), UInt::try_from(128).unwrap());
                assert_eq!(image_info.width.unwrap(), UInt::try_from(128).unwrap());
                assert_eq!(image_info.mimetype.unwrap(), "image/jpeg");
                assert_eq!(image_info.size.unwrap(), UInt::try_from(1024).unwrap());
                assert_eq!(
                    image_info.thumbnail_info.unwrap().size.unwrap(),
                    UInt::try_from(32).unwrap()
                );
                assert_eq!(event.content.url, "https://domain.com/image.jpg");
                assert_eq!(event.event_type, EventType::RoomAvatar);
                assert_eq!(event.state_key, "");
                assert_eq!(event.sender.to_string(), "@example:localhost");
            }
            _ => unreachable!(),
        };
    }
}
