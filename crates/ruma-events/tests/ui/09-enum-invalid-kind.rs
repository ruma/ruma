use ruma_macros::event_enum;

event_enum! {
    enum NotReal {
        "m.direct",
        "m.dummy",
        "m.ignored_user_list",
        "m.push_rules",
        "m.room_key",
    }
}

fn main() {}
