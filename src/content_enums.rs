use ruma_events_macros::event_content_enum;

event_content_enum! {
    /// Any message event's content.
    name: AnyMessageEventContent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.sticker",
    ]
}

event_content_enum! {
    /// Any state event's content.
    name: AnyStateEventContent,
    events: [
        "m.room.aliases",
        "m.room.avatar",
        "m.room.canonical_alias",
        "m.room.create",
        "m.room.encryption",
        "m.room.guest_access",
        "m.room.history_visibility",
        "m.room.join_rules",
        "m.room.member",
        "m.room.name",
        "m.room.pinned_events",
        "m.room.power_levels",
        "m.room.server_acl",
        "m.room.third_party_invite",
        "m.room.tombstone",
        "m.room.topic",
    ]
}

event_content_enum! {
    /// An ephemeral room event.
    name: AnyEphemeralRoomEventContent,
    events: [ "m.typing", "m.receipt" ]
}

event_content_enum! {
    /// A basic event.
    name: AnyBasicEventContent,
    events: [ "m.ignored_user_list", "m.room_key" ]
}
