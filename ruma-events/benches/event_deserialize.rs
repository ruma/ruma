// In order to run use either `cargo bench` or to save if you
// want to save the result to compare branches use
// `cargo bench --bench event_deserialize -- --save-baseline test-output`
// Since we are in a workspace the default `cargo bench` still picks up
// the args passed to it to avoid this use the above command.

use criterion::{criterion_group, criterion_main, Criterion};
use ruma_events::{
    room::{message::MessageEventContent, power_levels::PowerLevelsEventContent},
    AnyRoomEventStub, EventJson, MessageEvent, StateEventStub,
};
use serde_json::json;

fn deserialize_any_event_stub(c: &mut Criterion) {
    let json_data = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    });
    c.bench_function("deserialize to `AnyRoomEventStub`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<EventJson<AnyRoomEventStub>>(json_data.clone())
                .unwrap()
                .deserialize()
                .unwrap();
        })
    });
}

fn deserialize_specific_event_stub(c: &mut Criterion) {
    let json_data = json!({
        "content": {
            "ban": 50,
            "events": {
                "m.room.avatar": 50,
                "m.room.canonical_alias": 50,
                "m.room.history_visibility": 100,
                "m.room.name": 50,
                "m.room.power_levels": 100
            },
            "events_default": 0,
            "invite": 0,
            "kick": 50,
            "redact": 50,
            "state_default": 50,
            "users": {
                "@example:localhost": 100
            },
            "users_default": 0
        },
        "event_id": "$15139375512JaHAW:localhost",
        "origin_server_ts": 45,
        "sender": "@example:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    });
    c.bench_function("deserialize to `StateEventStub<PowerLevelsEventContent>`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<EventJson<StateEventStub<PowerLevelsEventContent>>>(
                json_data.clone(),
            )
            .unwrap()
            .deserialize()
            .unwrap();
        })
    });
}

fn deserialize_message_event(c: &mut Criterion) {
    let json_data = json!({
        "type": "m.room.message",
        "event_id": "$143273582443PhrSn:example.org",
        "origin_server_ts": 10_000,
        "room_id": "!testroomid:example.org",
        "sender": "@user:example.org",
        "content": {
            "body": "Hello, World!",
            "msgtype": "m.text",
            "format": "org.matrix.custom.html",
            "formatted_body": "Hello, <em>World</em>!",
        }
    });

    c.bench_function("deserialize to `MessageEvent<MessageEventContent>`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<EventJson<MessageEvent<MessageEventContent>>>(
                json_data.clone(),
            )
            .unwrap()
            .deserialize()
            .unwrap();
        })
    });
}

criterion_group!(
    benches,
    deserialize_any_event_stub,
    deserialize_specific_event_stub,
    deserialize_message_event
);

criterion_main!(benches);
