use ruma_events_macros::event_enum;

event_enum! {
    /// Any basic event.
    kind: Basic,
    events: [
        "m.direct",
        #[cfg(test)]
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
        #[cfg(any())]
        "m.ruma_test",
    ]
}

fn main() {}
