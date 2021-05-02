use ruma_events_macros::event_enum;

event_enum! {
    kind: NotReal,
    events: [
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
    ]
}

fn main() {}
