use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use serde::{Deserialize, Serialize};

/// The type of an event.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
// Cow<str> because deserialization sometimes needs to copy to unescape things
#[serde(from = "Cow<'_, str>", into = "String")]
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

    /// Additional variants may be added in the future and will not be considered breaking changes
    /// to ruma-events.
    #[doc(hidden)]
    __Nonexhaustive,
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
            EventType::__Nonexhaustive => {
                panic!("__Nonexhaustive enum variant is not intended for use.")
            }
        };

        write!(f, "{}", event_type_str)
    }
}

impl From<Cow<'_, str>> for EventType {
    fn from(s: Cow<'_, str>) -> EventType {
        match &s as &str {
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
            _ => EventType::Custom(s.into_owned()),
        }
    }
}

impl<'a> From<&str> for EventType {
    fn from(s: &str) -> EventType {
        EventType::from(Cow::Borrowed(s))
    }
}

impl From<EventType> for String {
    fn from(event_type: EventType) -> String {
        event_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn serialize_and_deserialize_from_display_form() {
        serde_eq!(r#""m.call.answer""#, EventType::CallAnswer);
        serde_eq!(r#""m.call.candidates""#, EventType::CallCandidates);
        serde_eq!(r#""m.call.hangup""#, EventType::CallHangup);
        serde_eq!(r#""m.call.invite""#, EventType::CallInvite);
        serde_eq!(r#""m.direct""#, EventType::Direct);
        serde_eq!(r#""m.dummy""#, EventType::Dummy);
        serde_eq!(r#""m.forwarded_room_key""#, EventType::ForwardedRoomKey);
        serde_eq!(r#""m.fully_read""#, EventType::FullyRead);
        serde_eq!(
            r#""m.key.verification.accept""#,
            EventType::KeyVerificationAccept
        );
        serde_eq!(
            r#""m.key.verification.cancel""#,
            EventType::KeyVerificationCancel
        );
        serde_eq!(r#""m.key.verification.key""#, EventType::KeyVerificationKey);
        serde_eq!(r#""m.key.verification.mac""#, EventType::KeyVerificationMac);
        serde_eq!(
            r#""m.key.verification.request""#,
            EventType::KeyVerificationRequest
        );
        serde_eq!(
            r#""m.key.verification.start""#,
            EventType::KeyVerificationStart
        );
        serde_eq!(r#""m.ignored_user_list""#, EventType::IgnoredUserList);
        serde_eq!(r#""m.presence""#, EventType::Presence);
        serde_eq!(r#""m.push_rules""#, EventType::PushRules);
        serde_eq!(r#""m.receipt""#, EventType::Receipt);
        serde_eq!(r#""m.room.aliases""#, EventType::RoomAliases);
        serde_eq!(r#""m.room.avatar""#, EventType::RoomAvatar);
        serde_eq!(r#""m.room.canonical_alias""#, EventType::RoomCanonicalAlias);
        serde_eq!(r#""m.room.create""#, EventType::RoomCreate);
        serde_eq!(r#""m.room.encrypted""#, EventType::RoomEncrypted);
        serde_eq!(r#""m.room.encryption""#, EventType::RoomEncryption);
        serde_eq!(r#""m.room.guest_access""#, EventType::RoomGuestAccess);
        serde_eq!(
            r#""m.room.history_visibility""#,
            EventType::RoomHistoryVisibility
        );
        serde_eq!(r#""m.room.join_rules""#, EventType::RoomJoinRules);
        serde_eq!(r#""m.room.member""#, EventType::RoomMember);
        serde_eq!(r#""m.room.message""#, EventType::RoomMessage);
        serde_eq!(
            r#""m.room.message.feedback""#,
            EventType::RoomMessageFeedback
        );
        serde_eq!(r#""m.room.name""#, EventType::RoomName);
        serde_eq!(r#""m.room.pinned_events""#, EventType::RoomPinnedEvents);
        serde_eq!(r#""m.room.power_levels""#, EventType::RoomPowerLevels);
        serde_eq!(r#""m.room.redaction""#, EventType::RoomRedaction);
        serde_eq!(r#""m.room.server_acl""#, EventType::RoomServerAcl);
        serde_eq!(
            r#""m.room.third_party_invite""#,
            EventType::RoomThirdPartyInvite
        );
        serde_eq!(r#""m.room.tombstone""#, EventType::RoomTombstone);
        serde_eq!(r#""m.room.topic""#, EventType::RoomTopic);
        serde_eq!(r#""m.room_key""#, EventType::RoomKey);
        serde_eq!(r#""m.room_key_request""#, EventType::RoomKeyRequest);
        serde_eq!(r#""m.sticker""#, EventType::Sticker);
        serde_eq!(r#""m.tag""#, EventType::Tag);
        serde_eq!(r#""m.typing""#, EventType::Typing);
        serde_eq!(
            r#""io.ruma.test""#,
            EventType::Custom("io.ruma.test".to_string())
        );
    }
}
