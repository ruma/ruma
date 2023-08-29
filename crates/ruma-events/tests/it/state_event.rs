use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{mxc_uri, room_alias_id, serde::CanBeEmpty, MilliSecondsSinceUnixEpoch};
use ruma_events::{
    room::aliases::RoomAliasesEventContent, AnyStateEvent, AnySyncStateEvent, AnyTimelineEvent,
    StateEvent, SyncStateEvent,
};
use serde_json::{from_value as from_json_value, json, Value as JsonValue};

fn aliases_event_with_prev_content() -> JsonValue {
    json!({
        "content": {
            "aliases": ["#somewhere:localhost"],
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "state_key": "room.com",
        "type": "m.room.aliases",
        "unsigned": {
            "prev_content": {
                "aliases": ["#inner:localhost"],
            },
        },
    })
}

#[test]
fn deserialize_aliases_content() {
    let json_data = json!({
        "aliases": ["#somewhere:localhost"],
    });

    let content = from_json_value::<RoomAliasesEventContent>(json_data).unwrap();
    assert_eq!(content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
}

#[test]
fn deserialize_aliases_with_prev_content() {
    let json_data = aliases_event_with_prev_content();

    assert_matches!(
        from_json_value::<AnyStateEvent>(json_data),
        Ok(AnyStateEvent::RoomAliases(StateEvent::Original(ev)))
    );
    assert_eq!(ev.content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");

    let prev_content = ev.unsigned.prev_content.unwrap();
    assert_eq!(prev_content.aliases.unwrap(), vec![room_alias_id!("#inner:localhost")]);
}

#[test]
fn deserialize_aliases_sync_with_room_id() {
    // The same JSON can be used to create a sync event, it just ignores the `room_id` field
    let json_data = aliases_event_with_prev_content();

    assert_matches!(
        from_json_value::<AnySyncStateEvent>(json_data),
        Ok(AnySyncStateEvent::RoomAliases(SyncStateEvent::Original(ev)))
    );
    assert_eq!(ev.content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.sender, "@carl:example.com");

    let prev_content = ev.unsigned.prev_content.unwrap();
    assert_eq!(prev_content.aliases.unwrap(), vec![room_alias_id!("#inner:localhost")]);
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

    assert_matches!(
        from_json_value::<AnyStateEvent>(json_data),
        Ok(AnyStateEvent::RoomAvatar(StateEvent::Original(ev)))
    );
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");
    assert!(ev.unsigned.is_empty());
    assert_eq!(ev.content.url.as_deref(), Some(mxc_uri!("mxc://matrix.org/rnsldl8srs98IRrs")));

    let info = ev.content.info.unwrap();
    assert_eq!(info.height, Some(uint!(423)));
    assert_eq!(info.width, Some(uint!(1011)));
    assert_eq!(info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(info.size, Some(uint!(84242)));
    assert_eq!(info.thumbnail_url.as_deref(), Some(mxc_uri!("mxc://matrix.org/98irRSS23srs")));

    let thumbnail_info = info.thumbnail_info.unwrap();
    assert_eq!(thumbnail_info.width, Some(uint!(800)));
    assert_eq!(thumbnail_info.height, Some(uint!(334)));
    assert_eq!(thumbnail_info.mimetype.as_deref(), Some("image/png"));
    assert_eq!(thumbnail_info.size, Some(uint!(82595)));
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
        from_json_value::<AnyTimelineEvent>(json_data),
        Ok(AnyTimelineEvent::State(AnyStateEvent::RoomMember(StateEvent::Original(ev))))
    );
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.sender, "@example:localhost");
    assert_eq!(ev.content.displayname.as_deref(), Some("example"));
}

#[test]
fn deserialize_full_event_convert_to_sync() {
    let json_data = aliases_event_with_prev_content();

    let full_ev: AnyStateEvent = from_json_value(json_data).unwrap();
    assert_matches!(
        AnySyncStateEvent::from(full_ev),
        AnySyncStateEvent::RoomAliases(SyncStateEvent::Original(sync_ev))
    );

    assert_eq!(sync_ev.content.aliases, vec![room_alias_id!("#somewhere:localhost")]);
    assert_eq!(sync_ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(sync_ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(
        sync_ev.unsigned.prev_content.unwrap().aliases.unwrap(),
        vec![room_alias_id!("#inner:localhost")]
    );
    assert_eq!(sync_ev.sender, "@carl:example.com");
}
