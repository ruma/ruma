use std::time::{Duration, UNIX_EPOCH};

use js_int::UInt;
use matches::assert_matches;
use ruma_events::{
    room::{aliases::AliasesEventContent, avatar::AvatarEventContent, ImageInfo, ThumbnailInfo},
    AnyRoomEvent, AnyStateEvent, AnyStateEventContent, AnySyncStateEvent, RawExt, StateEvent,
    SyncStateEvent, Unsigned,
};
use ruma_identifiers::{event_id, room_alias_id, room_id, user_id};
use ruma_serde::Raw;
use serde_json::{
    from_value as from_json_value, json, to_value as to_json_value, Value as JsonValue,
};

fn aliases_event_with_prev_content() -> JsonValue {
    json!({
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
    })
}

#[test]
fn serialize_aliases_with_prev_content() {
    let aliases_event = StateEvent {
        content: AnyStateEventContent::RoomAliases(AliasesEventContent::new(vec![room_alias_id!(
            "#somewhere:localhost"
        )])),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        prev_content: Some(AnyStateEventContent::RoomAliases(AliasesEventContent::new(vec![
            room_alias_id!("#inner:localhost"),
        ]))),
        room_id: room_id!("!roomid:room.com"),
        sender: user_id!("@carl:example.com"),
        state_key: "".into(),
        unsigned: Unsigned::default(),
    };

    let actual = to_json_value(&aliases_event).unwrap();
    let expected = aliases_event_with_prev_content();

    assert_eq!(actual, expected);
}

#[test]
fn serialize_aliases_without_prev_content() {
    let aliases_event = StateEvent {
        content: AnyStateEventContent::RoomAliases(AliasesEventContent::new(vec![room_alias_id!(
            "#somewhere:localhost"
        )])),
        event_id: event_id!("$h29iv0s8:example.com"),
        origin_server_ts: UNIX_EPOCH + Duration::from_millis(1),
        prev_content: None,
        room_id: room_id!("!roomid:room.com"),
        sender: user_id!("@carl:example.com"),
        state_key: "".into(),
        unsigned: Unsigned::default(),
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
        from_json_value::<Raw<AnyStateEventContent>>(json_data)
            .unwrap()
            .deserialize_content("m.room.aliases")
            .unwrap(),
        AnyStateEventContent::RoomAliases(content)
        if content.aliases == vec![room_alias_id!("#somewhere:localhost")]
    );
}

#[test]
fn deserialize_aliases_with_prev_content() {
    let json_data = aliases_event_with_prev_content();

    assert_matches!(
        from_json_value::<Raw<StateEvent<AnyStateEventContent>>>(json_data)
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
        } if content.aliases == vec![room_alias_id!("#somewhere:localhost")]
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && prev_content.aliases == vec![room_alias_id!("#inner:localhost")]
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
            && state_key.is_empty()
            && unsigned.is_empty()
    );
}

#[test]
fn deserialize_aliases_sync_with_room_id() {
    // The same JSON can be used to create a sync event, it just ignores the `room_id` field
    let json_data = aliases_event_with_prev_content();

    assert_matches!(
        from_json_value::<SyncStateEvent<AnyStateEventContent>>(json_data)
            .unwrap(),
        SyncStateEvent {
            content: AnyStateEventContent::RoomAliases(content),
            event_id,
            origin_server_ts,
            prev_content: Some(AnyStateEventContent::RoomAliases(prev_content)),
            sender,
            state_key,
            unsigned,
        } if content.aliases == vec![room_alias_id!("#somewhere:localhost")]
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && prev_content.aliases == vec![room_alias_id!("#inner:localhost")]
            && sender == user_id!("@carl:example.com")
            && state_key.is_empty()
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

    let expected_url = "http://www.matrix.org";

    #[cfg(feature = "unstable-pre-spec")]
    let expected_url = Some(expected_url.to_owned());

    assert_matches!(
        from_json_value::<Raw<StateEvent<AnyStateEventContent>>>(json_data)
            .unwrap()
            .deserialize()
            .unwrap(),
        StateEvent {
            content: AnyStateEventContent::RoomAvatar(AvatarEventContent {
                info: Some(info),
                url,
                ..
            }),
            event_id,
            origin_server_ts,
            prev_content: None,
            room_id,
            sender,
            state_key,
            unsigned
        } if event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && room_id == room_id!("!roomid:room.com")
            && sender == user_id!("@carl:example.com")
            && state_key.is_empty()
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
                    #[cfg(feature = "unstable-pre-spec")]
                    blurhash: None,
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
                            && *thumb_mimetype == Some("image/png".into())
                            && *thumb_size == UInt::new(82595)
                            && *thumbnail_url == "mxc://matrix.org"
                    )
            )
            && url == expected_url
            && unsigned.is_empty()
    );
}

#[test]
fn deserialize_member_event_with_top_level_membership_field() {
    let json_data = json!({
        "content": {
            "avatar_url": null,
            "displayname": "example",
            "membership": "join"
        },
        "event_id": "$h29iv0s8:example.com",
        "membership": "join",
        "room_id": "!room:localhost",
        "origin_server_ts": 1,
        "sender": "@example:localhost",
        "state_key": "@example:localhost",
        "type": "m.room.member",
        "unsigned": {
            "age": 1,
            "replaces_state": "$151800111315tsynI:localhost",
            "prev_content": {
                "avatar_url": null,
                "displayname": "example",
                "membership": "invite"
            }
        }
    });

    assert_matches!(
        from_json_value::<AnyRoomEvent>(json_data)
            .unwrap(),
        AnyRoomEvent::State(
            AnyStateEvent::RoomMember(StateEvent {
                content,
                event_id,
                origin_server_ts,
                prev_content: None,
                sender,
                ..
            }
        )) if event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && sender == user_id!("@example:localhost")
            && content.displayname == Some("example".into())
    );
}

#[test]
fn deserialize_full_event_convert_to_sync() {
    let json_data = aliases_event_with_prev_content();

    let full_ev = from_json_value::<Raw<AnyStateEvent>>(json_data).unwrap().deserialize().unwrap();

    // Test conversion to sync event (without room_id field)
    let sync: AnySyncStateEvent = full_ev.into();
    let sync_json = to_json_value(sync).unwrap();

    assert_matches!(
        from_json_value::<Raw<AnySyncStateEvent>>(sync_json)
            .unwrap()
            .deserialize()
            .unwrap(),
        AnySyncStateEvent::RoomAliases(SyncStateEvent {
            content,
            event_id,
            origin_server_ts,
            prev_content: Some(prev_content),
            sender,
            state_key,
            unsigned,
        }) if content.aliases == vec![room_alias_id!("#somewhere:localhost")]
            && event_id == "$h29iv0s8:example.com"
            && origin_server_ts == UNIX_EPOCH + Duration::from_millis(1)
            && prev_content.aliases == vec![room_alias_id!("#inner:localhost")]
            && sender == "@carl:example.com"
            && state_key.is_empty()
            && unsigned.is_empty()
    );
}
