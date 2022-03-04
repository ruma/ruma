use js_int::{uint, UInt};
use matches::assert_matches;
use ruma_common::{
    event_id,
    events::{
        room::{
            aliases::RoomAliasesEventContent,
            avatar::{ImageInfo, RoomAvatarEventContent},
            ThumbnailInfo,
        },
        AnyRoomEvent, AnyStateEvent, AnyStateEventContent, AnySyncStateEvent, RawExt, StateEvent,
        SyncStateEvent, Unsigned,
    },
    mxc_uri, room_alias_id, room_id, user_id, MilliSecondsSinceUnixEpoch,
};
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
        content: RoomAliasesEventContent::new(vec![
            room_alias_id!("#somewhere:localhost").to_owned()
        ]),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        prev_content: Some(RoomAliasesEventContent::new(vec![
            room_alias_id!("#inner:localhost").to_owned()
        ])),
        room_id: room_id!("!roomid:room.com").to_owned(),
        sender: user_id!("@carl:example.com").to_owned(),
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
        content: RoomAliasesEventContent::new(vec![
            room_alias_id!("#somewhere:localhost").to_owned()
        ]),
        event_id: event_id!("$h29iv0s8:example.com").to_owned(),
        origin_server_ts: MilliSecondsSinceUnixEpoch(uint!(1)),
        prev_content: None,
        room_id: room_id!("!roomid:room.com").to_owned(),
        sender: user_id!("@carl:example.com").to_owned(),
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
        from_json_value::<AnyStateEvent>(json_data).unwrap(),
        AnyStateEvent::RoomAliases(StateEvent {
            content,
            event_id,
            origin_server_ts,
            prev_content: Some(prev_content),
            room_id,
            sender,
            state_key,
            unsigned,
        }) if content.aliases == vec![room_alias_id!("#somewhere:localhost")]
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
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
        from_json_value::<AnySyncStateEvent>(json_data)
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
            && event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && prev_content.aliases == vec![room_alias_id!("#inner:localhost")]
            && sender == user_id!("@carl:example.com")
            && state_key.is_empty()
            && unsigned.is_empty()
    );
}

#[test]
#[allow(clippy::cmp_owned)] // seems buggy
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
                "thumbnail_url": "mxc://matrix.org/98irRSS23srs",
                "w": 1011
              },
            "url": "mxc://matrix.org/rnsldl8srs98IRrs"
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.room.avatar"
    });

    let expected_url = mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs").to_owned();

    #[cfg(feature = "unstable-pre-spec")]
    let expected_url = Some(expected_url);

    assert_matches!(
        from_json_value::<AnyStateEvent>(json_data).unwrap(),
        AnyStateEvent::RoomAvatar(StateEvent {
            content: RoomAvatarEventContent {
                info: Some(info),
                url,
                ..
            },
            event_id,
            origin_server_ts,
            prev_content: None,
            room_id,
            sender,
            state_key,
            unsigned
        }) if event_id == event_id!("$h29iv0s8:example.com")
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
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
                    ..
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
                            ..
                        } if *thumb_width == UInt::new(800)
                            && *thumb_height == UInt::new(334)
                            && *thumb_mimetype == Some("image/png".into())
                            && *thumb_size == UInt::new(82595)
                            && thumbnail_url == mxc_uri!("mxc://matrix.org/98irRSS23srs")
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
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && sender == user_id!("@example:localhost")
            && content.displayname == Some("example".into())
    );
}

#[test]
fn deserialize_full_event_convert_to_sync() {
    let json_data = aliases_event_with_prev_content();

    let full_ev: AnyStateEvent = from_json_value(json_data).unwrap();

    assert_matches!(
        AnySyncStateEvent::from(full_ev),
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
            && origin_server_ts == MilliSecondsSinceUnixEpoch(uint!(1))
            && prev_content.aliases == vec![room_alias_id!("#inner:localhost")]
            && sender == "@carl:example.com"
            && state_key.is_empty()
            && unsigned.is_empty()
    );
}
