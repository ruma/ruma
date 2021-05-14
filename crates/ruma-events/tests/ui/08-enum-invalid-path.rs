use ruma_events_macros::event_enum;

event_enum! {
    enum State {
        events: [
            "m.not.a.path",
        ]
    }
}

event_enum! {
    enum State {
        events: [
            "not.a.path",
        ]
    }
}

fn main() {}
