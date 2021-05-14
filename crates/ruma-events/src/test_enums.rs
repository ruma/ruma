use ruma_events_macros::test_event_enum;

test_event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        events: [
            "m.direct",
            "m.ignored_user_list",
            "m.push_rules",
        ]
    }

    /// blah blah
    enum State {
        events: [
            "m.policy.rule.room",
            "m.policy.rule.server",
            "m.policy.rule.user",
            "m.room.aliases",
            "m.room.avatar",
        ]
    }
}
