use ruma_events_macros::event_enum;

event_enum! {
    name: AnyStateEvent,
    events: [
        "m.not.a.path",
    ]
}

event_enum! {
    name: AnyStateEvent,
    events: [
        "not.a.path",
    ]
}

fn main() {}
