use assert_matches2::assert_matches;
use js_int::uint;
use ruma_common::{canonical_json::assert_to_canonical_json_eq, mxc_uri};
use ruma_events::{
    AnyStrippedStateEvent,
    room::{join_rules::JoinRule, topic::RoomTopicEventContent},
};
use serde_json::{from_value as from_json_value, json};

#[test]
fn serialize_stripped_state_event_any_content() {
    assert_to_canonical_json_eq!(
        RoomTopicEventContent::new("Testing room".into()),
        json!({
            "topic": "Testing room",
            "m.topic": {
                "m.text": [
                    { "body": "Testing room" },
                ],
            },
        })
    );
}

#[test]
fn deserialize_stripped_state_events() {
    let name_event = json!({
        "type": "m.room.name",
        "state_key": "",
        "sender": "@example:localhost",
        "content": { "name": "Ruma" }
    });

    let join_rules_event = json!({
        "type": "m.room.join_rules",
        "state_key": "",
        "sender": "@example:localhost",
        "content": { "join_rule": "public" }
    });

    let avatar_event = json!({
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
                "thumbnail_url": "mxc://example.com/THumbNa1l"
            },
            "thumbnail_info": {
                "h": 16,
                "w": 16,
                "mimetype": "image/jpeg",
                "size": 32
            },
            "thumbnail_url": "mxc://example.com/THumbNa1l",
            "url": "mxc://example.com/iMag3"
        }
    });

    let ev = from_json_value::<AnyStrippedStateEvent>(name_event).unwrap();
    assert_matches!(ev, AnyStrippedStateEvent::RoomName(ev));
    assert_eq!(ev.content.name.as_deref(), Some("Ruma"));
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let ev = from_json_value::<AnyStrippedStateEvent>(join_rules_event).unwrap();
    assert_matches!(ev, AnyStrippedStateEvent::RoomJoinRules(ev));
    assert_eq!(ev.content.join_rule, JoinRule::Public);
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let ev = from_json_value::<AnyStrippedStateEvent>(avatar_event).unwrap();
    assert_matches!(ev, AnyStrippedStateEvent::RoomAvatar(ev));
    assert_eq!(ev.content.url.unwrap(), mxc_uri!("mxc://example.com/iMag3"));
    assert_eq!(ev.sender.to_string(), "@example:localhost");

    let image_info = ev.content.info.unwrap();
    assert_eq!(image_info.height, Some(uint!(128)));
    assert_eq!(image_info.width, Some(uint!(128)));
    assert_eq!(image_info.mimetype.as_deref(), Some("image/jpeg"));
    assert_eq!(image_info.size, Some(uint!(1024)));
    assert_eq!(image_info.thumbnail_info.unwrap().size, Some(uint!(32)));
}

#[test]
#[cfg(feature = "unstable-msc4319")]
fn deserialize_stripped_state_msc4319_format() {
    use js_int::uint;
    use ruma_common::{MilliSecondsSinceUnixEpoch, user_id};
    use ruma_events::room::member::MembershipState;

    let user_id = user_id!("@patrick:localhost");
    let origin_server_ts = MilliSecondsSinceUnixEpoch(uint!(1_000_000));

    let event_json = json!({
        "content": {
            "membership": "invite",
        },
        "origin_server_ts": origin_server_ts,
        "sender": user_id,
        "state_key": user_id,
        "type": "m.room.member",
        "unsigned": {
            "prev_content": {
                "membership": "knock",
            },
        },
    });
    assert_matches!(
        from_json_value::<AnyStrippedStateEvent>(event_json).unwrap(),
        AnyStrippedStateEvent::RoomMember(member_event)
    );
    assert_eq!(member_event.content.membership, MembershipState::Invite);
    assert_eq!(member_event.origin_server_ts, Some(origin_server_ts));
    assert_eq!(member_event.sender, user_id);
    assert_eq!(member_event.state_key, user_id);

    let unsigned = member_event.unsigned.unwrap().deserialize().unwrap();
    assert_eq!(unsigned.prev_content.unwrap().membership, MembershipState::Knock);
}
