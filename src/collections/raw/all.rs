//! Enums for heterogeneous collections of events, inclusive for every event type that implements
//! the trait of the same name.

use serde::{de::Error as _, Deserialize, Deserializer};
use serde_json::{from_value, Value};

use crate::{
    call::{
        answer::raw::AnswerEvent, candidates::raw::CandidatesEvent, hangup::raw::HangupEvent,
        invite::raw::InviteEvent,
    },
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
    util::{get_type_field, serde_json_error_to_generic_de_error as conv_err},
    CustomEvent, CustomRoomEvent, CustomStateEvent, EventType,
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
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_type_field(&value)?;

        match event_type {
            CallAnswer => from_value(value).map(Self::CallAnswer).map_err(conv_err),
            CallCandidates => from_value(value)
                .map(Self::CallCandidates)
                .map_err(conv_err),
            CallHangup => from_value(value).map(Self::CallHangup).map_err(conv_err),
            CallInvite => from_value(value).map(Self::CallInvite).map_err(conv_err),
            Direct => from_value(value).map(Self::Direct).map_err(conv_err),
            Dummy => from_value(value).map(Self::Dummy).map_err(conv_err),
            ForwardedRoomKey => from_value(value)
                .map(Self::ForwardedRoomKey)
                .map_err(conv_err),
            FullyRead => from_value(value).map(Self::FullyRead).map_err(conv_err),
            IgnoredUserList => from_value(value)
                .map(Self::IgnoredUserList)
                .map_err(conv_err),
            KeyVerificationAccept => from_value(value)
                .map(Self::KeyVerificationAccept)
                .map_err(conv_err),
            KeyVerificationCancel => from_value(value)
                .map(Self::KeyVerificationCancel)
                .map_err(conv_err),
            KeyVerificationKey => from_value(value)
                .map(Self::KeyVerificationKey)
                .map_err(conv_err),
            KeyVerificationMac => from_value(value)
                .map(Self::KeyVerificationMac)
                .map_err(conv_err),
            KeyVerificationRequest => from_value(value)
                .map(Self::KeyVerificationRequest)
                .map_err(conv_err),
            KeyVerificationStart => from_value(value)
                .map(Self::KeyVerificationStart)
                .map_err(conv_err),
            Presence => from_value(value).map(Self::Presence).map_err(conv_err),
            PushRules => from_value(value).map(Self::PushRules).map_err(conv_err),
            Receipt => from_value(value).map(Self::Receipt).map_err(conv_err),
            RoomAliases => from_value(value).map(Self::RoomAliases).map_err(conv_err),
            RoomAvatar => from_value(value).map(Self::RoomAvatar).map_err(conv_err),
            RoomCanonicalAlias => from_value(value)
                .map(Self::RoomCanonicalAlias)
                .map_err(conv_err),
            RoomCreate => from_value(value).map(Self::RoomCreate).map_err(conv_err),
            RoomEncrypted => from_value(value).map(Self::RoomEncrypted).map_err(conv_err),
            RoomEncryption => from_value(value)
                .map(Self::RoomEncryption)
                .map_err(conv_err),
            RoomGuestAccess => from_value(value)
                .map(Self::RoomGuestAccess)
                .map_err(conv_err),
            RoomHistoryVisibility => from_value(value)
                .map(Self::RoomHistoryVisibility)
                .map_err(conv_err),
            RoomJoinRules => from_value(value).map(Self::RoomJoinRules).map_err(conv_err),
            RoomMember => from_value(value).map(Self::RoomMember).map_err(conv_err),
            RoomMessage => from_value(value).map(Self::RoomMessage).map_err(conv_err),
            RoomMessageFeedback => from_value(value)
                .map(Self::RoomMessageFeedback)
                .map_err(conv_err),
            RoomName => from_value(value).map(Self::RoomName).map_err(conv_err),
            RoomPinnedEvents => from_value(value)
                .map(Self::RoomPinnedEvents)
                .map_err(conv_err),
            RoomPowerLevels => from_value(value)
                .map(Self::RoomPowerLevels)
                .map_err(conv_err),
            RoomRedaction => from_value(value).map(Self::RoomRedaction).map_err(conv_err),
            RoomServerAcl => from_value(value).map(Self::RoomServerAcl).map_err(conv_err),
            RoomThirdPartyInvite => from_value(value)
                .map(Self::RoomThirdPartyInvite)
                .map_err(conv_err),
            RoomTombstone => from_value(value).map(Self::RoomTombstone).map_err(conv_err),
            RoomTopic => from_value(value).map(Self::RoomTopic).map_err(conv_err),
            RoomKey => from_value(value).map(Self::RoomKey).map_err(conv_err),
            RoomKeyRequest => from_value(value)
                .map(Self::RoomKeyRequest)
                .map_err(conv_err),
            Sticker => from_value(value).map(Self::Sticker).map_err(conv_err),
            Tag => from_value(value).map(Self::Tag).map_err(conv_err),
            Typing => from_value(value).map(Self::Typing).map_err(conv_err),
            // TODO
            Custom(_event_type_name) => Err(D::Error::custom("invalid event type")),
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
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_type_field(&value)?;

        match event_type {
            CallAnswer => from_value(value).map(Self::CallAnswer).map_err(conv_err),
            CallCandidates => from_value(value)
                .map(Self::CallCandidates)
                .map_err(conv_err),
            CallHangup => from_value(value).map(Self::CallHangup).map_err(conv_err),
            CallInvite => from_value(value).map(Self::CallInvite).map_err(conv_err),
            RoomAliases => from_value(value).map(Self::RoomAliases).map_err(conv_err),
            RoomAvatar => from_value(value).map(Self::RoomAvatar).map_err(conv_err),
            RoomCanonicalAlias => from_value(value)
                .map(Self::RoomCanonicalAlias)
                .map_err(conv_err),
            RoomCreate => from_value(value).map(Self::RoomCreate).map_err(conv_err),
            RoomEncrypted => from_value(value).map(Self::RoomEncrypted).map_err(conv_err),
            RoomEncryption => from_value(value)
                .map(Self::RoomEncryption)
                .map_err(conv_err),
            RoomGuestAccess => from_value(value)
                .map(Self::RoomGuestAccess)
                .map_err(conv_err),
            RoomHistoryVisibility => from_value(value)
                .map(Self::RoomHistoryVisibility)
                .map_err(conv_err),
            RoomJoinRules => from_value(value).map(Self::RoomJoinRules).map_err(conv_err),
            RoomMember => from_value(value).map(Self::RoomMember).map_err(conv_err),
            RoomMessage => from_value(value).map(Self::RoomMessage).map_err(conv_err),
            RoomMessageFeedback => from_value(value)
                .map(Self::RoomMessageFeedback)
                .map_err(conv_err),
            RoomName => from_value(value).map(Self::RoomName).map_err(conv_err),
            RoomPinnedEvents => from_value(value)
                .map(Self::RoomPinnedEvents)
                .map_err(conv_err),
            RoomPowerLevels => from_value(value)
                .map(Self::RoomPowerLevels)
                .map_err(conv_err),
            RoomRedaction => from_value(value).map(Self::RoomRedaction).map_err(conv_err),
            RoomServerAcl => from_value(value).map(Self::RoomServerAcl).map_err(conv_err),
            RoomThirdPartyInvite => from_value(value)
                .map(Self::RoomThirdPartyInvite)
                .map_err(conv_err),
            RoomTombstone => from_value(value).map(Self::RoomTombstone).map_err(conv_err),
            RoomTopic => from_value(value).map(Self::RoomTopic).map_err(conv_err),
            Sticker => from_value(value).map(Self::Sticker).map_err(conv_err),
            //Custom(_event_type_name) => unimplemented!("todo"),
            _ => Err(D::Error::custom("invalid event type")),
        }
    }
}

impl<'de> Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use EventType::*;

        let value = Value::deserialize(deserializer)?;
        let event_type = get_type_field(&value)?;

        match event_type {
            RoomAliases => from_value(value).map(Self::RoomAliases).map_err(conv_err),
            RoomAvatar => from_value(value).map(Self::RoomAvatar).map_err(conv_err),
            RoomCanonicalAlias => from_value(value)
                .map(Self::RoomCanonicalAlias)
                .map_err(conv_err),
            RoomCreate => from_value(value).map(Self::RoomCreate).map_err(conv_err),
            RoomEncryption => from_value(value)
                .map(Self::RoomEncryption)
                .map_err(conv_err),
            RoomGuestAccess => from_value(value)
                .map(Self::RoomGuestAccess)
                .map_err(conv_err),
            RoomHistoryVisibility => from_value(value)
                .map(Self::RoomHistoryVisibility)
                .map_err(conv_err),
            RoomJoinRules => from_value(value).map(Self::RoomJoinRules).map_err(conv_err),
            RoomMember => from_value(value).map(Self::RoomMember).map_err(conv_err),
            RoomName => from_value(value).map(Self::RoomName).map_err(conv_err),
            RoomPinnedEvents => from_value(value)
                .map(Self::RoomPinnedEvents)
                .map_err(conv_err),
            RoomPowerLevels => from_value(value)
                .map(Self::RoomPowerLevels)
                .map_err(conv_err),
            RoomServerAcl => from_value(value).map(Self::RoomServerAcl).map_err(conv_err),
            RoomThirdPartyInvite => from_value(value)
                .map(Self::RoomThirdPartyInvite)
                .map_err(conv_err),
            RoomTombstone => from_value(value).map(Self::RoomTombstone).map_err(conv_err),
            RoomTopic => from_value(value).map(Self::RoomTopic).map_err(conv_err),
            //Custom(_event_type_name) => unimplemented!("todo"),
            _ => Err(D::Error::custom("invalid event type")),
        }
    }
}

/*impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Event::CallAnswer(ref event) => event.serialize(serializer),
            Event::CallCandidates(ref event) => event.serialize(serializer),
            Event::CallHangup(ref event) => event.serialize(serializer),
            Event::CallInvite(ref event) => event.serialize(serializer),
            Event::Direct(ref event) => event.serialize(serializer),
            Event::Dummy(ref event) => event.serialize(serializer),
            Event::ForwardedRoomKey(ref event) => event.serialize(serializer),
            Event::FullyRead(ref event) => event.serialize(serializer),
            Event::KeyVerificationAccept(ref event) => event.serialize(serializer),
            Event::KeyVerificationCancel(ref event) => event.serialize(serializer),
            Event::KeyVerificationKey(ref event) => event.serialize(serializer),
            Event::KeyVerificationMac(ref event) => event.serialize(serializer),
            Event::KeyVerificationRequest(ref event) => event.serialize(serializer),
            Event::KeyVerificationStart(ref event) => event.serialize(serializer),
            Event::IgnoredUserList(ref event) => event.serialize(serializer),
            Event::Presence(ref event) => event.serialize(serializer),
            Event::PushRules(ref event) => event.serialize(serializer),
            Event::Receipt(ref event) => event.serialize(serializer),
            Event::RoomAliases(ref event) => event.serialize(serializer),
            Event::RoomAvatar(ref event) => event.serialize(serializer),
            Event::RoomCanonicalAlias(ref event) => event.serialize(serializer),
            Event::RoomCreate(ref event) => event.serialize(serializer),
            Event::RoomEncrypted(ref event) => event.serialize(serializer),
            Event::RoomEncryption(ref event) => event.serialize(serializer),
            Event::RoomGuestAccess(ref event) => event.serialize(serializer),
            Event::RoomHistoryVisibility(ref event) => event.serialize(serializer),
            Event::RoomJoinRules(ref event) => event.serialize(serializer),
            Event::RoomMember(ref event) => event.serialize(serializer),
            Event::RoomMessage(ref event) => event.serialize(serializer),
            Event::RoomMessageFeedback(ref event) => event.serialize(serializer),
            Event::RoomName(ref event) => event.serialize(serializer),
            Event::RoomPinnedEvents(ref event) => event.serialize(serializer),
            Event::RoomPowerLevels(ref event) => event.serialize(serializer),
            Event::RoomRedaction(ref event) => event.serialize(serializer),
            Event::RoomServerAcl(ref event) => event.serialize(serializer),
            Event::RoomThirdPartyInvite(ref event) => event.serialize(serializer),
            Event::RoomTombstone(ref event) => event.serialize(serializer),
            Event::RoomTopic(ref event) => event.serialize(serializer),
            Event::RoomKey(ref event) => event.serialize(serializer),
            Event::RoomKeyRequest(ref event) => event.serialize(serializer),
            Event::Sticker(ref event) => event.serialize(serializer),
            Event::Tag(ref event) => event.serialize(serializer),
            Event::Typing(ref event) => event.serialize(serializer),
            Event::Custom(ref event) => event.serialize(serializer),
            Event::CustomRoom(ref event) => event.serialize(serializer),
            Event::CustomState(ref event) => event.serialize(serializer),
        }
    }
}

impl Serialize for RoomEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            RoomEvent::CallAnswer(ref event) => event.serialize(serializer),
            RoomEvent::CallCandidates(ref event) => event.serialize(serializer),
            RoomEvent::CallHangup(ref event) => event.serialize(serializer),
            RoomEvent::CallInvite(ref event) => event.serialize(serializer),
            RoomEvent::RoomAliases(ref event) => event.serialize(serializer),
            RoomEvent::RoomAvatar(ref event) => event.serialize(serializer),
            RoomEvent::RoomCanonicalAlias(ref event) => event.serialize(serializer),
            RoomEvent::RoomCreate(ref event) => event.serialize(serializer),
            RoomEvent::RoomEncrypted(ref event) => event.serialize(serializer),
            RoomEvent::RoomEncryption(ref event) => event.serialize(serializer),
            RoomEvent::RoomGuestAccess(ref event) => event.serialize(serializer),
            RoomEvent::RoomHistoryVisibility(ref event) => event.serialize(serializer),
            RoomEvent::RoomJoinRules(ref event) => event.serialize(serializer),
            RoomEvent::RoomMember(ref event) => event.serialize(serializer),
            RoomEvent::RoomMessage(ref event) => event.serialize(serializer),
            RoomEvent::RoomMessageFeedback(ref event) => event.serialize(serializer),
            RoomEvent::RoomName(ref event) => event.serialize(serializer),
            RoomEvent::RoomPinnedEvents(ref event) => event.serialize(serializer),
            RoomEvent::RoomPowerLevels(ref event) => event.serialize(serializer),
            RoomEvent::RoomRedaction(ref event) => event.serialize(serializer),
            RoomEvent::RoomServerAcl(ref event) => event.serialize(serializer),
            RoomEvent::RoomThirdPartyInvite(ref event) => event.serialize(serializer),
            RoomEvent::RoomTombstone(ref event) => event.serialize(serializer),
            RoomEvent::RoomTopic(ref event) => event.serialize(serializer),
            RoomEvent::Sticker(ref event) => event.serialize(serializer),
            RoomEvent::CustomRoom(ref event) => event.serialize(serializer),
            RoomEvent::CustomState(ref event) => event.serialize(serializer),
        }
    }
}

impl Serialize for StateEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            StateEvent::RoomAliases(ref event) => event.serialize(serializer),
            StateEvent::RoomAvatar(ref event) => event.serialize(serializer),
            StateEvent::RoomCanonicalAlias(ref event) => event.serialize(serializer),
            StateEvent::RoomCreate(ref event) => event.serialize(serializer),
            StateEvent::RoomEncryption(ref event) => event.serialize(serializer),
            StateEvent::RoomGuestAccess(ref event) => event.serialize(serializer),
            StateEvent::RoomHistoryVisibility(ref event) => event.serialize(serializer),
            StateEvent::RoomJoinRules(ref event) => event.serialize(serializer),
            StateEvent::RoomMember(ref event) => event.serialize(serializer),
            StateEvent::RoomName(ref event) => event.serialize(serializer),
            StateEvent::RoomPinnedEvents(ref event) => event.serialize(serializer),
            StateEvent::RoomPowerLevels(ref event) => event.serialize(serializer),
            StateEvent::RoomServerAcl(ref event) => event.serialize(serializer),
            StateEvent::RoomThirdPartyInvite(ref event) => event.serialize(serializer),
            StateEvent::RoomTombstone(ref event) => event.serialize(serializer),
            StateEvent::RoomTopic(ref event) => event.serialize(serializer),
            StateEvent::CustomState(ref event) => event.serialize(serializer),
        }
    }
}*/

macro_rules! impl_from_t_for_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for Event {
            fn from(event: $ty) -> Self {
                Event::$variant(event)
            }
        }
    };
}

impl_from_t_for_event!(AnswerEvent, CallAnswer);
impl_from_t_for_event!(CandidatesEvent, CallCandidates);
impl_from_t_for_event!(HangupEvent, CallHangup);
impl_from_t_for_event!(InviteEvent, CallInvite);
impl_from_t_for_event!(DirectEvent, Direct);
impl_from_t_for_event!(DummyEvent, Dummy);
impl_from_t_for_event!(ForwardedRoomKeyEvent, ForwardedRoomKey);
impl_from_t_for_event!(FullyReadEvent, FullyRead);
impl_from_t_for_event!(AcceptEvent, KeyVerificationAccept);
impl_from_t_for_event!(CancelEvent, KeyVerificationCancel);
impl_from_t_for_event!(KeyEvent, KeyVerificationKey);
impl_from_t_for_event!(MacEvent, KeyVerificationMac);
impl_from_t_for_event!(RequestEvent, KeyVerificationRequest);
impl_from_t_for_event!(StartEvent, KeyVerificationStart);
impl_from_t_for_event!(IgnoredUserListEvent, IgnoredUserList);
impl_from_t_for_event!(PresenceEvent, Presence);
impl_from_t_for_event!(PushRulesEvent, PushRules);
impl_from_t_for_event!(ReceiptEvent, Receipt);
impl_from_t_for_event!(AliasesEvent, RoomAliases);
impl_from_t_for_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_event!(CreateEvent, RoomCreate);
impl_from_t_for_event!(EncryptedEvent, RoomEncrypted);
impl_from_t_for_event!(EncryptionEvent, RoomEncryption);
impl_from_t_for_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_event!(MemberEvent, RoomMember);
impl_from_t_for_event!(MessageEvent, RoomMessage);
impl_from_t_for_event!(FeedbackEvent, RoomMessageFeedback);
impl_from_t_for_event!(NameEvent, RoomName);
impl_from_t_for_event!(PinnedEventsEvent, RoomPinnedEvents);
impl_from_t_for_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_event!(ServerAclEvent, RoomServerAcl);
impl_from_t_for_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_event!(TombstoneEvent, RoomTombstone);
impl_from_t_for_event!(TopicEvent, RoomTopic);
impl_from_t_for_event!(RoomKeyEvent, RoomKey);
impl_from_t_for_event!(RoomKeyRequestEvent, RoomKeyRequest);
impl_from_t_for_event!(StickerEvent, Sticker);
impl_from_t_for_event!(TagEvent, Tag);
impl_from_t_for_event!(TypingEvent, Typing);
impl_from_t_for_event!(CustomEvent, Custom);
impl_from_t_for_event!(CustomRoomEvent, CustomRoom);
impl_from_t_for_event!(CustomStateEvent, CustomState);

macro_rules! impl_from_t_for_room_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for RoomEvent {
            fn from(event: $ty) -> Self {
                RoomEvent::$variant(event)
            }
        }
    };
}

impl_from_t_for_room_event!(AnswerEvent, CallAnswer);
impl_from_t_for_room_event!(CandidatesEvent, CallCandidates);
impl_from_t_for_room_event!(HangupEvent, CallHangup);
impl_from_t_for_room_event!(InviteEvent, CallInvite);
impl_from_t_for_room_event!(AliasesEvent, RoomAliases);
impl_from_t_for_room_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_room_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_room_event!(CreateEvent, RoomCreate);
impl_from_t_for_room_event!(EncryptedEvent, RoomEncrypted);
impl_from_t_for_room_event!(EncryptionEvent, RoomEncryption);
impl_from_t_for_room_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_room_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_room_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_room_event!(MemberEvent, RoomMember);
impl_from_t_for_room_event!(MessageEvent, RoomMessage);
impl_from_t_for_room_event!(FeedbackEvent, RoomMessageFeedback);
impl_from_t_for_room_event!(NameEvent, RoomName);
impl_from_t_for_room_event!(PinnedEventsEvent, RoomPinnedEvents);
impl_from_t_for_room_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_room_event!(RedactionEvent, RoomRedaction);
impl_from_t_for_room_event!(ServerAclEvent, RoomServerAcl);
impl_from_t_for_room_event!(StickerEvent, Sticker);
impl_from_t_for_room_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_room_event!(TombstoneEvent, RoomTombstone);
impl_from_t_for_room_event!(TopicEvent, RoomTopic);
impl_from_t_for_room_event!(CustomRoomEvent, CustomRoom);
impl_from_t_for_room_event!(CustomStateEvent, CustomState);

macro_rules! impl_from_t_for_state_event {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for StateEvent {
            fn from(event: $ty) -> Self {
                StateEvent::$variant(event)
            }
        }
    };
}

impl_from_t_for_state_event!(AliasesEvent, RoomAliases);
impl_from_t_for_state_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_state_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_state_event!(CreateEvent, RoomCreate);
impl_from_t_for_state_event!(EncryptionEvent, RoomEncryption);
impl_from_t_for_state_event!(GuestAccessEvent, RoomGuestAccess);
impl_from_t_for_state_event!(HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_t_for_state_event!(JoinRulesEvent, RoomJoinRules);
impl_from_t_for_state_event!(MemberEvent, RoomMember);
impl_from_t_for_state_event!(NameEvent, RoomName);
impl_from_t_for_state_event!(PinnedEventsEvent, RoomPinnedEvents);
impl_from_t_for_state_event!(PowerLevelsEvent, RoomPowerLevels);
impl_from_t_for_state_event!(ServerAclEvent, RoomServerAcl);
impl_from_t_for_state_event!(ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_t_for_state_event!(TombstoneEvent, RoomTombstone);
impl_from_t_for_state_event!(TopicEvent, RoomTopic);
impl_from_t_for_state_event!(CustomStateEvent, CustomState);
