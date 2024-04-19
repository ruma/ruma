use ruma_common::{
    serde::from_raw_json_value, EventId, MilliSecondsSinceUnixEpoch, OwnedRoomId, RoomId,
    TransactionId, UserId,
};
use ruma_macros::{event_enum, EventEnumFromEvent};
use serde::{de, Deserialize};
use serde_json::value::RawValue as RawJsonValue;

use super::room::encrypted;

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
        #[cfg(feature = "unstable-msc2867")]
        #[ruma_enum(alias = "m.marked_unread")]
        "com.famedly.marked_unread" => super::marked_unread,
    }

    /// Any ephemeral room event.
    enum EphemeralRoom {
        "m.receipt" => super::receipt,
        "m.typing" => super::typing,
    }

    /// Any message-like event.
    enum MessageLike {
        #[cfg(feature = "unstable-msc3927")]
        #[ruma_enum(alias = "m.audio")]
        "org.matrix.msc1767.audio" => super::audio,
        "m.call.answer" => super::call::answer,
        "m.call.invite" => super::call::invite,
        "m.call.hangup" => super::call::hangup,
        "m.call.candidates" => super::call::candidates,
        "m.call.negotiate" => super::call::negotiate,
        "m.call.reject" => super::call::reject,
        #[cfg(feature = "unstable-msc3291")]
        #[ruma_enum(alias = "m.call.sdp_stream_metadata_changed")]
        "org.matrix.call.sdp_stream_metadata_changed" => super::call::sdp_stream_metadata_changed,
        "m.call.select_answer" => super::call::select_answer,
        #[cfg(feature = "unstable-msc3954")]
        #[ruma_enum(alias = "m.emote")]
        "org.matrix.msc1767.emote" => super::emote,
        #[cfg(feature = "unstable-msc3956")]
        #[ruma_enum(alias = "m.encrypted")]
        "org.matrix.msc1767.encrypted" => super::encrypted,
        #[cfg(feature = "unstable-msc3551")]
        #[ruma_enum(alias = "m.file")]
        "org.matrix.msc1767.file" => super::file,
        #[cfg(feature = "unstable-msc3552")]
        #[ruma_enum(alias = "m.image")]
        "org.matrix.msc1767.image" => super::image,
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
        #[ruma_enum(alias = "m.message")]
        "org.matrix.msc1767.message" => super::message,
        #[cfg(feature = "unstable-msc3381")]
        "m.poll.start" => super::poll::start,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(ident = UnstablePollStart)]
        "org.matrix.msc3381.poll.start" => super::poll::unstable_start,
        #[cfg(feature = "unstable-msc3381")]
        "m.poll.response" => super::poll::response,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(ident = UnstablePollResponse)]
        "org.matrix.msc3381.poll.response" => super::poll::unstable_response,
        #[cfg(feature = "unstable-msc3381")]
        "m.poll.end" => super::poll::end,
        #[cfg(feature = "unstable-msc3381")]
        #[ruma_enum(ident = UnstablePollEnd)]
        "org.matrix.msc3381.poll.end" => super::poll::unstable_end,
        "m.reaction" => super::reaction,
        "m.room.encrypted" => super::room::encrypted,
        "m.room.message" => super::room::message,
        "m.room.redaction" => super::room::redaction,
        "m.sticker" => super::sticker,
        #[cfg(feature = "unstable-msc3553")]
        #[ruma_enum(alias = "m.video")]
        "org.matrix.msc1767.video" => super::video,
        #[cfg(feature = "unstable-msc3245")]
        #[ruma_enum(alias = "m.voice")]
        "org.matrix.msc3245.voice.v2" => super::voice,
        #[cfg(feature = "unstable-msc4075")]
        #[ruma_enum(alias = "m.call.notify")]
        "org.matrix.msc4075.call.notify" => super::call::notify,
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
        #[cfg(feature = "unstable-msc3401")]
        #[ruma_enum(alias = "m.call.member")]
        "org.matrix.msc3401.call.member" => super::call::member,
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

macro_rules! timeline_event_accessors {
    (
        $(
            #[doc = $docs:literal]
            pub fn $field:ident(&self) -> $ty:ty;
        )*
    ) => {
        $(
            #[doc = $docs]
            pub fn $field(&self) -> $ty {
                match self {
                    Self::MessageLike(ev) => ev.$field(),
                    Self::State(ev) => ev.$field(),
                }
            }
        )*
    };
}

/// Any room event.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, EventEnumFromEvent)]
pub enum AnyTimelineEvent {
    /// Any message-like event.
    MessageLike(AnyMessageLikeEvent),

    /// Any state event.
    State(AnyStateEvent),
}

impl AnyTimelineEvent {
    timeline_event_accessors! {
        /// Returns this event's `origin_server_ts` field.
        pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch;

        /// Returns this event's `room_id` field.
        pub fn room_id(&self) -> &RoomId;

        /// Returns this event's `event_id` field.
        pub fn event_id(&self) -> &EventId;

        /// Returns this event's `sender` field.
        pub fn sender(&self) -> &UserId;

        /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
        pub fn transaction_id(&self) -> Option<&TransactionId>;
    }

    /// Returns this event's `type`.
    pub fn event_type(&self) -> TimelineEventType {
        match self {
            Self::MessageLike(e) => e.event_type().into(),
            Self::State(e) => e.event_type().into(),
        }
    }
}

/// Any sync room event.
///
/// Sync room events are room event without a `room_id`, as returned in `/sync` responses.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, EventEnumFromEvent)]
pub enum AnySyncTimelineEvent {
    /// Any sync message-like event.
    MessageLike(AnySyncMessageLikeEvent),

    /// Any sync state event.
    State(AnySyncStateEvent),
}

impl AnySyncTimelineEvent {
    timeline_event_accessors! {
        /// Returns this event's `origin_server_ts` field.
        pub fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch;

        /// Returns this event's `event_id` field.
        pub fn event_id(&self) -> &EventId;

        /// Returns this event's `sender` field.
        pub fn sender(&self) -> &UserId;

        /// Returns this event's `transaction_id` from inside `unsigned`, if there is one.
        pub fn transaction_id(&self) -> Option<&TransactionId>;
    }

    /// Returns this event's `type`.
    pub fn event_type(&self) -> TimelineEventType {
        match self {
            Self::MessageLike(e) => e.event_type().into(),
            Self::State(e) => e.event_type().into(),
        }
    }

    /// Converts `self` to an `AnyTimelineEvent` by adding the given a room ID.
    pub fn into_full_event(self, room_id: OwnedRoomId) -> AnyTimelineEvent {
        match self {
            Self::MessageLike(ev) => AnyTimelineEvent::MessageLike(ev.into_full_event(room_id)),
            Self::State(ev) => AnyTimelineEvent::State(ev.into_full_event(room_id)),
        }
    }
}

impl From<AnyTimelineEvent> for AnySyncTimelineEvent {
    fn from(ev: AnyTimelineEvent) -> Self {
        match ev {
            AnyTimelineEvent::MessageLike(ev) => Self::MessageLike(ev.into()),
            AnyTimelineEvent::State(ev) => Self::State(ev.into()),
        }
    }
}

#[derive(Deserialize)]
#[allow(clippy::exhaustive_structs)]
struct EventDeHelper {
    state_key: Option<de::IgnoredAny>,
}

impl<'de> Deserialize<'de> for AnyTimelineEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(AnyTimelineEvent::State(from_raw_json_value(&json)?))
        } else {
            Ok(AnyTimelineEvent::MessageLike(from_raw_json_value(&json)?))
        }
    }
}

impl<'de> Deserialize<'de> for AnySyncTimelineEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(AnySyncTimelineEvent::State(from_raw_json_value(&json)?))
        } else {
            Ok(AnySyncTimelineEvent::MessageLike(from_raw_json_value(&json)?))
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
        use super::poll::{
            end::PollEndEventContent, response::PollResponseEventContent,
            unstable_end::UnstablePollEndEventContent,
            unstable_response::UnstablePollResponseEventContent,
        };

        match self {
            #[rustfmt::skip]
            Self::KeyVerificationReady(KeyVerificationReadyEventContent { relates_to, .. })
            | Self::KeyVerificationStart(KeyVerificationStartEventContent { relates_to, .. })
            | Self::KeyVerificationCancel(KeyVerificationCancelEventContent { relates_to, .. })
            | Self::KeyVerificationAccept(KeyVerificationAcceptEventContent { relates_to, .. })
            | Self::KeyVerificationKey(KeyVerificationKeyEventContent { relates_to, .. })
            | Self::KeyVerificationMac(KeyVerificationMacEventContent { relates_to, .. })
            | Self::KeyVerificationDone(KeyVerificationDoneEventContent { relates_to, .. }) => {
                Some(encrypted::Relation::Reference(relates_to.clone()))
            },
            Self::Reaction(ev) => Some(encrypted::Relation::Annotation(ev.relates_to.clone())),
            Self::RoomEncrypted(ev) => ev.relates_to.clone(),
            Self::RoomMessage(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc1767")]
            Self::Message(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3954")]
            Self::Emote(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3956")]
            Self::Encrypted(ev) => ev.relates_to.clone(),
            #[cfg(feature = "unstable-msc3245")]
            Self::Voice(ev) => ev.relates_to.clone().map(Into::into),
            #[cfg(feature = "unstable-msc3927")]
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
            | Self::UnstablePollResponse(UnstablePollResponseEventContent { relates_to, .. })
            | Self::PollEnd(PollEndEventContent { relates_to, .. })
            | Self::UnstablePollEnd(UnstablePollEndEventContent { relates_to, .. }) => {
                Some(encrypted::Relation::Reference(relates_to.clone()))
            }
            #[cfg(feature = "unstable-msc3381")]
            Self::PollStart(_) | Self::UnstablePollStart(_) => None,
            #[cfg(feature = "unstable-msc4075")]
            Self::CallNotify(_) => None,
            #[cfg(feature = "unstable-msc3291")]
            Self::CallSdpStreamMetadataChanged(_) => None,
            Self::CallNegotiate(_)
            | Self::CallReject(_)
            | Self::CallSelectAnswer(_)
            | Self::CallAnswer(_)
            | Self::CallInvite(_)
            | Self::CallHangup(_)
            | Self::CallCandidates(_)
            | Self::RoomRedaction(_)
            | Self::Sticker(_)
            | Self::_Custom { .. } => None,
        }
    }
}
