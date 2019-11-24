//! Enums for heterogeneous collections of events, inclusive for every event type that implements
//! the trait of the same name.

use serde::Serialize;

use super::raw::all as raw;
use crate::{
    call::{
        answer::AnswerEvent, candidates::CandidatesEvent, hangup::HangupEvent, invite::InviteEvent,
    },
    direct::DirectEvent,
    dummy::DummyEvent,
    forwarded_room_key::ForwardedRoomKeyEvent,
    fully_read::FullyReadEvent,
    ignored_user_list::IgnoredUserListEvent,
    key::verification::{
        accept::AcceptEvent, cancel::CancelEvent, key::KeyEvent, mac::MacEvent,
        request::RequestEvent, start::StartEvent,
    },
    presence::PresenceEvent,
    push_rules::PushRulesEvent,
    receipt::ReceiptEvent,
    room::{
        aliases::AliasesEvent,
        avatar::AvatarEvent,
        canonical_alias::CanonicalAliasEvent,
        create::CreateEvent,
        encrypted::EncryptedEvent,
        encryption::EncryptionEvent,
        guest_access::GuestAccessEvent,
        history_visibility::HistoryVisibilityEvent,
        join_rules::JoinRulesEvent,
        member::MemberEvent,
        message::{feedback::FeedbackEvent, MessageEvent},
        name::NameEvent,
        pinned_events::PinnedEventsEvent,
        power_levels::PowerLevelsEvent,
        redaction::RedactionEvent,
        server_acl::ServerAclEvent,
        third_party_invite::ThirdPartyInviteEvent,
        tombstone::TombstoneEvent,
        topic::TopicEvent,
    },
    room_key::RoomKeyEvent,
    room_key_request::RoomKeyRequestEvent,
    sticker::StickerEvent,
    tag::TagEvent,
    typing::TypingEvent,
    CustomEvent, CustomRoomEvent, CustomStateEvent, TryFromRaw,
};

/// A basic event, room event, or state event.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
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

impl TryFromRaw for Event {
    type Raw = raw::Event;
    type Err = String;

    fn try_from_raw(raw: raw::Event) -> Result<Self, Self::Err> {
        use crate::util::try_convert_variant as conv;
        use raw::Event::*;

        match raw {
            CallAnswer(c) => conv(Event::CallAnswer, c),
            CallCandidates(c) => conv(Event::CallCandidates, c),
            CallHangup(c) => conv(Event::CallHangup, c),
            CallInvite(c) => conv(Event::CallInvite, c),
            Direct(c) => conv(Event::Direct, c),
            Dummy(c) => conv(Event::Dummy, c),
            ForwardedRoomKey(c) => conv(Event::ForwardedRoomKey, c),
            FullyRead(c) => conv(Event::FullyRead, c),
            IgnoredUserList(c) => conv(Event::IgnoredUserList, c),
            KeyVerificationAccept(c) => conv(Event::KeyVerificationAccept, c),
            KeyVerificationCancel(c) => conv(Event::KeyVerificationCancel, c),
            KeyVerificationKey(c) => conv(Event::KeyVerificationKey, c),
            KeyVerificationMac(c) => conv(Event::KeyVerificationMac, c),
            KeyVerificationRequest(c) => conv(Event::KeyVerificationRequest, c),
            KeyVerificationStart(c) => conv(Event::KeyVerificationStart, c),
            Presence(c) => conv(Event::Presence, c),
            PushRules(c) => conv(Event::PushRules, c),
            Receipt(c) => conv(Event::Receipt, c),
            RoomAliases(c) => conv(Event::RoomAliases, c),
            RoomAvatar(c) => conv(Event::RoomAvatar, c),
            RoomCanonicalAlias(c) => conv(Event::RoomCanonicalAlias, c),
            RoomCreate(c) => conv(Event::RoomCreate, c),
            RoomEncrypted(c) => conv(Event::RoomEncrypted, c),
            RoomEncryption(c) => conv(Event::RoomEncryption, c),
            RoomGuestAccess(c) => conv(Event::RoomGuestAccess, c),
            RoomHistoryVisibility(c) => conv(Event::RoomHistoryVisibility, c),
            RoomJoinRules(c) => conv(Event::RoomJoinRules, c),
            RoomMember(c) => conv(Event::RoomMember, c),
            RoomMessage(c) => conv(Event::RoomMessage, c),
            RoomMessageFeedback(c) => conv(Event::RoomMessageFeedback, c),
            RoomName(c) => conv(Event::RoomName, c),
            RoomPinnedEvents(c) => conv(Event::RoomPinnedEvents, c),
            RoomPowerLevels(c) => conv(Event::RoomPowerLevels, c),
            RoomRedaction(c) => conv(Event::RoomRedaction, c),
            RoomServerAcl(c) => conv(Event::RoomServerAcl, c),
            RoomThirdPartyInvite(c) => conv(Event::RoomThirdPartyInvite, c),
            RoomTombstone(c) => conv(Event::RoomTombstone, c),
            RoomTopic(c) => conv(Event::RoomTopic, c),
            RoomKey(c) => conv(Event::RoomKey, c),
            RoomKeyRequest(c) => conv(Event::RoomKeyRequest, c),
            Sticker(c) => conv(Event::Sticker, c),
            Tag(c) => conv(Event::Tag, c),
            Typing(c) => conv(Event::Typing, c),
            Custom(c) => conv(Event::Custom, c),
            CustomRoom(c) => conv(Event::CustomRoom, c),
            CustomState(c) => conv(Event::CustomState, c),
        }
    }
}

impl TryFromRaw for RoomEvent {
    type Raw = raw::RoomEvent;
    type Err = String;

    fn try_from_raw(raw: raw::RoomEvent) -> Result<Self, Self::Err> {
        use crate::util::try_convert_variant as conv;
        use raw::RoomEvent::*;

        match raw {
            CallAnswer(c) => conv(RoomEvent::CallAnswer, c),
            CallCandidates(c) => conv(RoomEvent::CallCandidates, c),
            CallHangup(c) => conv(RoomEvent::CallHangup, c),
            CallInvite(c) => conv(RoomEvent::CallInvite, c),
            RoomAliases(c) => conv(RoomEvent::RoomAliases, c),
            RoomAvatar(c) => conv(RoomEvent::RoomAvatar, c),
            RoomCanonicalAlias(c) => conv(RoomEvent::RoomCanonicalAlias, c),
            RoomCreate(c) => conv(RoomEvent::RoomCreate, c),
            RoomEncrypted(c) => conv(RoomEvent::RoomEncrypted, c),
            RoomEncryption(c) => conv(RoomEvent::RoomEncryption, c),
            RoomGuestAccess(c) => conv(RoomEvent::RoomGuestAccess, c),
            RoomHistoryVisibility(c) => conv(RoomEvent::RoomHistoryVisibility, c),
            RoomJoinRules(c) => conv(RoomEvent::RoomJoinRules, c),
            RoomMember(c) => conv(RoomEvent::RoomMember, c),
            RoomMessage(c) => conv(RoomEvent::RoomMessage, c),
            RoomMessageFeedback(c) => conv(RoomEvent::RoomMessageFeedback, c),
            RoomName(c) => conv(RoomEvent::RoomName, c),
            RoomPinnedEvents(c) => conv(RoomEvent::RoomPinnedEvents, c),
            RoomPowerLevels(c) => conv(RoomEvent::RoomPowerLevels, c),
            RoomRedaction(c) => conv(RoomEvent::RoomRedaction, c),
            RoomServerAcl(c) => conv(RoomEvent::RoomServerAcl, c),
            RoomThirdPartyInvite(c) => conv(RoomEvent::RoomThirdPartyInvite, c),
            RoomTombstone(c) => conv(RoomEvent::RoomTombstone, c),
            RoomTopic(c) => conv(RoomEvent::RoomTopic, c),
            Sticker(c) => conv(RoomEvent::Sticker, c),
            CustomRoom(c) => conv(RoomEvent::CustomRoom, c),
            CustomState(c) => conv(RoomEvent::CustomState, c),
        }
    }
}

impl TryFromRaw for StateEvent {
    type Raw = raw::StateEvent;
    type Err = String;

    fn try_from_raw(raw: raw::StateEvent) -> Result<Self, Self::Err> {
        use crate::util::try_convert_variant as conv;
        use raw::StateEvent::*;

        match raw {
            RoomAliases(c) => conv(StateEvent::RoomAliases, c),
            RoomAvatar(c) => conv(StateEvent::RoomAvatar, c),
            RoomCanonicalAlias(c) => conv(StateEvent::RoomCanonicalAlias, c),
            RoomCreate(c) => conv(StateEvent::RoomCreate, c),
            RoomEncryption(c) => conv(StateEvent::RoomEncryption, c),
            RoomGuestAccess(c) => conv(StateEvent::RoomGuestAccess, c),
            RoomHistoryVisibility(c) => conv(StateEvent::RoomHistoryVisibility, c),
            RoomJoinRules(c) => conv(StateEvent::RoomJoinRules, c),
            RoomMember(c) => conv(StateEvent::RoomMember, c),
            RoomName(c) => conv(StateEvent::RoomName, c),
            RoomPinnedEvents(c) => conv(StateEvent::RoomPinnedEvents, c),
            RoomPowerLevels(c) => conv(StateEvent::RoomPowerLevels, c),
            RoomServerAcl(c) => conv(StateEvent::RoomServerAcl, c),
            RoomThirdPartyInvite(c) => conv(StateEvent::RoomThirdPartyInvite, c),
            RoomTombstone(c) => conv(StateEvent::RoomTombstone, c),
            RoomTopic(c) => conv(StateEvent::RoomTopic, c),
            CustomState(c) => conv(StateEvent::CustomState, c),
        }
    }
}

impl_from_for_enum!(Event, AnswerEvent, CallAnswer);
impl_from_for_enum!(Event, CandidatesEvent, CallCandidates);
impl_from_for_enum!(Event, HangupEvent, CallHangup);
impl_from_for_enum!(Event, InviteEvent, CallInvite);
impl_from_for_enum!(Event, DirectEvent, Direct);
impl_from_for_enum!(Event, DummyEvent, Dummy);
impl_from_for_enum!(Event, ForwardedRoomKeyEvent, ForwardedRoomKey);
impl_from_for_enum!(Event, FullyReadEvent, FullyRead);
impl_from_for_enum!(Event, AcceptEvent, KeyVerificationAccept);
impl_from_for_enum!(Event, CancelEvent, KeyVerificationCancel);
impl_from_for_enum!(Event, KeyEvent, KeyVerificationKey);
impl_from_for_enum!(Event, MacEvent, KeyVerificationMac);
impl_from_for_enum!(Event, RequestEvent, KeyVerificationRequest);
impl_from_for_enum!(Event, StartEvent, KeyVerificationStart);
impl_from_for_enum!(Event, IgnoredUserListEvent, IgnoredUserList);
impl_from_for_enum!(Event, PresenceEvent, Presence);
impl_from_for_enum!(Event, PushRulesEvent, PushRules);
impl_from_for_enum!(Event, ReceiptEvent, Receipt);
impl_from_for_enum!(Event, AliasesEvent, RoomAliases);
impl_from_for_enum!(Event, AvatarEvent, RoomAvatar);
impl_from_for_enum!(Event, CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_for_enum!(Event, CreateEvent, RoomCreate);
impl_from_for_enum!(Event, EncryptedEvent, RoomEncrypted);
impl_from_for_enum!(Event, EncryptionEvent, RoomEncryption);
impl_from_for_enum!(Event, GuestAccessEvent, RoomGuestAccess);
impl_from_for_enum!(Event, HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_for_enum!(Event, JoinRulesEvent, RoomJoinRules);
impl_from_for_enum!(Event, MemberEvent, RoomMember);
impl_from_for_enum!(Event, MessageEvent, RoomMessage);
impl_from_for_enum!(Event, FeedbackEvent, RoomMessageFeedback);
impl_from_for_enum!(Event, NameEvent, RoomName);
impl_from_for_enum!(Event, PinnedEventsEvent, RoomPinnedEvents);
impl_from_for_enum!(Event, PowerLevelsEvent, RoomPowerLevels);
impl_from_for_enum!(Event, RedactionEvent, RoomRedaction);
impl_from_for_enum!(Event, ServerAclEvent, RoomServerAcl);
impl_from_for_enum!(Event, ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_for_enum!(Event, TombstoneEvent, RoomTombstone);
impl_from_for_enum!(Event, TopicEvent, RoomTopic);
impl_from_for_enum!(Event, RoomKeyEvent, RoomKey);
impl_from_for_enum!(Event, RoomKeyRequestEvent, RoomKeyRequest);
impl_from_for_enum!(Event, StickerEvent, Sticker);
impl_from_for_enum!(Event, TagEvent, Tag);
impl_from_for_enum!(Event, TypingEvent, Typing);
impl_from_for_enum!(Event, CustomEvent, Custom);
impl_from_for_enum!(Event, CustomRoomEvent, CustomRoom);
impl_from_for_enum!(Event, CustomStateEvent, CustomState);

impl_from_for_enum!(RoomEvent, AnswerEvent, CallAnswer);
impl_from_for_enum!(RoomEvent, CandidatesEvent, CallCandidates);
impl_from_for_enum!(RoomEvent, HangupEvent, CallHangup);
impl_from_for_enum!(RoomEvent, InviteEvent, CallInvite);
impl_from_for_enum!(RoomEvent, AliasesEvent, RoomAliases);
impl_from_for_enum!(RoomEvent, AvatarEvent, RoomAvatar);
impl_from_for_enum!(RoomEvent, CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_for_enum!(RoomEvent, CreateEvent, RoomCreate);
impl_from_for_enum!(RoomEvent, EncryptedEvent, RoomEncrypted);
impl_from_for_enum!(RoomEvent, EncryptionEvent, RoomEncryption);
impl_from_for_enum!(RoomEvent, GuestAccessEvent, RoomGuestAccess);
impl_from_for_enum!(RoomEvent, HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_for_enum!(RoomEvent, JoinRulesEvent, RoomJoinRules);
impl_from_for_enum!(RoomEvent, MemberEvent, RoomMember);
impl_from_for_enum!(RoomEvent, MessageEvent, RoomMessage);
impl_from_for_enum!(RoomEvent, FeedbackEvent, RoomMessageFeedback);
impl_from_for_enum!(RoomEvent, NameEvent, RoomName);
impl_from_for_enum!(RoomEvent, PinnedEventsEvent, RoomPinnedEvents);
impl_from_for_enum!(RoomEvent, PowerLevelsEvent, RoomPowerLevels);
impl_from_for_enum!(RoomEvent, RedactionEvent, RoomRedaction);
impl_from_for_enum!(RoomEvent, ServerAclEvent, RoomServerAcl);
impl_from_for_enum!(RoomEvent, StickerEvent, Sticker);
impl_from_for_enum!(RoomEvent, ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_for_enum!(RoomEvent, TombstoneEvent, RoomTombstone);
impl_from_for_enum!(RoomEvent, TopicEvent, RoomTopic);
impl_from_for_enum!(RoomEvent, CustomRoomEvent, CustomRoom);
impl_from_for_enum!(RoomEvent, CustomStateEvent, CustomState);

impl_from_for_enum!(StateEvent, AliasesEvent, RoomAliases);
impl_from_for_enum!(StateEvent, AvatarEvent, RoomAvatar);
impl_from_for_enum!(StateEvent, CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_for_enum!(StateEvent, CreateEvent, RoomCreate);
impl_from_for_enum!(StateEvent, EncryptionEvent, RoomEncryption);
impl_from_for_enum!(StateEvent, GuestAccessEvent, RoomGuestAccess);
impl_from_for_enum!(StateEvent, HistoryVisibilityEvent, RoomHistoryVisibility);
impl_from_for_enum!(StateEvent, JoinRulesEvent, RoomJoinRules);
impl_from_for_enum!(StateEvent, MemberEvent, RoomMember);
impl_from_for_enum!(StateEvent, NameEvent, RoomName);
impl_from_for_enum!(StateEvent, PinnedEventsEvent, RoomPinnedEvents);
impl_from_for_enum!(StateEvent, PowerLevelsEvent, RoomPowerLevels);
impl_from_for_enum!(StateEvent, ServerAclEvent, RoomServerAcl);
impl_from_for_enum!(StateEvent, ThirdPartyInviteEvent, RoomThirdPartyInvite);
impl_from_for_enum!(StateEvent, TombstoneEvent, RoomTombstone);
impl_from_for_enum!(StateEvent, TopicEvent, RoomTopic);
impl_from_for_enum!(StateEvent, CustomStateEvent, CustomState);
