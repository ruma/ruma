//! "Stripped-down" versions of the core state events.
//!
//! Each "stripped" event includes only the `content`, `type`, and `state_key` fields of its full
//! version. These stripped types are useful for APIs where the user is providing the content of a
//! state event to be created, when the other fields can be inferred from a larger context, or where
//! the other fields are otherwise inapplicable.

use std::convert::TryFrom;

use ruma_identifiers::UserId;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::from_value;

use crate::{
    room::{
        aliases::AliasesEventContent, avatar::AvatarEventContent,
        canonical_alias::CanonicalAliasEventContent, create::CreateEventContent,
        guest_access::GuestAccessEventContent, history_visibility::HistoryVisibilityEventContent,
        join_rules::JoinRulesEventContent, member::MemberEventContent, name::NameEventContent,
        power_levels::PowerLevelsEventContent, third_party_invite::ThirdPartyInviteEventContent,
        topic::TopicEventContent,
    },
    EventResult, EventType, InnerInvalidEvent, InvalidEvent,
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

impl<'de> Deserialize<'de> for EventResult<StrippedState> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => {
                return Ok(EventResult::Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: "missing field `type`".to_string(),
                })))
            }
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => {
                return Ok(EventResult::Err(InvalidEvent(InnerInvalidEvent::Validation {
                    json: value,
                    message: error.to_string(),
                })))
            }
        };

        let content = match value.get("content") {
            Some(content_value) => content_value,
            None => {
                return Ok(EventResult::validation_error("missing field `content`".to_string(), value))
            }
        };

        let stripped_state = match event_type {
            // TODO: On the next stream, start with doing the other variants in this match.
            EventType::RoomAliases => {
                let content_result = match from_value::<EventResult<AliasesEventContent>>(content.clone()) {
                    Ok(content_result) => content_result,
                    Err(error) => return Err(D::Error::custom(error)),
                };

                let content = match content_result {
                    EventResult::Ok(content) => content,
                    EventResult::Err(error) => return Ok(EventResult::Err(error)),
                };

                StrippedState::RoomAliases(StrippedStateContent {
                    content,
                    event_type,
                    state_key: match value.get("state_key") {
                        Some(state_key_value) => match state_key_value.as_str() {
                            Some(state_key) => state_key.to_string(),
                            None => {
                                return Ok(EventResult::validation_error("field `state_key` must be a string".to_string(), value));
                            }
                        },
                        None => {
                            return Ok(EventResult::validation_error("missing field `state_key`".to_string(), value));
                        }
                    },
                    sender: match value.get("sender") {
                        Some(sender_value) => match sender_value.as_str() {
                            Some(sender_str) => match UserId::try_from(sender_str) {
                                Ok(sender) => sender,
                                Err(error) => {
                                    return Ok(EventResult::validation_error(error.to_string(), value));
                                }
                            },
                            None => {
                                return Ok(EventResult::validation_error("field `sender` must be a string".to_string(), value));
                            }
                        },
                        None => {
                            return Ok(EventResult::validation_error("missing field `sender`".to_string(), value));
                        }
                    },
                })
            }
            // EventType::RoomAvatar => match from_value::<StrippedRoomAvatar>(value) {
            //     Ok(stripped_state) => StrippedState::RoomAvatar(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomCanonicalAlias => match from_value::<StrippedRoomCanonicalAlias>(value) {
            //     Ok(stripped_state) => StrippedState::RoomCanonicalAlias(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomCreate => match from_value::<StrippedRoomCreate>(value) {
            //     Ok(stripped_state) => StrippedState::RoomCreate(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomGuestAccess => match from_value::<StrippedRoomGuestAccess>(value) {
            //     Ok(stripped_state) => StrippedState::RoomGuestAccess(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomHistoryVisibility => match from_value::<StrippedRoomHistoryVisibility>(value) {
            //     Ok(stripped_state) => StrippedState::RoomHistoryVisibility(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomJoinRules => match from_value::<StrippedRoomJoinRules>(value) {
            //     Ok(stripped_state) => StrippedState::RoomJoinRules(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomMember => match from_value::<StrippedRoomMember>(value) {
            //     Ok(stripped_state) => StrippedState::RoomMember(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomName => match from_value::<StrippedRoomName>(value) {
            //     Ok(stripped_state) => StrippedState::RoomName(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomPowerLevels => match from_value::<StrippedRoomPowerLevels>(value) {
            //     Ok(stripped_state) => StrippedState::RoomPowerLevels(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomThirdPartyInvite => match from_value::<StrippedRoomThirdPartyInvite>(value) {
            //     Ok(stripped_state) => StrippedState::RoomThirdPartyInvite(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            // EventType::RoomTopic => match from_value::<StrippedRoomTopic>(value) {
            //     Ok(stripped_state) => StrippedState::RoomTopic(stripped_state),
            //     Err(error) => {
            //         return Ok(EventResult::Ok(InvalidEvent(InnerInvalidEvent::Validation {
            //             json: value,
            //             message: error.to_string(),
            //         })))
            //     }
            // },
            _ => return Ok(EventResult::Err(InvalidEvent(InnerInvalidEvent::Validation {
                json: value,
                message: "not a state event".to_string(),
            }))),
        };

        Ok(EventResult::Ok(stripped_state))
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

// impl<'de, C> Deserialize<'de> for EventResult<StrippedStateContent<C>>
// where
//     EventResult<C>: Deserialize<'de>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let value = serde_json::Value::deserialize(deserializer)?;

//         let event_type_value = match value.get("type") {
//             Some(value) => value.clone(),
//             None => {
//                 return Ok(EventResult::validation_error("missing field `type`".to_string(), value))
//             }
//         };

//         let event_type = match from_value::<EventType>(event_type_value.clone()) {
//             Ok(event_type) => event_type,
//             Err(error) => {
//                 return Ok(EventResult::validation_error(error.to_string(), value))
//             }
//         };

//         let content = match value.get("content") {
//             Some(content_value) => match content_value.as_object() {
//                 Some(content) => content,
//                 None => {
//                     return Ok(EventResult::validation_error("field `content` must be an object".to_string(), value))
//                 }
//             },
//             None => {
//                 return Ok(EventResult::validation_error("missing field `content`".to_string(), value))
//             }
//         };

//         match event_type {
//             EventType::RoomAliases => stripped_state_content::<AliasesEventContent>(event_type, value),
//             EventType::RoomAvatar => stripped_state_content(event_type, value),
//             EventType::RoomCanonicalAlias => {
//                 stripped_state_content(event_type, value)
//             }
//             EventType::RoomCreate => stripped_state_content(event_type, value),
//             EventType::RoomGuestAccess => stripped_state_content(event_type, value),
//             EventType::RoomHistoryVisibility => {
//                 stripped_state_content(event_type, value)
//             }
//             EventType::RoomJoinRules => stripped_state_content(event_type, value),
//             EventType::RoomMember => stripped_state_content(event_type, value),
//             EventType::RoomName => stripped_state_content(event_type, value),
//             EventType::RoomPowerLevels => stripped_state_content(event_type, value),
//             EventType::RoomThirdPartyInvite => {
//                 stripped_state_content(event_type, value)
//             }
//             EventType::RoomTopic => stripped_state_content(event_type, value),
//             _ => Ok(EventResult::validation_error("not a state event".to_string(), value)),
//         }
//     }
// }

// /// Reduces the boilerplate in the match arms of `impl Deserialize for StrippedState`.
// #[inline]
// fn create_stripped_state(
//     event_type: EventType,
//     value: Value,
// ) -> Result<EventResult<StrippedState>, serde_json::Error>
// where
//     for<'de> EventResult<C>: Deserialize<'de>,
// {
//     let event_result = from_value::<EventResult<C>>(value)?;

//     Ok(EventResult::Ok(StrippedStateContent {
//         content: event_result.into_result().unwrap(),
//         event_type,
//         state_key: match value.get("state_key") {
//             Some(state_key_value) => match state_key_value.as_str() {
//                 Some(state_key) => state_key.to_string(),
//                 None => {
//                     return Ok(EventResult::validation_error("field `state_key` must be a string".to_string(), value));
//                 }
//             },
//             None => {
//                 return Ok(EventResult::validation_error("missing field `state_key`".to_string(), value));
//             }
//         },
//         sender: match value.get("sender") {
//             Some(sender_value) => match sender_value.as_str() {
//                 Some(sender_str) => match UserId::try_from(sender_str) {
//                     Ok(sender) => sender,
//                     Err(error) => {
//                         return Ok(EventResult::validation_error(error.to_string(), value));
//                     }
//                 },
//                 None => {
//                     return Ok(EventResult::validation_error("field `sender` must be a string".to_string(), value));
//                 }
//             },
//             None => {
//                 return Ok(EventResult::validation_error("missing field `sender`".to_string(), value));
//             }
//         },
//     }))
// }

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use js_int::UInt;
    use ruma_identifiers::UserId;
    use serde_json::to_string;

    use super::{StrippedRoomName, StrippedRoomTopic, StrippedState};
    use crate::{
        room::{join_rules::JoinRule, topic::TopicEventContent},
        EventType,
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

        match name_event.parse().unwrap() {
            StrippedState::RoomName(event) => {
                assert_eq!(event.content.name, Some("Ruma".to_string()));
                assert_eq!(event.event_type, EventType::RoomName);
                assert_eq!(event.state_key, "");
                assert_eq!(event.sender.to_string(), "@example:localhost");
            }
            _ => unreachable!(),
        };

        // Ensure `StrippedStateContent` can be parsed, not just `StrippedState`.
        assert!(name_event.parse::<StrippedRoomName>().is_ok());

        match join_rules_event.parse().unwrap() {
            StrippedState::RoomJoinRules(event) => {
                assert_eq!(event.content.join_rule, JoinRule::Public);
                assert_eq!(event.event_type, EventType::RoomJoinRules);
                assert_eq!(event.state_key, "");
                assert_eq!(event.sender.to_string(), "@example:localhost");
            }
            _ => unreachable!(),
        };

        match avatar_event.parse().unwrap() {
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
