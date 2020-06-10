use ruma_events_macros::event_content_enum;

event_content_enum! {
    name: InvalidEvent,
    events: [
        "m.not.a.path",
    ]
}

event_content_enum! {
    name: InvalidEvent,
    events: [
        "not.a.path",
    ]
}

fn main() {}
