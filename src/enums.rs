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
        "m.tag",
    ]
}

event_enum! {
    /// Any ephemeral room event.
    name: AnyEphemeralRoomEvent,
    events: [ "m.typing", "m.receipt" ]
}

event_enum! {
    /// Any message event.
    name: AnyMessageEvent,
    events: [
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        "m.room.message",
        "m.room.message.feedback",
        "m.sticker",
    ]
}

event_enum! {
    /// Any state event.
    name: AnyStateEvent,
    events: [
        "m.room.aliases",
        "m.room.avatar",
        // "m.room.canonical_alias",
        // "m.room.create",
        // "m.room.encryption",
        // "m.room.guest_access",
        // "m.room.history_visibility",
        // "m.room.join_rules",
        "m.room.member",
        // "m.room.name",
        // "m.room.pinned_events",
        // "m.room.power_levels",
        // "m.room.server_acl",
        // "m.room.third_party_invite",
        // "m.room.tombstone",
        // "m.room.topic",
    ]
}

event_enum! {
    /// Any to-device event.
    name: AnyToDeviceEvent,
    events: [
        "m.dummy",
        "m.room_key",
        //"m.room_key_request",
        //"m.forwarded_room_key",
        //"m.key.verification.request",
        "m.key.verification.start",
        //"m.key.verification.cancel",
        //"m.key.verification.accept",
        //"m.key.verification.key",
        //"m.key.verification.mac",
        //"m.room.encrypted",
    ]
}
