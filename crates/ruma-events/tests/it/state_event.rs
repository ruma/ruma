use assert_matches2::{assert_let, assert_matches};
use js_int::uint;
use ruma_common::{MilliSecondsSinceUnixEpoch, mxc_uri, serde::CanBeEmpty};
use ruma_events::{AnyStateEvent, AnySyncStateEvent, AnyTimelineEvent, StateEvent, SyncStateEvent};
use serde_json::{from_value as from_json_value, json};

#[test]
fn deserialize_room_name_with_prev_content() {
    let json = json!({
        "content": {
            "name": "Somewhere",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.room.name",
        "unsigned": {
            "prev_content": {
                "name": "Here",
            },
        },
    });

    assert_let!(Ok(AnyStateEvent::RoomName(StateEvent::Original(ev))) = from_json_value(json));
    assert_eq!(ev.content.name, "Somewhere");
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.room_id, "!roomid:room.com");
    assert_eq!(ev.sender, "@carl:example.com");

    let prev_content = ev.unsigned.prev_content.unwrap();
    assert_eq!(prev_content.name.as_deref(), Some("Here"));
}

#[test]
fn deserialize_sync_room_name_with_prev_content() {
    let json = json!({
        "content": {
            "name": "Somewhere",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.room.name",
        "unsigned": {
            "prev_content": {
                "name": "Here",
            },
        },
    });

    assert_let!(
        Ok(AnySyncStateEvent::RoomName(SyncStateEvent::Original(ev))) = from_json_value(json)
    );
    assert_eq!(ev.content.name, "Somewhere");
    assert_eq!(ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(ev.sender, "@carl:example.com");

    let prev_content = ev.unsigned.prev_content.unwrap();
    assert_eq!(prev_content.name.as_deref(), Some("Here"));
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
    let json = json!({
        "content": {
            "topic": "We welcome everybody",
        },
        "event_id": "$h29iv0s8:example.com",
        "origin_server_ts": 1,
        "room_id": "!roomid:room.com",
        "sender": "@carl:example.com",
        "state_key": "",
        "type": "m.room.topic",
        "unsigned": {
            "prev_content": {
                "topic": "No one is welcome here",
            },
        },
    });

    let full_ev: AnyStateEvent = from_json_value(json).unwrap();
    assert_matches!(&full_ev, AnyStateEvent::RoomTopic(StateEvent::Original(_)));
    assert_let!(
        AnySyncStateEvent::RoomTopic(SyncStateEvent::Original(sync_ev)) =
            AnySyncStateEvent::from(full_ev)
    );

    assert_eq!(sync_ev.content.topic, "We welcome everybody");
    assert_eq!(sync_ev.event_id, "$h29iv0s8:example.com");
    assert_eq!(sync_ev.origin_server_ts, MilliSecondsSinceUnixEpoch(uint!(1)));
    assert_eq!(
        sync_ev.unsigned.prev_content.unwrap().topic.as_deref(),
        Some("No one is welcome here")
    );
    assert_eq!(sync_ev.sender, "@carl:example.com");
}
