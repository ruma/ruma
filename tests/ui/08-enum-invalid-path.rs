use ruma_events_macros::event_enum;

event_enum! {
    name: InvalidEvent,
    events: [
        "m.not.a.path",
    ]
}

event_enum! {
    name: InvalidEvent,
    events: [
        "not.a.path",
    ]
}

fn main() {}
