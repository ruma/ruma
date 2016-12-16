//! Stripped-down versions of certain state events.

use serde::{Deserialize, Deserializer, Error as SerdeError, Serialize, Serializer};
use serde_json::{Value, from_value};

use EventType;
use room::aliases::AliasesEventContent;
use room::avatar::AvatarEventContent;
use room::canonical_alias::CanonicalAliasEventContent;
use room::create::CreateEventContent;
use room::guest_access::GuestAccessEventContent;
use room::history_visibility::HistoryVisibilityEventContent;
use room::join_rules::JoinRulesEventContent;
use room::member::MemberEventContent;
use room::name::NameEventContent;
use room::power_levels::PowerLevelsEventContent;
use room::third_party_invite::ThirdPartyInviteEventContent;
use room::topic::TopicEventContent;

/// A stripped-down version of a state event that is included along with some other events.
#[derive(Clone, Debug)]
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

impl Serialize for StrippedState {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
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

impl Deserialize for StrippedState {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        let value: Value = try!(Deserialize::deserialize(deserializer));

        let event_type_value = match value.find("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::RoomAliases => {
                let event = match from_value::<StrippedRoomAliases>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomAliases(event))
            },
            EventType::RoomAvatar => {
                let event = match from_value::<StrippedRoomAvatar>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomAvatar(event))
            },
            EventType::RoomCanonicalAlias => {
                let event = match from_value::<StrippedRoomCanonicalAlias>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomCanonicalAlias(event))
            },
            EventType::RoomCreate => {
                let event = match from_value::<StrippedRoomCreate>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomCreate(event))
            },
            EventType::RoomGuestAccess => {
                let event = match from_value::<StrippedRoomGuestAccess>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomGuestAccess(event))
            },
            EventType::RoomHistoryVisibility => {
                let event = match from_value::<StrippedRoomHistoryVisibility>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomHistoryVisibility(event))
            },
            EventType::RoomJoinRules => {
                let event = match from_value::<StrippedRoomJoinRules>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomJoinRules(event))
            },
            EventType::RoomMember => {
                let event = match from_value::<StrippedRoomMember>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomMember(event))
            },
            EventType::RoomName => {
                let event = match from_value::<StrippedRoomName>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomName(event))
            },
            EventType::RoomPowerLevels => {
                let event = match from_value::<StrippedRoomPowerLevels>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomPowerLevels(event))
            },
            EventType::RoomThirdPartyInvite => {
                let event = match from_value::<StrippedRoomThirdPartyInvite>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomThirdPartyInvite(event))
            },
            EventType::RoomTopic => {
                let event = match from_value::<StrippedRoomTopic>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StrippedState::RoomTopic(event))
            },
            _ => {
                return Err(D::Error::custom("not a state event".to_string()));
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use EventType;
    use room::join_rules::JoinRule;
    use room::topic::TopicEventContent;
    use serde_json::{from_str, to_string};
    use super::{StrippedRoomTopic, StrippedState};

    #[test]
    fn serialize_stripped_state_event() {
        let content = StrippedRoomTopic {
            content: TopicEventContent { topic: "Testing room".to_string() },
            state_key: "".to_string(),
            event_type: EventType::RoomTopic
        };

        let event = StrippedState::RoomTopic(content);

        assert_eq!(
            to_string(&event).unwrap(),
            r#"{"content":{"topic":"Testing room"},"type":"m.room.topic","state_key":""}"#
        );
    }

    #[test]
    fn deserialize_stripped_state_events() {
        let name_event = r#"{
            "type": "m.room.name",
            "state_key": "",
            "content": {"name": "Ruma"}
        }"#;

        let join_rules_event = r#"{
            "type": "m.room.join_rules",
            "state_key": "",
            "content": { "join_rule": "public" }
        }"#;

        let avatar_event = r#"{
            "type": "m.room.avatar",
            "state_key": "",
            "content": {
                "info": {
                    "height": 128,
                    "width": 128,
                    "mimetype": "image/jpeg",
                    "size": 1024
                },
                "thumbnail_info": {
                    "height": 16,
                    "width": 16,
                    "mimetype": "image/jpeg",
                    "size": 32
                },
                "thumbnail_url": "https://domain.com/image-thumbnail.jpg",
                "url": "https://domain.com/image.jpg"
            }
        }"#;

        match from_str::<StrippedState>(name_event).unwrap() {
            StrippedState::RoomName(event) => {
                assert_eq!(event.content.name, "Ruma");
                assert_eq!(event.event_type, EventType::RoomName);
                assert_eq!(event.state_key, "");
            },
            _ => {
                assert!(false);
            }
        };

        match from_str::<StrippedState>(join_rules_event).unwrap() {
            StrippedState::RoomJoinRules(event) => {
                assert_eq!(event.content.join_rule, JoinRule::Public);
                assert_eq!(event.event_type, EventType::RoomJoinRules);
                assert_eq!(event.state_key, "");
            },
            _ => {
                assert!(false);
            }
        };

        match from_str::<StrippedState>(avatar_event).unwrap() {
            StrippedState::RoomAvatar(event) => {
                assert_eq!(event.content.info.height, 128);
                assert_eq!(event.content.info.width, 128);
                assert_eq!(event.content.info.mimetype, "image/jpeg");
                assert_eq!(event.content.info.size, 1024);
                assert_eq!(event.content.thumbnail_info.size, 32);
                assert_eq!(event.content.url, "https://domain.com/image.jpg");
                assert_eq!(event.event_type, EventType::RoomAvatar);
                assert_eq!(event.state_key, "");
            },
            _ => {
                assert!(false);
            }
        };
    }
}
