use ruma_events_macros::event_content_enum;

event_content_enum! {
    /// Any basic event.
    name: AnyBasicEventContent,
    events: [
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
    ]
}

fn main() {}
