use ruma_serde::StringEnum;

/// The type of an event.
///
/// This type can hold an arbitrary string. To check for events that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
// FIXME: Add `m.foo.bar` or `m.foo_bar` as a naming scheme in StringEnum and remove most rename
//        attributes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, StringEnum)]
pub enum EventType {
    /// m.call.answer
    #[ruma_enum(rename = "m.call.answer")]
    CallAnswer,

    /// m.call.candidates
    #[ruma_enum(rename = "m.call.candidates")]
    CallCandidates,

    /// m.call.hangup
    #[ruma_enum(rename = "m.call.hangup")]
    CallHangup,

    /// m.call.invite
    #[ruma_enum(rename = "m.call.invite")]
    CallInvite,

    /// m.direct
    #[ruma_enum(rename = "m.direct")]
    Direct,

    /// m.dummy
    #[ruma_enum(rename = "m.dummy")]
    Dummy,

    /// m.forwarded_room_key
    #[ruma_enum(rename = "m.forwarded_room_key")]
    ForwardedRoomKey,

    /// m.fully_read
    #[ruma_enum(rename = "m.fully_read")]
    FullyRead,

    /// m.key.verification.accept
    #[ruma_enum(rename = "m.key.verification.accept")]
    KeyVerificationAccept,

    /// m.key.verification.cancel
    #[ruma_enum(rename = "m.key.verification.cancel")]
    KeyVerificationCancel,

    /// m.key.verification.key
    #[ruma_enum(rename = "m.key.verification.key")]
    KeyVerificationKey,

    /// m.key.verification.mac
    #[ruma_enum(rename = "m.key.verification.mac")]
    KeyVerificationMac,

    /// m.key.verification.request
    #[ruma_enum(rename = "m.key.verification.request")]
    KeyVerificationRequest,

    /// m.key.verification.start
    #[ruma_enum(rename = "m.key.verification.start")]
    KeyVerificationStart,

    /// m.ignored_user_list
    #[ruma_enum(rename = "m.ignored_user_list")]
    IgnoredUserList,

    /// m.policy.rule.room
    #[ruma_enum(rename = "m.policy.rule.room")]
    PolicyRuleRoom,

    /// m.policy.rule.server
    #[ruma_enum(rename = "m.policy.rule.server")]
    PolicyRuleServer,

    /// m.policy.rule.user
    #[ruma_enum(rename = "m.policy.rule.user")]
    PolicyRuleUser,

    /// m.presence
    #[ruma_enum(rename = "m.presence")]
    Presence,

    /// m.push_rules
    #[ruma_enum(rename = "m.push_rules")]
    PushRules,

    /// m.receipt
    #[ruma_enum(rename = "m.receipt")]
    Receipt,

    /// m.room.aliases
    #[ruma_enum(rename = "m.room.aliases")]
    RoomAliases,

    /// m.room.avatar
    #[ruma_enum(rename = "m.room.avatar")]
    RoomAvatar,

    /// m.room.canonical_alias
    #[ruma_enum(rename = "m.room.canonical_alias")]
    RoomCanonicalAlias,

    /// m.room.create
    #[ruma_enum(rename = "m.room.create")]
    RoomCreate,

    /// m.room.encrypted
    #[ruma_enum(rename = "m.room.encrypted")]
    RoomEncrypted,

    /// m.room.encryption
    #[ruma_enum(rename = "m.room.encryption")]
    RoomEncryption,

    /// m.room.guest_access
    #[ruma_enum(rename = "m.room.guest_access")]
    RoomGuestAccess,

    /// m.room.history_visibility
    #[ruma_enum(rename = "m.room.history_visibility")]
    RoomHistoryVisibility,

    /// m.room.join_rules
    #[ruma_enum(rename = "m.room.join_rules")]
    RoomJoinRules,

    /// m.room.member
    #[ruma_enum(rename = "m.room.member")]
    RoomMember,

    /// m.room.message
    #[ruma_enum(rename = "m.room.message")]
    RoomMessage,

    /// m.room.message.feedback
    #[ruma_enum(rename = "m.room.message.feedback")]
    RoomMessageFeedback,

    /// m.room.name
    #[ruma_enum(rename = "m.room.name")]
    RoomName,

    /// m.room.pinned_events
    #[ruma_enum(rename = "m.room.pinned_events")]
    RoomPinnedEvents,

    /// m.room.power_levels
    #[ruma_enum(rename = "m.room.power_levels")]
    RoomPowerLevels,

    /// m.room.redaction
    #[ruma_enum(rename = "m.room.redaction")]
    RoomRedaction,

    /// m.room.server_acl
    #[ruma_enum(rename = "m.room.server_acl")]
    RoomServerAcl,

    /// m.room.third_party_invite
    #[ruma_enum(rename = "m.room.third_party_invite")]
    RoomThirdPartyInvite,

    /// m.room.tombstone
    #[ruma_enum(rename = "m.room.tombstone")]
    RoomTombstone,

    /// m.room.topic
    #[ruma_enum(rename = "m.room.topic")]
    RoomTopic,

    /// m.room_key
    #[ruma_enum(rename = "m.room_key")]
    RoomKey,

    /// m.room_key_request
    #[ruma_enum(rename = "m.room_key_request")]
    RoomKeyRequest,

    /// m.sticker
    #[ruma_enum(rename = "m.sticker")]
    Sticker,

    /// m.tag
    #[ruma_enum(rename = "m.tag")]
    Tag,

    /// m.typing
    #[ruma_enum(rename = "m.typing")]
    Typing,

    #[doc(hidden)]
    _Custom(String),
}

#[cfg(test)]
mod tests {
    use ruma_serde::test::serde_json_eq;
    use serde_json::json;

    use super::EventType;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn serialize_and_deserialize_from_display_form() {
        serde_json_eq(EventType::CallAnswer, json!("m.call.answer"));
        serde_json_eq(EventType::CallCandidates, json!("m.call.candidates"));
        serde_json_eq(EventType::CallHangup, json!("m.call.hangup"));
        serde_json_eq(EventType::CallInvite, json!("m.call.invite"));
        serde_json_eq(EventType::Direct, json!("m.direct"));
        serde_json_eq(EventType::Dummy, json!("m.dummy"));
        serde_json_eq(EventType::ForwardedRoomKey, json!("m.forwarded_room_key"));
        serde_json_eq(EventType::FullyRead, json!("m.fully_read"));
        serde_json_eq(EventType::KeyVerificationAccept, json!("m.key.verification.accept"));
        serde_json_eq(EventType::KeyVerificationCancel, json!("m.key.verification.cancel"));
        serde_json_eq(EventType::KeyVerificationKey, json!("m.key.verification.key"));
        serde_json_eq(EventType::KeyVerificationMac, json!("m.key.verification.mac"));
        serde_json_eq(EventType::KeyVerificationRequest, json!("m.key.verification.request"));
        serde_json_eq(EventType::KeyVerificationStart, json!("m.key.verification.start"));
        serde_json_eq(EventType::IgnoredUserList, json!("m.ignored_user_list"));
        serde_json_eq(EventType::PolicyRuleRoom, json!("m.policy.rule.room"));
        serde_json_eq(EventType::PolicyRuleServer, json!("m.policy.rule.server"));
        serde_json_eq(EventType::PolicyRuleUser, json!("m.policy.rule.user"));
        serde_json_eq(EventType::Presence, json!("m.presence"));
        serde_json_eq(EventType::PushRules, json!("m.push_rules"));
        serde_json_eq(EventType::Receipt, json!("m.receipt"));
        serde_json_eq(EventType::RoomAliases, json!("m.room.aliases"));
        serde_json_eq(EventType::RoomAvatar, json!("m.room.avatar"));
        serde_json_eq(EventType::RoomCanonicalAlias, json!("m.room.canonical_alias"));
        serde_json_eq(EventType::RoomCreate, json!("m.room.create"));
        serde_json_eq(EventType::RoomEncrypted, json!("m.room.encrypted"));
        serde_json_eq(EventType::RoomEncryption, json!("m.room.encryption"));
        serde_json_eq(EventType::RoomGuestAccess, json!("m.room.guest_access"));
        serde_json_eq(EventType::RoomHistoryVisibility, json!("m.room.history_visibility"));
        serde_json_eq(EventType::RoomJoinRules, json!("m.room.join_rules"));
        serde_json_eq(EventType::RoomMember, json!("m.room.member"));
        serde_json_eq(EventType::RoomMessage, json!("m.room.message"));
        serde_json_eq(EventType::RoomMessageFeedback, json!("m.room.message.feedback"));
        serde_json_eq(EventType::RoomName, json!("m.room.name"));
        serde_json_eq(EventType::RoomPinnedEvents, json!("m.room.pinned_events"));
        serde_json_eq(EventType::RoomPowerLevels, json!("m.room.power_levels"));
        serde_json_eq(EventType::RoomRedaction, json!("m.room.redaction"));
        serde_json_eq(EventType::RoomServerAcl, json!("m.room.server_acl"));
        serde_json_eq(EventType::RoomThirdPartyInvite, json!("m.room.third_party_invite"));
        serde_json_eq(EventType::RoomTombstone, json!("m.room.tombstone"));
        serde_json_eq(EventType::RoomTopic, json!("m.room.topic"));
        serde_json_eq(EventType::RoomKey, json!("m.room_key"));
        serde_json_eq(EventType::RoomKeyRequest, json!("m.room_key_request"));
        serde_json_eq(EventType::Sticker, json!("m.sticker"));
        serde_json_eq(EventType::Tag, json!("m.tag"));
        serde_json_eq(EventType::Typing, json!("m.typing"));
        serde_json_eq(EventType::_Custom("io.ruma.test".into()), json!("io.ruma.test"));
    }
}
