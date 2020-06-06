//! An enum that represents any state event. A state event is represented by
//! a parameterized struct allowing more flexibility in whats being sent.

use std::{
    convert::TryFrom,
    time::{SystemTime, UNIX_EPOCH},
};

use js_int::UInt;
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};

use crate::{RoomEventContent, StateEventContent, TryFromRaw, UnsignedData};
use ruma_events_macros::{event_content_collection, Event};

event_content_collection! {
    /// A state event.
    name: AnyStateEventContent,
    events: [
        "m.room.aliases",
        "m.room.avatar",
        "m.room.canonical_alias",
        "m.room.create",
        "m.room.encryption",
        "m.room.guest_access",
        "m.room.history_visibility",
        "m.room.join_rules",
        "m.room.member",
        "m.room.name",
        "m.room.pinned_events",
        "m.room.power_levels",
        "m.room.server_acl",
        "m.room.third_party_invite",
        "m.room.tombstone",
        "m.room.topic",
    ]
}

/// State event.
#[derive(Clone, Debug, Event)]
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

    /// Additional key-value pairs not signed by the homeserver.
    pub unsigned: UnsignedData,
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

    use super::{AnyStateEventContent, StateEvent};
    use crate::{
        room::{
            aliases::AliasesEventContent, avatar::AvatarEventContent, ImageInfo, ThumbnailInfo,
        },
        EventJson, UnsignedData,
    };

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
            unsigned: UnsignedData::default(),
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
            unsigned: UnsignedData::default(),
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
    fn deserialize_aliases_content() {
        let json_data = json!({
            "aliases": [ "#somewhere:localhost" ]
        });

        assert_matches!(
            from_json_value::<EventJson<AnyStateEventContent>>(json_data)
                .unwrap()
                .deserialize_content("m.room.aliases")
                .unwrap(),
            AnyStateEventContent::RoomAliases(content)
            if content.aliases == vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()]
        );
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
            from_json_value::<EventJson<StateEvent<AnyStateEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent {
                content: AnyStateEventContent::RoomAliases(content),
                event_id,
                origin_server_ts,
                prev_content: Some(AnyStateEventContent::RoomAliases(prev_content)),
                room_id,
                sender,
                state_key,
                unsigned,
            } if content.aliases == vec![RoomAliasId::try_from("#somewhere:localhost").unwrap()]
                && event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && prev_content.aliases == vec![RoomAliasId::try_from("#inner:localhost").unwrap()]
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && state_key == ""
                && unsigned.is_empty()
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
            from_json_value::<EventJson<StateEvent<AnyStateEventContent>>>(json_data)
                .unwrap()
                .deserialize()
                .unwrap(),
            StateEvent {
                content: AnyStateEventContent::RoomAvatar(AvatarEventContent {
                    info: Some(info),
                    url,
                }),
                event_id,
                origin_server_ts,
                prev_content: None,
                room_id,
                sender,
                state_key,
                unsigned
            } if event_id == EventId::try_from("$h29iv0s8:example.com").unwrap()
                && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
                && room_id == RoomId::try_from("!roomid:room.com").unwrap()
                && sender == UserId::try_from("@carl:example.com").unwrap()
                && state_key == ""
                && matches!(
                    info.as_ref(),
                    ImageInfo {
                        height,
                        width,
                        mimetype: Some(mimetype),
                        size,
                        thumbnail_info: Some(thumbnail_info),
                        thumbnail_url: Some(thumbnail_url),
                        thumbnail_file: None,
                    } if *height == UInt::new(423)
                        && *width == UInt::new(1011)
                        && *mimetype == "image/png"
                        && *size == UInt::new(84242)
                        && matches!(
                            thumbnail_info.as_ref(),
                            ThumbnailInfo {
                                width: thumb_width,
                                height: thumb_height,
                                mimetype: thumb_mimetype,
                                size: thumb_size,
                            } if *thumb_width == UInt::new(800)
                                && *thumb_height == UInt::new(334)
                                && *thumb_mimetype == Some("image/png".to_string())
                                && *thumb_size == UInt::new(82595)
                                && *thumbnail_url == "mxc://matrix.org"
                        )
                )
                && url == "http://www.matrix.org"
                && unsigned.is_empty()
        );
    }
}
