use ruma_events_macros::event_enum;

event_enum! {
    enum State {
        "m.not.a.path",
    }
}

event_enum! {
    enum State {
        "not.a.path",
    }
}

fn main() {}
