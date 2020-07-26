use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

/// The type of an event.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(from = "String", into = "String")]
pub enum EventType {
    /// m.call.answer
    CallAnswer,

    /// m.call.candidates
    CallCandidates,

    /// m.call.hangup
    CallHangup,

    /// m.call.invite
    CallInvite,

    /// m.direct
    Direct,

    /// m.dummy
    Dummy,

    /// m.forwarded_room_key
    ForwardedRoomKey,

    /// m.fully_read
    FullyRead,

    /// m.key.verification.accept
    KeyVerificationAccept,

    /// m.key.verification.cancel
    KeyVerificationCancel,

    /// m.key.verification.key
    KeyVerificationKey,

    /// m.key.verification.mac
    KeyVerificationMac,

    /// m.key.verification.request
    KeyVerificationRequest,

    /// m.key.verification.start
    KeyVerificationStart,

    /// m.ignored_user_list
    IgnoredUserList,

    /// m.presence
    Presence,

    /// m.push_rules
    PushRules,

    /// m.receipt
    Receipt,

    /// m.room.aliases
    RoomAliases,

    /// m.room.avatar
    RoomAvatar,

    /// m.room.canonical_alias
    RoomCanonicalAlias,

    /// m.room.create
    RoomCreate,

    /// m.room.encrypted
    RoomEncrypted,

    /// m.room.encryption
    RoomEncryption,

    /// m.room.guest_access
    RoomGuestAccess,

    /// m.room.history_visibility
    RoomHistoryVisibility,

    /// m.room.join_rules
    RoomJoinRules,

    /// m.room.member
    RoomMember,

    /// m.room.message
    RoomMessage,

    /// m.room.message.feedback
    RoomMessageFeedback,

    /// m.room.name
    RoomName,

    /// m.room.pinned_events
    RoomPinnedEvents,

    /// m.room.power_levels
    RoomPowerLevels,

    /// m.room.redaction
    RoomRedaction,

    /// m.room.server_acl
    RoomServerAcl,

    /// m.room.third_party_invite
    RoomThirdPartyInvite,

    /// m.room.tombstone
    RoomTombstone,

    /// m.room.topic
    RoomTopic,

    /// m.room_key
    RoomKey,

    /// m.room_key_request
    RoomKeyRequest,

    /// m.sticker
    Sticker,

    /// m.tag
    Tag,

    /// m.typing
    Typing,

    /// Any event that is not part of the specification.
    Custom(String),
}

impl Display for EventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let event_type_str = match *self {
            EventType::CallAnswer => "m.call.answer",
            EventType::CallCandidates => "m.call.candidates",
            EventType::CallHangup => "m.call.hangup",
            EventType::CallInvite => "m.call.invite",
            EventType::Direct => "m.direct",
            EventType::Dummy => "m.dummy",
            EventType::ForwardedRoomKey => "m.forwarded_room_key",
            EventType::FullyRead => "m.fully_read",
            EventType::KeyVerificationAccept => "m.key.verification.accept",
            EventType::KeyVerificationCancel => "m.key.verification.cancel",
            EventType::KeyVerificationKey => "m.key.verification.key",
            EventType::KeyVerificationMac => "m.key.verification.mac",
            EventType::KeyVerificationRequest => "m.key.verification.request",
            EventType::KeyVerificationStart => "m.key.verification.start",
            EventType::IgnoredUserList => "m.ignored_user_list",
            EventType::Presence => "m.presence",
            EventType::PushRules => "m.push_rules",
            EventType::Receipt => "m.receipt",
            EventType::RoomAliases => "m.room.aliases",
            EventType::RoomAvatar => "m.room.avatar",
            EventType::RoomCanonicalAlias => "m.room.canonical_alias",
            EventType::RoomCreate => "m.room.create",
            EventType::RoomEncrypted => "m.room.encrypted",
            EventType::RoomEncryption => "m.room.encryption",
            EventType::RoomGuestAccess => "m.room.guest_access",
            EventType::RoomHistoryVisibility => "m.room.history_visibility",
            EventType::RoomJoinRules => "m.room.join_rules",
            EventType::RoomMember => "m.room.member",
            EventType::RoomMessage => "m.room.message",
            EventType::RoomMessageFeedback => "m.room.message.feedback",
            EventType::RoomName => "m.room.name",
            EventType::RoomPinnedEvents => "m.room.pinned_events",
            EventType::RoomPowerLevels => "m.room.power_levels",
            EventType::RoomRedaction => "m.room.redaction",
            EventType::RoomServerAcl => "m.room.server_acl",
            EventType::RoomThirdPartyInvite => "m.room.third_party_invite",
            EventType::RoomTombstone => "m.room.tombstone",
            EventType::RoomTopic => "m.room.topic",
            EventType::RoomKey => "m.room_key",
            EventType::RoomKeyRequest => "m.room_key_request",
            EventType::Sticker => "m.sticker",
            EventType::Tag => "m.tag",
            EventType::Typing => "m.typing",
            EventType::Custom(ref event_type) => event_type,
        };

        write!(f, "{}", event_type_str)
    }
}

impl<T> From<T> for EventType
where
    T: Into<String> + AsRef<str>,
{
    fn from(s: T) -> EventType {
        match s.as_ref() {
            "m.call.answer" => EventType::CallAnswer,
            "m.call.candidates" => EventType::CallCandidates,
            "m.call.hangup" => EventType::CallHangup,
            "m.call.invite" => EventType::CallInvite,
            "m.direct" => EventType::Direct,
            "m.dummy" => EventType::Dummy,
            "m.forwarded_room_key" => EventType::ForwardedRoomKey,
            "m.fully_read" => EventType::FullyRead,
            "m.key.verification.accept" => EventType::KeyVerificationAccept,
            "m.key.verification.cancel" => EventType::KeyVerificationCancel,
            "m.key.verification.key" => EventType::KeyVerificationKey,
            "m.key.verification.mac" => EventType::KeyVerificationMac,
            "m.key.verification.request" => EventType::KeyVerificationRequest,
            "m.key.verification.start" => EventType::KeyVerificationStart,
            "m.ignored_user_list" => EventType::IgnoredUserList,
            "m.presence" => EventType::Presence,
            "m.push_rules" => EventType::PushRules,
            "m.receipt" => EventType::Receipt,
            "m.room.aliases" => EventType::RoomAliases,
            "m.room.avatar" => EventType::RoomAvatar,
            "m.room.canonical_alias" => EventType::RoomCanonicalAlias,
            "m.room.create" => EventType::RoomCreate,
            "m.room.encrypted" => EventType::RoomEncrypted,
            "m.room.encryption" => EventType::RoomEncryption,
            "m.room.guest_access" => EventType::RoomGuestAccess,
            "m.room.history_visibility" => EventType::RoomHistoryVisibility,
            "m.room.join_rules" => EventType::RoomJoinRules,
            "m.room.member" => EventType::RoomMember,
            "m.room.message" => EventType::RoomMessage,
            "m.room.message.feedback" => EventType::RoomMessageFeedback,
            "m.room.name" => EventType::RoomName,
            "m.room.pinned_events" => EventType::RoomPinnedEvents,
            "m.room.power_levels" => EventType::RoomPowerLevels,
            "m.room.redaction" => EventType::RoomRedaction,
            "m.room.server_acl" => EventType::RoomServerAcl,
            "m.room.third_party_invite" => EventType::RoomThirdPartyInvite,
            "m.room.tombstone" => EventType::RoomTombstone,
            "m.room.topic" => EventType::RoomTopic,
            "m.room_key" => EventType::RoomKey,
            "m.room_key_request" => EventType::RoomKeyRequest,
            "m.sticker" => EventType::Sticker,
            "m.tag" => EventType::Tag,
            "m.typing" => EventType::Typing,
            _ => EventType::Custom(s.into()),
        }
    }
}

impl From<EventType> for String {
    fn from(event_type: EventType) -> String {
        event_type.to_string()
    }
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
        serde_json_eq(EventType::Custom("io.ruma.test".into()), json!("io.ruma.test"));
    }
}
