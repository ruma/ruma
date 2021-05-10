use ruma_events_macros::event_enum;

event_enum! {
    /// Any global account data event.
    kind: GlobalAccountData,
    events: [
        "m.direct",
        #[cfg(test)]
        "m.ignored_user_list",
        "m.push_rules",
        #[cfg(any())]
        "m.ruma_test",
    ]
}

fn main() {}
