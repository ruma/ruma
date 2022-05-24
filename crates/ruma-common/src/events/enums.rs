use ruma_macros::{event_enum, EventEnumFromEvent};
use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use super::{
    key,
    room::{encrypted, redaction::SyncRoomRedactionEvent},
    Redact,
};
use crate::{
    serde::from_raw_json_value, EventId, MilliSecondsSinceUnixEpoch, OwnedRoomId, RoomId,
    RoomVersionId, TransactionId, UserId,
};

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        "m.direct" => super::direct,
        "m.identity_server" => super::identity_server,
        "m.ignored_user_list" => super::ignored_user_list,
        "m.push_rules" => super::push_rules,
        "m.secret_storage.default_key" => super::secret_storage::default_key,
        "m.secret_storage.key.*" => super::secret_storage::key,
    }

    /// Any room account data event.
    enum RoomAccountData {
        "m.fully_read" => super::fully_read,
        "m.tag" => super::tag,
    }

    /// Any ephemeral room event.
    enum EphemeralRoom {
        "m.receipt" => super::receipt,
        "m.typing" => super::typing,
    }

    /// Any message-like event.
    enum MessageLike {
        #[cfg(feature = "unstable-msc3246")]
        "m.audio" => super::audio,
        "m.call.answer" => super::call::answer,
        "m.call.invite" => super::call::invite,
        "m.call.hangup" => super::call::hangup,
        "m.call.candidates" => super::call::candidates,
        #[cfg(feature = "unstable-msc1767")]
        "m.emote" => super::emote,
        #[cfg(feature = "unstable-msc3551")]
        "m.file" => super::file,
        #[cfg(feature = "unstable-msc3552")]
        "m.image" => super::image,
        "m.key.verification.ready" => super::key::verification::ready,
        "m.key.verification.start" => super::key::verification::start,
        "m.key.verification.cancel" => super::key::verification::cancel,
        "m.key.verification.accept" => super::key::verification::accept,
        "m.key.verification.key" => super::key::verification::key,
        "m.key.verification.mac" => super::key::verification::mac,
        "m.key.verification.done" => super::key::verification::done,
        #[cfg(feature = "unstable-msc3488")]
        "m.location" => super::location,
        #[cfg(feature = "unstable-msc1767")]
        "m.message" => super::message,
        #[cfg(feature = "unstable-msc1767")]
        "m.notice" => super::notice,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(alias = "m.poll.start")]
        "org.matrix.msc3381.poll.start" => super::poll::start,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(alias = "m.poll.response")]
        "org.matrix.msc3381.poll.response" => super::poll::response,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(alias = "m.poll.end")]
        "org.matrix.msc3381.poll.end" => super::poll::end,
        #[cfg(feature = "unstable-msc2677")]
        "m.reaction" => super::reaction,
        "m.room.encrypted" => super::room::encrypted,
        "m.room.message" => super::room::message,
        "m.room.message.feedback" => super::room::message::feedback,
        "m.room.redaction" => super::room::redaction,
        "m.sticker" => super::sticker,
        #[cfg(feature = "unstable-msc3553")]
        "m.video" => super::video,
        #[cfg(feature = "unstable-msc3245")]
        "m.voice" => super::voice,
    }

    /// Any state event.
    enum State {
        "m.policy.rule.room" => super::policy::rule::room,
        "m.policy.rule.server" => super::policy::rule::server,
        "m.policy.rule.user" => super::policy::rule::user,
        "m.room.aliases" => super::room::aliases,
        "m.room.avatar" => super::room::avatar,
        "m.room.canonical_alias" => super::room::canonical_alias,
        "m.room.create" => super::room::create,
        "m.room.encryption" => super::room::encryption,
        "m.room.guest_access" => super::room::guest_access,
        "m.room.history_visibility" => super::room::history_visibility,
        "m.room.join_rules" => super::room::join_rules,
        "m.room.member" => super::room::member,
        "m.room.name" => super::room::name,
        "m.room.pinned_events" => super::room::pinned_events,
        "m.room.power_levels" => super::room::power_levels,
        "m.room.server_acl" => super::room::server_acl,
        "m.room.third_party_invite" => super::room::third_party_invite,
        "m.room.tombstone" => super::room::tombstone,
        "m.room.topic" => super::room::topic,
        "m.space.child" => super::space::child,
        "m.space.parent" => super::space::parent,
    }

    /// Any to-device event.
    enum ToDevice {
        "m.dummy" => super::dummy,
        "m.room_key" => super::room_key,
        "m.room_key_request" => super::room_key_request,
        "m.forwarded_room_key" => super::forwarded_room_key,
        "m.key.verification.request" => super::key::verification::request,
        "m.key.verification.ready" => super::key::verification::ready,
        "m.key.verification.start" => super::key::verification::start,
        "m.key.verification.cancel" => super::key::verification::cancel,
        "m.key.verification.accept" => super::key::verification::accept,
        "m.key.verification.key" => super::key::verification::key,
        "m.key.verification.mac" => super::key::verification::mac,
        "m.key.verification.done" => super::key::verification::done,
        "m.room.encrypted" => super::room::encrypted,
        "m.secret.request"=> super::secret::request,
        "m.secret.send" => super::secret::send,
    }
}

/// Declares an item with a doc attribute computed by some macro expression.
/// This allows documentation to be dynamically generated based on input.
/// Necessary to work around <https://github.com/rust-lang/rust/issues/52607>.
macro_rules! doc_concat {
    ( $( #[doc = $doc:expr] $( $thing:tt )* )* ) => ( $( #[doc = $doc] $( $thing )* )* );
}

macro_rules! room_ev_accessor {
    ($field:ident: $ty:ty) => {
        doc_concat! {
            #[doc = concat!("Returns this event's `", stringify!($field), "` field.")]
            pub fn $field(&self) -> $ty {
                match self {
                    Self::MessageLike(ev) => ev.$field(),
                    Self::State(ev) => ev.$field(),
                }
            }
        }
    };
}

/// Any room event.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, EventEnumFromEvent)]
pub enum AnyRoomEvent {
    /// Any message-like event.
    MessageLike(AnyMessageLikeEvent),

    /// Any state event.
    State(AnyStateEvent),
}

impl AnyRoomEvent {
    room_ev_accessor!(origin_server_ts: MilliSecondsSinceUnixEpoch);
    room_ev_accessor!(room_id: &RoomId);
    room_ev_accessor!(event_id: &EventId);
    room_ev_accessor!(sender: &UserId);

    /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::MessageLike(ev) => ev.transaction_id(),
            Self::State(ev) => ev.transaction_id(),
        }
    }
}

/// Any sync room event.
///
/// Sync room events are room event without a `room_id`, as returned in `/sync` responses.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, EventEnumFromEvent)]
pub enum AnySyncRoomEvent {
    /// Any sync message-like event.
    MessageLike(AnySyncMessageLikeEvent),

    /// Any sync state event.
    State(AnySyncStateEvent),
}

impl AnySyncRoomEvent {
    room_ev_accessor!(origin_server_ts: MilliSecondsSinceUnixEpoch);
    room_ev_accessor!(event_id: &EventId);
    room_ev_accessor!(sender: &UserId);

    /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
    pub fn transaction_id(&self) -> Option<&TransactionId> {
        match self {
            Self::MessageLike(ev) => ev.transaction_id(),
            Self::State(ev) => ev.transaction_id(),
        }
    }

    /// Converts `self` to an `AnyRoomEvent` by adding the given a room ID.
    pub fn into_full_event(self, room_id: OwnedRoomId) -> AnyRoomEvent {
        match self {
            Self::MessageLike(ev) => AnyRoomEvent::MessageLike(ev.into_full_event(room_id)),
            Self::State(ev) => AnyRoomEvent::State(ev.into_full_event(room_id)),
        }
    }
}

#[derive(Deserialize)]
#[allow(clippy::exhaustive_structs)]
struct EventDeHelper {
    pub state_key: Option<de::IgnoredAny>,
}

impl<'de> Deserialize<'de> for AnyRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(AnyRoomEvent::State(from_raw_json_value(&json)?))
        } else {
            Ok(AnyRoomEvent::MessageLike(from_raw_json_value(&json)?))
        }
    }
}

impl<'de> Deserialize<'de> for AnySyncRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(AnySyncRoomEvent::State(from_raw_json_value(&json)?))
        } else {
            Ok(AnySyncRoomEvent::MessageLike(from_raw_json_value(&json)?))
        }
    }
}

impl Redact for AnyRoomEvent {
    type Redacted = Self;

    /// Redacts `self`, referencing the given event in `unsigned.redacted_because`.
    ///
    /// Does nothing for events that are already redacted.
    fn redact(self, redaction: SyncRoomRedactionEvent, version: &RoomVersionId) -> Self {
        match self {
            Self::MessageLike(ev) => Self::MessageLike(ev.redact(redaction, version)),
            Self::State(ev) => Self::State(ev.redact(redaction, version)),
        }
    }
}

impl Redact for AnySyncRoomEvent {
    type Redacted = Self;

    /// Redacts `self`, referencing the given event in `unsigned.redacted_because`.
    ///
    /// Does nothing for events that are already redacted.
    fn redact(self, redaction: SyncRoomRedactionEvent, version: &RoomVersionId) -> Self {
        match self {
            Self::MessageLike(ev) => Self::MessageLike(ev.redact(redaction, version)),
            Self::State(ev) => Self::State(ev.redact(redaction, version)),
        }
    }
}

impl AnyMessageLikeEventContent {
    /// Get a copy of the event's `m.relates_to` field, if any.
    ///
    /// This is a helper function intended for encryption. There should not be a reason to access
    /// `m.relates_to` without first destructuring an `AnyMessageLikeEventContent` otherwise.
    pub fn relation(&self) -> Option<encrypted::Relation> {
        use super::key::verification::{
            accept::KeyVerificationAcceptEventContent, cancel::KeyVerificationCancelEventContent,
            done::KeyVerificationDoneEventContent, key::KeyVerificationKeyEventContent,
            mac::KeyVerificationMacEventContent, ready::KeyVerificationReadyEventContent,
            start::KeyVerificationStartEventContent,
        };
        #[cfg(feature = "unstable-msc3381")]
        use super::poll::{end::PollEndEventContent, response::PollResponseEventContent};

        match self {
            #[rustfmt::skip]
            Self::KeyVerificationReady(KeyVerificationReadyEventContent { relates_to, .. })
            | Self::KeyVerificationStart(KeyVerificationStartEventContent { relates_to, .. })
            | Self::KeyVerificationCancel(KeyVerificationCancelEventContent { relates_to, .. })
            | Self::KeyVerificationAccept(KeyVerificationAcceptEventContent { relates_to, .. })
            | Self::KeyVerificationKey(KeyVerificationKeyEventContent { relates_to, .. })
            | Self::KeyVerificationMac(KeyVerificationMacEventContent { relates_to, .. })
            | Self::KeyVerificationDone(KeyVerificationDoneEventContent { relates_to, .. }) => {
                let key::verification::Relation { event_id } = relates_to;
                Some(encrypted::Relation::Reference(encrypted::Reference {
                    event_id: event_id.clone(),
                }))
            }
            #[cfg(feature = "unstable-msc2677")]
            Self::Reaction(ev) => {
                use super::reaction;

                let reaction::Relation { event_id, key } = &ev.relates_to;
                Some(encrypted::Relation::Annotation(encrypted::Annotation {
                    event_id: event_id.clone(),
                    key: key.clone(),
                }))
            }
            Self::RoomEncrypted(ev) => ev.relates_to.clone(),
            Self::RoomMessage(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc1767")]
            Self::Message(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc1767")]
            Self::Notice(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc1767")]
            Self::Emote(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3245")]
            Self::Voice(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3246")]
            Self::Audio(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3488")]
            Self::Location(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3551")]
            Self::File(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3552")]
            Self::Image(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3553")]
            Self::Video(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3381")]
            Self::PollResponse(PollResponseEventContent { relates_to, .. })
            | Self::PollEnd(PollEndEventContent { relates_to, .. }) => {
                let super::poll::ReferenceRelation { event_id } = relates_to;
                Some(encrypted::Relation::Reference(encrypted::Reference {
                    event_id: event_id.clone(),
                }))
            }
            #[cfg(feature = "unstable-msc3381")]
            Self::PollStart(_) => None,
            Self::CallAnswer(_)
            | Self::CallInvite(_)
            | Self::CallHangup(_)
            | Self::CallCandidates(_)
            | Self::RoomMessageFeedback(_)
            | Self::RoomRedaction(_)
            | Self::Sticker(_)
            | Self::_Custom { .. } => None,
        }
    }
}
