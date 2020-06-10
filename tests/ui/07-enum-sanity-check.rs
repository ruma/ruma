use ruma_events_macros::event_enum;

event_enum! {
    /// Any basic event.
    name: AnyBasicEvent,
    events: [
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
    ]
}

fn main() {}
