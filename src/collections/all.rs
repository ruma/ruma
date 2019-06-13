//! Enums for heterogeneous collections of events, inclusive for every event type that implements
//! the trait of the same name.

use crate::{
    call::{
        answer::AnswerEvent, candidates::CandidatesEvent, hangup::HangupEvent, invite::InviteEvent,
    },
    direct::DirectEvent,
    fully_read::FullyReadEvent,
    ignored_user_list::IgnoredUserListEvent,
    presence::PresenceEvent,
    receipt::ReceiptEvent,
    room::{
        aliases::AliasesEvent,
        avatar::AvatarEvent,
        canonical_alias::CanonicalAliasEvent,
        create::CreateEvent,
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
    sticker::StickerEvent,
    tag::TagEvent,
    typing::TypingEvent,
    CustomEvent, CustomRoomEvent, CustomStateEvent, EventType,
};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};

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
    /// m.fully_read
    FullyRead(FullyReadEvent),
    /// m.ignored_user_list
    IgnoredUserList(IgnoredUserListEvent),
    /// m.presence
    Presence(PresenceEvent),
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
    /// m.room.server_acl,
    RoomServerAcl(ServerAclEvent),
    /// m.room.third_party_invite
    RoomThirdPartyInvite(ThirdPartyInviteEvent),
    /// m.room.tombstone
    RoomTombstone(TombstoneEvent),
    /// m.room.topic
    RoomTopic(TopicEvent),
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
    /// m.room.server_acl,
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
    /// m.room.server_acl,
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

impl Serialize for Event {
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
            Event::FullyRead(ref event) => event.serialize(serializer),
            Event::IgnoredUserList(ref event) => event.serialize(serializer),
            Event::Presence(ref event) => event.serialize(serializer),
            Event::Receipt(ref event) => event.serialize(serializer),
            Event::RoomAliases(ref event) => event.serialize(serializer),
            Event::RoomAvatar(ref event) => event.serialize(serializer),
            Event::RoomCanonicalAlias(ref event) => event.serialize(serializer),
            Event::RoomCreate(ref event) => event.serialize(serializer),
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
            Event::Sticker(ref event) => event.serialize(serializer),
            Event::Tag(ref event) => event.serialize(serializer),
            Event::Typing(ref event) => event.serialize(serializer),
            Event::Custom(ref event) => event.serialize(serializer),
            Event::CustomRoom(ref event) => event.serialize(serializer),
            Event::CustomState(ref event) => event.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::CallAnswer => {
                let event = match from_value::<AnswerEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::CallAnswer(event))
            }
            EventType::CallCandidates => {
                let event = match from_value::<CandidatesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::CallCandidates(event))
            }
            EventType::CallHangup => {
                let event = match from_value::<HangupEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::CallHangup(event))
            }
            EventType::CallInvite => {
                let event = match from_value::<InviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::CallInvite(event))
            }
            EventType::Direct => {
                let event = match from_value::<DirectEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Direct(event))
            }
            EventType::FullyRead => {
                let event = match from_value::<FullyReadEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::FullyRead(event))
            }
            EventType::IgnoredUserList => {
                let event = match from_value::<IgnoredUserListEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::IgnoredUserList(event))
            }
            EventType::Presence => {
                let event = match from_value::<PresenceEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Presence(event))
            }
            EventType::Receipt => {
                let event = match from_value::<ReceiptEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Receipt(event))
            }
            EventType::RoomAliases => {
                let event = match from_value::<AliasesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomAliases(event))
            }
            EventType::RoomAvatar => {
                let event = match from_value::<AvatarEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomAvatar(event))
            }
            EventType::RoomCanonicalAlias => {
                let event = match from_value::<CanonicalAliasEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomCanonicalAlias(event))
            }
            EventType::RoomCreate => {
                let event = match from_value::<CreateEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomCreate(event))
            }
            EventType::RoomGuestAccess => {
                let event = match from_value::<GuestAccessEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomGuestAccess(event))
            }
            EventType::RoomHistoryVisibility => {
                let event = match from_value::<HistoryVisibilityEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomHistoryVisibility(event))
            }
            EventType::RoomJoinRules => {
                let event = match from_value::<JoinRulesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomJoinRules(event))
            }
            EventType::RoomMember => {
                let event = match from_value::<MemberEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomMember(event))
            }
            EventType::RoomMessage => {
                let event = match from_value::<MessageEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomMessage(event))
            }
            EventType::RoomMessageFeedback => {
                let event = match from_value::<FeedbackEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomMessageFeedback(event))
            }
            EventType::RoomName => {
                let event = match from_value::<NameEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomName(event))
            }
            EventType::RoomPinnedEvents => {
                let event = match from_value::<PinnedEventsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomPinnedEvents(event))
            }
            EventType::RoomPowerLevels => {
                let event = match from_value::<PowerLevelsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomPowerLevels(event))
            }
            EventType::RoomRedaction => {
                let event = match from_value::<RedactionEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomRedaction(event))
            }
            EventType::RoomServerAcl => {
                let event = match from_value::<ServerAclEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomServerAcl(event))
            }
            EventType::RoomThirdPartyInvite => {
                let event = match from_value::<ThirdPartyInviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomThirdPartyInvite(event))
            }
            EventType::RoomTombstone => {
                let event = match from_value::<TombstoneEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomTombstone(event))
            }
            EventType::RoomTopic => {
                let event = match from_value::<TopicEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::RoomTopic(event))
            }
            EventType::Sticker => {
                let event = match from_value::<StickerEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Sticker(event))
            }
            EventType::Tag => {
                let event = match from_value::<TagEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Tag(event))
            }
            EventType::Typing => {
                let event = match from_value::<TypingEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(Event::Typing(event))
            }
            EventType::Custom(_) => {
                if value.get("state_key").is_some() {
                    let event = match from_value::<CustomStateEvent>(value) {
                        Ok(event) => event,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(Event::CustomState(event))
                } else if value.get("event_id").is_some()
                    && value.get("room_id").is_some()
                    && value.get("sender").is_some()
                {
                    let event = match from_value::<CustomRoomEvent>(value) {
                        Ok(event) => event,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(Event::CustomRoom(event))
                } else {
                    let event = match from_value::<CustomEvent>(value) {
                        Ok(event) => event,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(Event::Custom(event))
                }
            }
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

impl<'de> Deserialize<'de> for RoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::CallAnswer => {
                let event = match from_value::<AnswerEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallAnswer(event))
            }
            EventType::CallCandidates => {
                let event = match from_value::<CandidatesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallCandidates(event))
            }
            EventType::CallHangup => {
                let event = match from_value::<HangupEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallHangup(event))
            }
            EventType::CallInvite => {
                let event = match from_value::<InviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::CallInvite(event))
            }
            EventType::RoomAliases => {
                let event = match from_value::<AliasesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomAliases(event))
            }
            EventType::RoomAvatar => {
                let event = match from_value::<AvatarEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomAvatar(event))
            }
            EventType::RoomCanonicalAlias => {
                let event = match from_value::<CanonicalAliasEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomCanonicalAlias(event))
            }
            EventType::RoomCreate => {
                let event = match from_value::<CreateEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomCreate(event))
            }
            EventType::RoomGuestAccess => {
                let event = match from_value::<GuestAccessEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomGuestAccess(event))
            }
            EventType::RoomHistoryVisibility => {
                let event = match from_value::<HistoryVisibilityEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomHistoryVisibility(event))
            }
            EventType::RoomJoinRules => {
                let event = match from_value::<JoinRulesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomJoinRules(event))
            }
            EventType::RoomMember => {
                let event = match from_value::<MemberEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomMember(event))
            }
            EventType::RoomMessage => {
                let event = match from_value::<MessageEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomMessage(event))
            }
            EventType::RoomMessageFeedback => {
                let event = match from_value::<FeedbackEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomMessageFeedback(event))
            }
            EventType::RoomName => {
                let event = match from_value::<NameEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomName(event))
            }
            EventType::RoomPinnedEvents => {
                let event = match from_value::<PinnedEventsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomPinnedEvents(event))
            }
            EventType::RoomPowerLevels => {
                let event = match from_value::<PowerLevelsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomPowerLevels(event))
            }
            EventType::RoomRedaction => {
                let event = match from_value::<RedactionEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomRedaction(event))
            }
            EventType::RoomServerAcl => {
                let event = match from_value::<ServerAclEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomServerAcl(event))
            }
            EventType::RoomThirdPartyInvite => {
                let event = match from_value::<ThirdPartyInviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomThirdPartyInvite(event))
            }
            EventType::RoomTombstone => {
                let event = match from_value::<TombstoneEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomTombstone(event))
            }
            EventType::RoomTopic => {
                let event = match from_value::<TopicEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::RoomTopic(event))
            }
            EventType::Sticker => {
                let event = match from_value::<StickerEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(RoomEvent::Sticker(event))
            }
            EventType::Custom(_) => {
                if value.get("state_key").is_some() {
                    let event = match from_value::<CustomStateEvent>(value) {
                        Ok(event) => event,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(RoomEvent::CustomState(event))
                } else {
                    let event = match from_value::<CustomRoomEvent>(value) {
                        Ok(event) => event,
                        Err(error) => return Err(D::Error::custom(error.to_string())),
                    };

                    Ok(RoomEvent::CustomRoom(event))
                }
            }
            EventType::Direct
            | EventType::FullyRead
            | EventType::IgnoredUserList
            | EventType::Presence
            | EventType::Receipt
            | EventType::Tag
            | EventType::Typing => Err(D::Error::custom("not a room event".to_string())),
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
}

impl<'de> Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        let event_type_value = match value.get("type") {
            Some(value) => value.clone(),
            None => return Err(D::Error::missing_field("type")),
        };

        let event_type = match from_value::<EventType>(event_type_value.clone()) {
            Ok(event_type) => event_type,
            Err(error) => return Err(D::Error::custom(error.to_string())),
        };

        match event_type {
            EventType::RoomAliases => {
                let event = match from_value::<AliasesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomAliases(event))
            }
            EventType::RoomAvatar => {
                let event = match from_value::<AvatarEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomAvatar(event))
            }
            EventType::RoomCanonicalAlias => {
                let event = match from_value::<CanonicalAliasEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomCanonicalAlias(event))
            }
            EventType::RoomCreate => {
                let event = match from_value::<CreateEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomCreate(event))
            }
            EventType::RoomGuestAccess => {
                let event = match from_value::<GuestAccessEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomGuestAccess(event))
            }
            EventType::RoomHistoryVisibility => {
                let event = match from_value::<HistoryVisibilityEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomHistoryVisibility(event))
            }
            EventType::RoomJoinRules => {
                let event = match from_value::<JoinRulesEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomJoinRules(event))
            }
            EventType::RoomMember => {
                let event = match from_value::<MemberEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomMember(event))
            }
            EventType::RoomName => {
                let event = match from_value::<NameEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomName(event))
            }
            EventType::RoomPinnedEvents => {
                let event = match from_value::<PinnedEventsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomPinnedEvents(event))
            }
            EventType::RoomPowerLevels => {
                let event = match from_value::<PowerLevelsEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomPowerLevels(event))
            }
            EventType::RoomServerAcl => {
                let event = match from_value::<ServerAclEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomServerAcl(event))
            }
            EventType::RoomThirdPartyInvite => {
                let event = match from_value::<ThirdPartyInviteEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomThirdPartyInvite(event))
            }
            EventType::RoomTombstone => {
                let event = match from_value::<TombstoneEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomTombstone(event))
            }
            EventType::RoomTopic => {
                let event = match from_value::<TopicEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::RoomTopic(event))
            }
            EventType::Custom(_) => {
                let event = match from_value::<CustomStateEvent>(value) {
                    Ok(event) => event,
                    Err(error) => return Err(D::Error::custom(error.to_string())),
                };

                Ok(StateEvent::CustomState(event))
            }
            EventType::CallAnswer
            | EventType::CallCandidates
            | EventType::CallHangup
            | EventType::CallInvite
            | EventType::Direct
            | EventType::FullyRead
            | EventType::IgnoredUserList
            | EventType::Presence
            | EventType::Receipt
            | EventType::RoomMessage
            | EventType::RoomMessageFeedback
            | EventType::RoomRedaction
            | EventType::Sticker
            | EventType::Tag
            | EventType::Typing => Err(D::Error::custom("not a state event".to_string())),
        }
    }
}

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
impl_from_t_for_event!(FullyReadEvent, FullyRead);
impl_from_t_for_event!(IgnoredUserListEvent, IgnoredUserList);
impl_from_t_for_event!(PresenceEvent, Presence);
impl_from_t_for_event!(ReceiptEvent, Receipt);
impl_from_t_for_event!(AliasesEvent, RoomAliases);
impl_from_t_for_event!(AvatarEvent, RoomAvatar);
impl_from_t_for_event!(CanonicalAliasEvent, RoomCanonicalAlias);
impl_from_t_for_event!(CreateEvent, RoomCreate);
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
