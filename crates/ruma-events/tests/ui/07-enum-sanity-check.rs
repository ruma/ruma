use ruma_events_macros::event_enum;

event_enum! {
    /// Any global account data event.
    kind: GlobalAccountData,
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
