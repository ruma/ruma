use ruma_events_macros::event_enum;

event_enum! {
    kind: State,
    events: [
        "m.not.a.path",
    ]
}

event_enum! {
    kind: State,
    events: [
        "not.a.path",
    ]
}

fn main() {}
