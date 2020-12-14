use std::{collections::BTreeMap, sync::Arc};

use ruma::{
    events::EventType,
    identifiers::{EventId, RoomVersionId},
};
use serde_json::json;
use state_res::{StateEvent, StateResolution};
use tracing_subscriber as tracer;

mod utils;
use utils::{
    alice, bob, ella, member_content_ban, member_content_join, room_id, to_pdu_event, zara,
    TestStore, INITIAL_EVENTS, LOGGER,
};

#[test]
fn resolve_incoming_jr() {
    let _ = LOGGER.call_once(|| {
        tracer::fmt()
            .with_env_filter(tracer::EnvFilter::from_default_env())
            .init()
    });

    let events = INITIAL_EVENTS();
    let conflicted = JOIN_RULE();
    let store = TestStore(
        events
            .clone()
            .into_iter()
            .chain(conflicted.clone())
            .collect(),
    );

    let res = match StateResolution::resolve_incoming(
        &room_id(),
        &RoomVersionId::Version6,
        &events
            .iter()
            .map(|(_, ev)| ((ev.kind(), ev.state_key()), ev.event_id()))
            .collect(),
        conflicted
            .iter()
            .map(|(_, ev)| ((ev.kind(), ev.state_key()), ev.event_id()))
            .collect(),
        None,
        &store,
    ) {
        Ok(state) => state,
        Err(e) => panic!("{:?}", e),
    };

    assert_eq!(
        vec![
            "$CREATE:foo",
            "$JR:foo",
            "$IMA:foo",
            "$IMB:foo",
            "$IMC:foo",
            "$IPOWER:foo",
            "$START:foo"
        ],
        res.values().map(|id| id.to_string()).collect::<Vec<_>>()
    )
}

#[test]
fn resolve_incoming_ban() {
    let _ = LOGGER.call_once(|| {
        tracer::fmt()
            .with_env_filter(tracer::EnvFilter::from_default_env())
            .init()
    });

    let events = INITIAL_EVENTS();
    let conflicted = BAN_STATE_SET();
    let store = TestStore(
        events
            .clone()
            .into_iter()
            .chain(conflicted.clone())
            .collect(),
    );

    let res = match StateResolution::resolve_incoming(
        &room_id(),
        &RoomVersionId::Version6,
        &events
            .iter()
            .map(|(_, ev)| ((ev.kind(), ev.state_key()), ev.event_id()))
            .collect(),
        conflicted
            .iter()
            .map(|(_, ev)| ((ev.kind(), ev.state_key()), ev.event_id()))
            .collect(),
        None,
        &store,
    ) {
        Ok(state) => state,
        Err(e) => panic!("{:?}", e),
    };

    assert_eq!(
        vec![
            "$CREATE:foo",
            "$IJR:foo",
            "$IMA:foo",
            "$IMB:foo",
            "$IMC:foo",
            "$MB:foo",
            "$PB:foo",
            "$START:foo"
        ],
        res.values().map(|id| id.to_string()).collect::<Vec<_>>()
    )
}

#[allow(non_snake_case)]
fn JOIN_RULE() -> BTreeMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event(
            "JR",
            alice(),
            EventType::RoomJoinRules,
            Some(""),
            json!({"join_rule": "invite"}),
            &["CREATE", "IMA", "IPOWER"],
            &["START"],
        ),
        to_pdu_event(
            "IMZ",
            zara(),
            EventType::RoomPowerLevels,
            Some(zara().as_str()),
            member_content_join(),
            &["CREATE", "JR", "IPOWER"],
            &["START"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}

#[allow(non_snake_case)]
fn BAN_STATE_SET() -> BTreeMap<EventId, Arc<StateEvent>> {
    vec![
        to_pdu_event(
            "PA",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
            &["CREATE", "IMA", "IPOWER"], // auth_events
            &["START"],                   // prev_events
        ),
        to_pdu_event(
            "PB",
            alice(),
            EventType::RoomPowerLevels,
            Some(""),
            json!({"users": {alice(): 100, bob(): 50}}),
            &["CREATE", "IMA", "IPOWER"],
            &["END"],
        ),
        to_pdu_event(
            "MB",
            alice(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_ban(),
            &["CREATE", "IMA", "PB"],
            &["PA"],
        ),
        to_pdu_event(
            "IME",
            ella(),
            EventType::RoomMember,
            Some(ella().as_str()),
            member_content_join(),
            &["CREATE", "IJR", "PA"],
            &["MB"],
        ),
    ]
    .into_iter()
    .map(|ev| (ev.event_id(), ev))
    .collect()
}
