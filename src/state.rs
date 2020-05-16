//! An enum that represents any state event. A state event is represented by
//! a parameterized struct allowing more flexibility in whats being sent.

use std::{
    convert::TryFrom,
    fmt,
    marker::PhantomData,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{
    de::{self, Deserialize, Deserializer, Error as _, MapAccess, Visitor},
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};
use serde_json::value::RawValue as RawJsonValue;

use crate::{
    error::{InvalidEvent, InvalidEventKind},
    room::{aliases::AliasesEventContent, avatar::AvatarEventContent},
    EventContent, RoomEventContent, StateEventContent,
};

/// A state event.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum AnyStateEventContent {
    /// m.room.aliases
    RoomAliases(AliasesEventContent),

    /// m.room.avatar
    RoomAvatar(AvatarEventContent),
    // /// m.room.canonical_alias
    // RoomCanonicalAlias(StateEvent<CanonicalAliasEventContent>),

    // /// m.room.create
    // RoomCreate(StateEvent<CreateEventContent>),

    // /// m.room.encryption
    // RoomEncryption(StateEvent<EncryptionEventContent>),

    // /// m.room.guest_access
    // RoomGuestAccess(StateEvent<GuestAccessEventContent>),

    // /// m.room.history_visibility
    // RoomHistoryVisibility(StateEvent<HistoryVisibilityEventContent>),

    // /// m.room.join_rules
    // RoomJoinRules(StateEvent<JoinRulesEventContent>),

    // /// m.room.member
    // RoomMember(StateEvent<MemberEventContent>),

    // /// m.room.name
    // RoomName(StateEvent<NameEventContent>),

    // /// m.room.pinned_events
    // RoomPinnedEvents(StateEvent<PinnedEventsEventContent>),

    // /// m.room.power_levels
    // RoomPowerLevels(StateEvent<PinnedEventsEventContent>),

    // /// m.room.server_acl
    // RoomServerAcl(StateEvent<ServerAclEventContent>),

    // /// m.room.third_party_invite
    // RoomThirdPartyInvite(StateEvent<ThirdPartyInviteEventContent>),

    // /// m.room.tombstone
    // RoomTombstone(StateEvent<TombstoneEventContent>),

    // /// m.room.topic
    // RoomTopic(StateEvent<TopicEventContent>),

    // /// Any state event that is not part of the specification.
    // CustomState(StateEvent<CustomEventContent>),
}

/// To-device event.
#[derive(Clone, Debug)]
pub struct StateEvent<C: StateEventContent> {
    /// Data specific to the event type.
    pub content: C,

    /// The globally unique event identifier for the user who sent the event.
    pub event_id: EventId,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: UserId,

    /// Timestamp in milliseconds on originating homeserver when this event was sent.
    pub origin_server_ts: SystemTime,

    /// The ID of the room associated with this event.
    pub room_id: RoomId,

    /// A unique key which defines the overwriting semantics for this piece of room state.
    ///
    /// This is often an empty string, but some events send a `UserId` to show
    /// which user the event affects.
    pub state_key: String,

    /// Optional previous content for this event.
    pub prev_content: Option<C>,
}

impl EventContent for AnyStateEventContent {
    fn event_type(&self) -> &str {
        match self {
            AnyStateEventContent::RoomAliases(content) => content.event_type(),
            AnyStateEventContent::RoomAvatar(content) => content.event_type(),
        }
    }

    fn from_parts(event_type: &str, content: Box<RawJsonValue>) -> Result<Self, InvalidEvent> {
        fn deserialize_variant<T: StateEventContent>(
            ev_type: &str,
            input: Box<RawJsonValue>,
            variant: fn(T) -> AnyStateEventContent,
        ) -> Result<AnyStateEventContent, InvalidEvent> {
            let content = T::from_parts(ev_type, input)?;
            Ok(variant(content))
        }

        match event_type {
            "m.room.avatar" => deserialize_variant::<AvatarEventContent>(
                event_type,
                content,
                AnyStateEventContent::RoomAvatar,
            ),
            "m.room.aliases" => deserialize_variant::<AliasesEventContent>(
                event_type,
                content,
                AnyStateEventContent::RoomAliases,
            ),
            ev => Err(InvalidEvent {
                kind: InvalidEventKind::Deserialization,
                message: format!("event not supported {}", ev),
            }),
        }
    }
}

impl RoomEventContent for AnyStateEventContent {}

impl StateEventContent for AnyStateEventContent {}

impl<C: StateEventContent> Serialize for StateEvent<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let event_type = self.content.event_type();

        let time_since_epoch = self.origin_server_ts.duration_since(UNIX_EPOCH).unwrap();
        let timestamp = match UInt::try_from(time_since_epoch.as_millis()) {
            Ok(uint) => uint,
            Err(err) => return Err(S::Error::custom(err)),
        };

        let mut state = serializer.serialize_struct("StateEvent", 7)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("event_id", &self.event_id)?;
        state.serialize_field("sender", &self.sender)?;
        state.serialize_field("origin_server_ts", &timestamp)?;
        state.serialize_field("room_id", &self.room_id)?;
        state.serialize_field("state_key", &self.state_key)?;
        if let Some(content) = self.prev_content.as_ref() {
            state.serialize_field("prev_content", content)?;
        }
        state.serialize_field("type", event_type)?;
        state.end()
    }
}

impl<'de, C: StateEventContent> Deserialize<'de> for StateEvent<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StateEventVisitor(std::marker::PhantomData))
    }
}

#[derive(serde::Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
    Type,
    Content,
    EventId,
    Sender,
    OriginServerTs,
    RoomId,
    StateKey,
    PrevContent,
}

/// Visits the fields of a StateEvent<C> to handle deserialization of
/// the `content` and `prev_content` fields.
struct StateEventVisitor<C: StateEventContent>(PhantomData<C>);

impl<'de, C: StateEventContent> Visitor<'de> for StateEventVisitor<C> {
    type Value = StateEvent<C>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "struct implementing StateEventContent")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut content: Option<Box<RawJsonValue>> = None;
        let mut event_type: Option<String> = None;
        let mut event_id: Option<EventId> = None;
        let mut sender: Option<UserId> = None;
        let mut origin_server_ts: Option<UInt> = None;
        let mut room_id: Option<RoomId> = None;
        let mut state_key: Option<String> = None;
        let mut prev_content: Option<Box<RawJsonValue>> = None;

        while let Some(key) = map.next_key()? {
            match key {
                Field::Content => {
                    if content.is_some() {
                        return Err(de::Error::duplicate_field("content"));
                    }
                    content = Some(map.next_value()?);
                }
                Field::EventId => {
                    if event_id.is_some() {
                        return Err(de::Error::duplicate_field("event_id"));
                    }
                    event_id = Some(map.next_value()?);
                }
                Field::Sender => {
                    if sender.is_some() {
                        return Err(de::Error::duplicate_field("sender"));
                    }
                    sender = Some(map.next_value()?);
                }
                Field::OriginServerTs => {
                    if origin_server_ts.is_some() {
                        return Err(de::Error::duplicate_field("origin_server_ts"));
                    }
                    origin_server_ts = Some(map.next_value()?);
                }
                Field::RoomId => {
                    if room_id.is_some() {
                        return Err(de::Error::duplicate_field("room_id"));
                    }
                    room_id = Some(map.next_value()?);
                }
                Field::StateKey => {
                    if state_key.is_some() {
                        return Err(de::Error::duplicate_field("state_key"));
                    }
                    state_key = Some(map.next_value()?);
                }
                Field::PrevContent => {
                    if prev_content.is_some() {
                        return Err(de::Error::duplicate_field("prev_content"));
                    }
                    prev_content = Some(map.next_value()?);
                }
                Field::Type => {
                    if event_type.is_some() {
                        return Err(de::Error::duplicate_field("type"));
                    }
                    event_type = Some(map.next_value()?);
                }
            }
        }

        let event_type = event_type.ok_or_else(|| de::Error::missing_field("type"))?;

        let raw = content.ok_or_else(|| de::Error::missing_field("content"))?;
        let content = C::from_parts(&event_type, raw).map_err(A::Error::custom)?;

        let event_id = event_id.ok_or_else(|| de::Error::missing_field("event_id"))?;
        let sender = sender.ok_or_else(|| de::Error::missing_field("sender"))?;

        let origin_server_ts = origin_server_ts
            .map(|time| UNIX_EPOCH + Duration::from_millis(time.into()))
            .ok_or_else(|| de::Error::missing_field("origin_server_ts"))?;

        let room_id = room_id.ok_or_else(|| de::Error::missing_field("room_id"))?;
        let state_key = state_key.ok_or_else(|| de::Error::missing_field("state_key"))?;

        let prev_content = if let Some(raw) = prev_content {
            Some(C::from_parts(&event_type, raw).map_err(A::Error::custom)?)
        } else {
            None
        };

        Ok(StateEvent {
            content,
            event_id,
            sender,
            origin_server_ts,
            room_id,
            state_key,
            prev_content,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use js_int::UInt;
    use matches::assert_matches;
    use ruma_identifiers::{EventId, RoomAliasId, RoomId, UserId};
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{AliasesEventContent, AnyStateEventContent, AvatarEventContent, StateEvent};
    use crate::room::{ImageInfo, ThumbnailInfo};

    #[test]
    fn serialize_aliases_with_prev_content() {
        let aliases_event = StateEvent {
            content: AnyStateEventContent::RoomAliases(AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()],
            }),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: Some(AnyStateEventContent::RoomAliases(AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()],
            })),
            room_id: RoomId::try_from("!roomid:room.com").unwrap(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
        };

        let actual = to_json_value(&aliases_event).unwrap();
        let expected = json!({
            "content": {
                "aliases": [ "#somewhere:localhost" ]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": [ "#somewhere:localhost" ]
            },
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.aliases",
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn serialize_aliases_without_prev_content() {
        let aliases_event = StateEvent {
            content: AnyStateEventContent::RoomAliases(AliasesEventContent {
                aliases: vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()],
            }),
            event_id: EventId::try_from("$h29iv0s8:example.com").unwrap(),
            origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
            prev_content: None,
            room_id: RoomId::try_from("!roomid:room.com").unwrap(),
            sender: UserId::try_from("@carl:example.com").unwrap(),
            state_key: "".to_string(),
        };

        let actual = to_json_value(&aliases_event).unwrap();
        let expected = json!({
            "content": {
                "aliases": [ "#somewhere:localhost" ]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.aliases",
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_aliases_with_prev_content() {
        let json_data = json!({
            "content": {
                "aliases": [ "#somewhere:localhost" ]
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "prev_content": {
                "aliases": [ "#inner:localhost" ]
            },
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.aliases"
        });

        assert_matches!(
            from_json_value::<StateEvent<AnyStateEventContent>>(json_data).unwrap(),
            StateEvent {
                content: AnyStateEventContent::RoomAliases(content),
                event_id,
                origin_server_ts,
                prev_content: Some(AnyStateEventContent::RoomAliases(prev_content)),
                room_id,
                sender,
                state_key,
            } if content.aliases == vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()]
                && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && prev_content.aliases == vec![RoomAliasId::try_from("#inner:localhost").unwrap()]
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && state_key == ""
        );
    }

    #[test]
    fn deserialize_avatar_without_prev_content() {
        let json_data = json!({
            "content": {
                "info": {
                    "h": 423,
                    "mimetype": "image/png",
                    "size": 84242,
                    "thumbnail_info": {
                      "h": 334,
                      "mimetype": "image/png",
                      "size": 82595,
                      "w": 800
                    },
                    "thumbnail_url": "mxc://matrix.org",
                    "w": 1011
                  },
                "url": "http://www.matrix.org"
            },
            "event_id": "$h29iv0s8:example.com",
            "origin_server_ts": 1,
            "room_id": "!roomid:room.com",
            "sender": "@carl:example.com",
            "state_key": "",
            "type": "m.room.avatar"
        });

        assert_matches!(
            from_json_value::<StateEvent<AnyStateEventContent>>(json_data).unwrap(),
            StateEvent {
                content: AnyStateEventContent::RoomAvatar(AvatarEventContent {
                    info: Some(ImageInfo {
                        height,
                        width,
                        mimetype: Some(mimetype),
                        size,
                        thumbnail_info: Some(ThumbnailInfo {
                            width: thumb_width,
                            height: thumb_height,
                            mimetype: thumb_mimetype,
                            size: thumb_size,
                        }),
                        thumbnail_url: Some(thumbnail_url),
                        thumbnail_file: None,
                    }),
                    url,
                }),
                event_id,
                origin_server_ts,
                prev_content: None,
                room_id,
                sender,
                state_key,
            } if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && state_key == ""
                && height == UInt::new(423)
                && width == UInt::new(1011)
                && mimetype == "image/png"
                && size == UInt::new(84242)
                && thumb_width == UInt::new(800)
                && thumb_height == UInt::new(334)
                && thumb_mimetype == Some("image/png".to_string())
                && thumb_size == UInt::new(82595)
                && thumbnail_url == "mxc://matrix.org"
                && url == "http://www.matrix.org"
        );
    }
}
