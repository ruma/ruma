// `cargo bench` works, but if you use `cargo bench -- --save-baseline <name>`
// or pass any other args to it, it fails with the error
// `cargo bench unknown option --save-baseline`.
// To pass args to criterion, use this form
// `cargo bench --features criterion --bench <name of the bench> -- --save-baseline <name>`.

#![allow(unused_imports, dead_code)]

#[cfg(feature = "criterion")]
use criterion::{criterion_group, criterion_main, Criterion};
use ruma_events::{
    room::power_levels::PowerLevelsEventContent, AnyEvent, AnyRoomEvent, AnyStateEvent, StateEvent,
};
use ruma_serde::Raw;
use serde_json::json;

fn power_levels() -> serde_json::Value {
    json!({
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
        "room_id": "!room:localhost",
        "state_key": "",
        "type": "m.room.power_levels",
        "unsigned": {
            "age": 45
        }
    })
}

#[cfg(feature = "criterion")]
fn deserialize_any_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("deserialize to `AnyEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<Raw<AnyEvent>>(json_data.clone())
                .unwrap()
                .deserialize()
                .unwrap();
        })
    });
}

#[cfg(feature = "criterion")]
fn deserialize_any_room_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("deserialize to `AnyRoomEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<Raw<AnyRoomEvent>>(json_data.clone())
                .unwrap()
                .deserialize()
                .unwrap();
        })
    });
}

#[cfg(feature = "criterion")]
fn deserialize_any_state_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("deserialize to `AnyStateEvent`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<Raw<AnyStateEvent>>(json_data.clone())
                .unwrap()
                .deserialize()
                .unwrap();
        })
    });
}

#[cfg(feature = "criterion")]
fn deserialize_specific_event(c: &mut Criterion) {
    let json_data = power_levels();

    c.bench_function("deserialize to `StateEvent<PowerLevelsEventContent>`", |b| {
        b.iter(|| {
            let _ = serde_json::from_value::<Raw<StateEvent<PowerLevelsEventContent>>>(
                json_data.clone(),
            )
            .unwrap()
            .deserialize()
            .unwrap();
        })
    });
}

#[cfg(feature = "criterion")]
criterion_group!(
    benches,
    deserialize_any_event,
    deserialize_any_room_event,
    deserialize_any_state_event,
    deserialize_specific_event
);

#[cfg(feature = "criterion")]
criterion_main!(benches);

#[cfg(not(feature = "criterion"))]
fn main() {
    // To run the benchmarks the "criterion" feature must be enabled use:
    // `cargo bench --features criterion --bench event_deserialize`
    panic!("Enable the criterion feature to run benchmarks");
}
