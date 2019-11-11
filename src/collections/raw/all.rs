//! Enums for heterogeneous collections of events, inclusive for every event type that implements
//! the trait of the same name.

use serde::{de::Error as _, Deserialize, Deserializer};
use serde_json::Value;

use crate::{
    call::{
        answer::raw::AnswerEvent, candidates::raw::CandidatesEvent, hangup::raw::HangupEvent,
        invite::raw::InviteEvent,
    },
    custom::raw::CustomEvent,
    custom_room::raw::CustomRoomEvent,
    custom_state::raw::CustomStateEvent,
    direct::raw::DirectEvent,
    dummy::raw::DummyEvent,
    forwarded_room_key::raw::ForwardedRoomKeyEvent,
    fully_read::raw::FullyReadEvent,
    ignored_user_list::raw::IgnoredUserListEvent,
    key::verification::{
        accept::raw::AcceptEvent, cancel::raw::CancelEvent, key::raw::KeyEvent, mac::raw::MacEvent,
        request::raw::RequestEvent, start::raw::StartEvent,
    },
    presence::raw::PresenceEvent,
    push_rules::raw::PushRulesEvent,
    receipt::raw::ReceiptEvent,
    room::{
        aliases::raw::AliasesEvent,
        avatar::raw::AvatarEvent,
        canonical_alias::raw::CanonicalAliasEvent,
        create::raw::CreateEvent,
        encrypted::raw::EncryptedEvent,
        encryption::raw::EncryptionEvent,
        guest_access::raw::GuestAccessEvent,
        history_visibility::raw::HistoryVisibilityEvent,
        join_rules::raw::JoinRulesEvent,
        member::raw::MemberEvent,
        message::{feedback::raw::FeedbackEvent, raw::MessageEvent},
        name::raw::NameEvent,
        pinned_events::raw::PinnedEventsEvent,
        power_levels::raw::PowerLevelsEvent,
        redaction::raw::RedactionEvent,
        server_acl::raw::ServerAclEvent,
        third_party_invite::raw::ThirdPartyInviteEvent,
        tombstone::raw::TombstoneEvent,
        topic::raw::TopicEvent,
    },
    room_key::raw::RoomKeyEvent,
    room_key_request::raw::RoomKeyRequestEvent,
    sticker::raw::StickerEvent,
    tag::raw::TagEvent,
    typing::raw::TypingEvent,
    util::get_field,
    EventType,
};

/// A basic event, room event, or state event.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    /// m.call.answer
    CallAnswer(AnswerEvent),

    /// m.call.candidates
    CallCandidates(CandidatesEvent),

    /// m.call.hangup
    CallHangup(HangupEvent),

    /// m.call.invite
    CallInvite(InviteEvent),

    /// m.direct
    Direct(DirectEvent),

    /// m.dummy
    Dummy(DummyEvent),

    /// m.forwarded_room_key
    ForwardedRoomKey(ForwardedRoomKeyEvent),

    /// m.fully_read
    FullyRead(FullyReadEvent),

    /// m.ignored_user_list
    IgnoredUserList(IgnoredUserListEvent),

    /// m.key.verification.accept
    KeyVerificationAccept(AcceptEvent),

    /// m.key.verification.cancel
    KeyVerificationCancel(CancelEvent),

    /// m.key.verification.key
    KeyVerificationKey(KeyEvent),

    /// m.key.verification.mac
    KeyVerificationMac(MacEvent),

    /// m.key.verification.request
    KeyVerificationRequest(RequestEvent),

    /// m.key.verification.start
    KeyVerificationStart(StartEvent),

    /// m.presence
    Presence(PresenceEvent),

    /// m.push_rules
    PushRules(PushRulesEvent),

    /// m.receipt
    Receipt(ReceiptEvent),

    /// m.room.aliases
    RoomAliases(AliasesEvent),

    /// m.room.avatar
    RoomAvatar(AvatarEvent),

    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),

    /// m.room.create
    RoomCreate(CreateEvent),

    /// m.room.encrypted
    RoomEncrypted(EncryptedEvent),

    /// m.room.encryption
    RoomEncryption(EncryptionEvent),

    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),

    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),

    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),

    /// m.room.member
    RoomMember(MemberEvent),

    /// m.room.message
    RoomMessage(MessageEvent),

    /// m.room.message.feedback
    RoomMessageFeedback(FeedbackEvent),

    /// m.room.name
    RoomName(NameEvent),

    /// m.room.pinned_events
    RoomPinnedEvents(PinnedEventsEvent),

    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),

    /// m.room.redaction
    RoomRedaction(RedactionEvent),

    /// m.room.server_acl
    RoomServerAcl(ServerAclEvent),

    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),

    /// m.room.tombstone
    RoomTombstone(TombstoneEvent),

    /// m.room.topic
    RoomTopic(TopicEvent),

    /// m.room_key
    RoomKey(RoomKeyEvent),

    /// m.room_key_request
    RoomKeyRequest(RoomKeyRequestEvent),

    /// m.sticker
    Sticker(StickerEvent),

    /// m.tag
    Tag(TagEvent),

    /// m.typing
    Typing(TypingEvent),

    /// Any basic event that is not part of the specification.
    Custom(CustomEvent),

    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),

    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
}

/// A room event or state event.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum RoomEvent {
    /// m.call.answer
    CallAnswer(AnswerEvent),

    /// m.call.candidates
    CallCandidates(CandidatesEvent),

    /// m.call.hangup
    CallHangup(HangupEvent),

    /// m.call.invite
    CallInvite(InviteEvent),

    /// m.room.aliases
    RoomAliases(AliasesEvent),

    /// m.room.avatar
    RoomAvatar(AvatarEvent),

    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),

    /// m.room.create
    RoomCreate(CreateEvent),

    /// m.room.encrypted
    RoomEncrypted(EncryptedEvent),

    /// m.room.encryption
    RoomEncryption(EncryptionEvent),

    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),

    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),

    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),

    /// m.room.member
    RoomMember(MemberEvent),

    /// m.room.message
    RoomMessage(MessageEvent),

    /// m.room.message.feedback
    RoomMessageFeedback(FeedbackEvent),

    /// m.room.name
    RoomName(NameEvent),

    /// m.room.pinned_events
    RoomPinnedEvents(PinnedEventsEvent),

    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),

    /// m.room.redaction
    RoomRedaction(RedactionEvent),

    /// m.room.server_acl
    RoomServerAcl(ServerAclEvent),

    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),

    /// m.room.tombstone
    RoomTombstone(TombstoneEvent),

    /// m.room.topic
    RoomTopic(TopicEvent),

    /// m.sticker
    Sticker(StickerEvent),

    /// Any room event that is not part of the specification.
    CustomRoom(CustomRoomEvent),

    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
}

/// A state event.
#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum StateEvent {
    /// m.room.aliases
    RoomAliases(AliasesEvent),

    /// m.room.avatar
    RoomAvatar(AvatarEvent),

    /// m.room.canonical_alias
    RoomCanonicalAlias(CanonicalAliasEvent),

    /// m.room.create
    RoomCreate(CreateEvent),

    /// m.room.encryption
    RoomEncryption(EncryptionEvent),

    /// m.room.guest_access
    RoomGuestAccess(GuestAccessEvent),

    /// m.room.history_visibility
    RoomHistoryVisibility(HistoryVisibilityEvent),

    /// m.room.join_rules
    RoomJoinRules(JoinRulesEvent),

    /// m.room.member
    RoomMember(MemberEvent),

    /// m.room.name
    RoomName(NameEvent),

    /// m.room.pinned_events
    RoomPinnedEvents(PinnedEventsEvent),

    /// m.room.power_levels
    RoomPowerLevels(PowerLevelsEvent),

    /// m.room.server_acl
    RoomServerAcl(ServerAclEvent),

    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),

    /// m.room.tombstone
    RoomTombstone(TombstoneEvent),

    /// m.room.topic
    RoomTopic(TopicEvent),

    /// Any state event that is not part of the specification.
    CustomState(CustomStateEvent),
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::util::try_variant_from_value as from_value;
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_field(&value, "type")?;

        match event_type {
            CallAnswer => from_value(value, Event::CallAnswer),
            CallCandidates => from_value(value, Event::CallCandidates),
            CallHangup => from_value(value, Event::CallHangup),
            CallInvite => from_value(value, Event::CallInvite),
            Direct => from_value(value, Event::Direct),
            Dummy => from_value(value, Event::Dummy),
            ForwardedRoomKey => from_value(value, Event::ForwardedRoomKey),
            FullyRead => from_value(value, Event::FullyRead),
            IgnoredUserList => from_value(value, Event::IgnoredUserList),
            KeyVerificationAccept => from_value(value, Event::KeyVerificationAccept),
            KeyVerificationCancel => from_value(value, Event::KeyVerificationCancel),
            KeyVerificationKey => from_value(value, Event::KeyVerificationKey),
            KeyVerificationMac => from_value(value, Event::KeyVerificationMac),
            KeyVerificationRequest => from_value(value, Event::KeyVerificationRequest),
            KeyVerificationStart => from_value(value, Event::KeyVerificationStart),
            Presence => from_value(value, Event::Presence),
            PushRules => from_value(value, Event::PushRules),
            Receipt => from_value(value, Event::Receipt),
            RoomAliases => from_value(value, Event::RoomAliases),
            RoomAvatar => from_value(value, Event::RoomAvatar),
            RoomCanonicalAlias => from_value(value, Event::RoomCanonicalAlias),
            RoomCreate => from_value(value, Event::RoomCreate),
            RoomEncrypted => from_value(value, Event::RoomEncrypted),
            RoomEncryption => from_value(value, Event::RoomEncryption),
            RoomGuestAccess => from_value(value, Event::RoomGuestAccess),
            RoomHistoryVisibility => from_value(value, Event::RoomHistoryVisibility),
            RoomJoinRules => from_value(value, Event::RoomJoinRules),
            RoomMember => from_value(value, Event::RoomMember),
            RoomMessage => from_value(value, Event::RoomMessage),
            RoomMessageFeedback => from_value(value, Event::RoomMessageFeedback),
            RoomName => from_value(value, Event::RoomName),
            RoomPinnedEvents => from_value(value, Event::RoomPinnedEvents),
            RoomPowerLevels => from_value(value, Event::RoomPowerLevels),
            RoomRedaction => from_value(value, Event::RoomRedaction),
            RoomServerAcl => from_value(value, Event::RoomServerAcl),
            RoomThirdPartyInvite => from_value(value, Event::RoomThirdPartyInvite),
            RoomTombstone => from_value(value, Event::RoomTombstone),
            RoomTopic => from_value(value, Event::RoomTopic),
            RoomKey => from_value(value, Event::RoomKey),
            RoomKeyRequest => from_value(value, Event::RoomKeyRequest),
            Sticker => from_value(value, Event::Sticker),
            Tag => from_value(value, Event::Tag),
            Typing => from_value(value, Event::Typing),
            Custom(_event_type_name) => {
                if value.get("state_key").is_some() {
                    from_value(value, Event::CustomState)
                } else if value.get("event_id").is_some()
                    && value.get("room_id").is_some()
                    && value.get("sender").is_some()
                {
                    from_value(value, Event::CustomRoom)
                } else {
                    from_value(value, Event::Custom)
                }
            }
            __Nonexhaustive => {
                unreachable!("__Nonexhaustive variant should be impossible to obtain.")
            }
        }
    }
}

impl<'de> Deserialize<'de> for RoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::util::try_variant_from_value as from_value;
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_field(&value, "type")?;

        match event_type {
            CallAnswer => from_value(value, RoomEvent::CallAnswer),
            CallCandidates => from_value(value, RoomEvent::CallCandidates),
            CallHangup => from_value(value, RoomEvent::CallHangup),
            CallInvite => from_value(value, RoomEvent::CallInvite),
            RoomAliases => from_value(value, RoomEvent::RoomAliases),
            RoomAvatar => from_value(value, RoomEvent::RoomAvatar),
            RoomCanonicalAlias => from_value(value, RoomEvent::RoomCanonicalAlias),
            RoomCreate => from_value(value, RoomEvent::RoomCreate),
            RoomEncrypted => from_value(value, RoomEvent::RoomEncrypted),
            RoomEncryption => from_value(value, RoomEvent::RoomEncryption),
            RoomGuestAccess => from_value(value, RoomEvent::RoomGuestAccess),
            RoomHistoryVisibility => from_value(value, RoomEvent::RoomHistoryVisibility),
            RoomJoinRules => from_value(value, RoomEvent::RoomJoinRules),
            RoomMember => from_value(value, RoomEvent::RoomMember),
            RoomMessage => from_value(value, RoomEvent::RoomMessage),
            RoomMessageFeedback => from_value(value, RoomEvent::RoomMessageFeedback),
            RoomName => from_value(value, RoomEvent::RoomName),
            RoomPinnedEvents => from_value(value, RoomEvent::RoomPinnedEvents),
            RoomPowerLevels => from_value(value, RoomEvent::RoomPowerLevels),
            RoomRedaction => from_value(value, RoomEvent::RoomRedaction),
            RoomServerAcl => from_value(value, RoomEvent::RoomServerAcl),
            RoomThirdPartyInvite => from_value(value, RoomEvent::RoomThirdPartyInvite),
            RoomTombstone => from_value(value, RoomEvent::RoomTombstone),
            RoomTopic => from_value(value, RoomEvent::RoomTopic),
            Sticker => from_value(value, RoomEvent::Sticker),
            Custom(_event_type_name) => {
                if value.get("state_key").is_some() {
                    from_value(value, RoomEvent::CustomState)
                } else {
                    from_value(value, RoomEvent::CustomRoom)
                }
            }
            Direct
            | Dummy
            | ForwardedRoomKey
            | FullyRead
            | IgnoredUserList
            | KeyVerificationAccept
            | KeyVerificationCancel
            | KeyVerificationKey
            | KeyVerificationMac
            | KeyVerificationRequest
            | KeyVerificationStart
            | Presence
            | PushRules
            | Receipt
            | RoomKey
            | RoomKeyRequest
            | Tag
            | Typing => Err(D::Error::custom("invalid event type")),
            __Nonexhaustive => {
                unreachable!("__Nonexhaustive variant should be impossible to obtain.")
            }
        }
    }
}

impl<'de> Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::util::try_variant_from_value as from_value;
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_field(&value, "type")?;

        match event_type {
            RoomAliases => from_value(value, StateEvent::RoomAliases),
            RoomAvatar => from_value(value, StateEvent::RoomAvatar),
            RoomCanonicalAlias => from_value(value, StateEvent::RoomCanonicalAlias),
            RoomCreate => from_value(value, StateEvent::RoomCreate),
            RoomEncryption => from_value(value, StateEvent::RoomEncryption),
            RoomGuestAccess => from_value(value, StateEvent::RoomGuestAccess),
            RoomHistoryVisibility => from_value(value, StateEvent::RoomHistoryVisibility),
            RoomJoinRules => from_value(value, StateEvent::RoomJoinRules),
            RoomMember => from_value(value, StateEvent::RoomMember),
            RoomName => from_value(value, StateEvent::RoomName),
            RoomPinnedEvents => from_value(value, StateEvent::RoomPinnedEvents),
            RoomPowerLevels => from_value(value, StateEvent::RoomPowerLevels),
            RoomServerAcl => from_value(value, StateEvent::RoomServerAcl),
            RoomThirdPartyInvite => from_value(value, StateEvent::RoomThirdPartyInvite),
            RoomTombstone => from_value(value, StateEvent::RoomTombstone),
            RoomTopic => from_value(value, StateEvent::RoomTopic),
            Custom(_event_type_name) => from_value(value, StateEvent::CustomState),
            CallAnswer
            | CallCandidates
            | CallHangup
            | CallInvite
            | Direct
            | Dummy
            | ForwardedRoomKey
            | FullyRead
            | IgnoredUserList
            | KeyVerificationAccept
            | KeyVerificationCancel
            | KeyVerificationKey
            | KeyVerificationMac
            | KeyVerificationRequest
            | KeyVerificationStart
            | Presence
            | PushRules
            | Receipt
            | RoomEncrypted
            | RoomKey
            | RoomKeyRequest
            | RoomMessage
            | RoomMessageFeedback
            | RoomRedaction
            | Sticker
            | Tag
            | Typing => Err(D::Error::custom("invalid event type")),
            __Nonexhaustive => {
                unreachable!("__Nonexhaustive variant should be impossible to obtain.")
            }
        }
    }
}
