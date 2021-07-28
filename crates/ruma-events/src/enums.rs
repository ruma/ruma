use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events_macros::event_enum;
use ruma_identifiers::{EventId, RoomId, RoomVersionId, UserId};
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;

use crate::{from_raw_json_value, room::redaction::SyncRedactionEvent, EventDeHelper, Redact};

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        "m.direct",
        "m.ignored_user_list",
        "m.push_rules",
    }

    /// Any room account data event.
    enum RoomAccountData {
        "m.fully_read",
        "m.tag",
    }

    /// Any ephemeral room event.
    enum EphemeralRoom {
        "m.receipt",
        "m.typing",
    }

    /// Any message event.
    enum Message {
        "m.call.answer",
        "m.call.invite",
        "m.call.hangup",
        "m.call.candidates",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.ready",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.start",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.cancel",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.accept",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.key",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.mac",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.done",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.reaction",
        "m.room.encrypted",
        "m.room.message",
        "m.room.message.feedback",
        "m.room.redaction",
        "m.sticker",
    }


    /// Any state event.
    enum State {
        "m.policy.rule.room",
        "m.policy.rule.server",
        "m.policy.rule.user",
        "m.room.aliases",
        "m.room.avatar",
        "m.room.canonical_alias",
        "m.room.create",
        "m.room.encryption",
        "m.room.guest_access",
        "m.room.history_visibility",
        "m.room.join_rules",
        "m.room.member",
        "m.room.name",
        "m.room.pinned_events",
        "m.room.power_levels",
        "m.room.server_acl",
        "m.room.third_party_invite",
        "m.room.tombstone",
        "m.room.topic",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.space.child",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.space.parent",
    }

    /// Any to-device event.
    enum ToDevice {
        "m.dummy",
        "m.room_key",
        "m.room_key_request",
        "m.forwarded_room_key",
        "m.key.verification.request",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.ready",
        "m.key.verification.start",
        "m.key.verification.cancel",
        "m.key.verification.accept",
        "m.key.verification.key",
        "m.key.verification.mac",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.key.verification.done",
        "m.room.encrypted",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.secret.request",
        #[cfg(feature = "unstable-pre-spec")]
        #[cfg_attr(docsrs, doc(cfg(feature = "unstable-pre-spec")))]
        "m.secret.send",
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
                    Self::Message(ev) => ev.$field(),
                    Self::State(ev) => ev.$field(),
                    Self::RedactedMessage(ev) => ev.$field(),
                    Self::RedactedState(ev) => ev.$field(),
                }
            }
        }
    };
}

/// Any room event.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnyRoomEvent {
    /// Any message event.
    Message(AnyMessageEvent),

    /// Any state event.
    State(AnyStateEvent),

    /// Any message event that has been redacted.
    RedactedMessage(AnyRedactedMessageEvent),

    /// Any state event that has been redacted.
    RedactedState(AnyRedactedStateEvent),
}

impl AnyRoomEvent {
    room_ev_accessor!(origin_server_ts: &MilliSecondsSinceUnixEpoch);
    room_ev_accessor!(room_id: &RoomId);
    room_ev_accessor!(event_id: &EventId);
    room_ev_accessor!(sender: &UserId);
}

/// Any sync room event (room event without a `room_id`, as returned in `/sync` responses)
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum AnySyncRoomEvent {
    /// Any sync message event
    Message(AnySyncMessageEvent),

    /// Any sync state event
    State(AnySyncStateEvent),

    /// Any sync message event that has been redacted.
    RedactedMessage(AnyRedactedSyncMessageEvent),

    /// Any sync state event that has been redacted.
    RedactedState(AnyRedactedSyncStateEvent),
}

impl AnySyncRoomEvent {
    room_ev_accessor!(origin_server_ts: &MilliSecondsSinceUnixEpoch);
    room_ev_accessor!(event_id: &EventId);
    room_ev_accessor!(sender: &UserId);

    /// Converts `self` to an `AnyRoomEvent` by adding the given a room ID.
    pub fn into_full_event(self, room_id: RoomId) -> AnyRoomEvent {
        match self {
            Self::Message(ev) => AnyRoomEvent::Message(ev.into_full_event(room_id)),
            Self::State(ev) => AnyRoomEvent::State(ev.into_full_event(room_id)),
            Self::RedactedMessage(ev) => AnyRoomEvent::RedactedMessage(ev.into_full_event(room_id)),
            Self::RedactedState(ev) => AnyRoomEvent::RedactedState(ev.into_full_event(room_id)),
        }
    }
}

impl<'de> de::Deserialize<'de> for AnyRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    AnyRoomEvent::RedactedState(from_raw_json_value(&json)?)
                }
                _ => AnyRoomEvent::State(from_raw_json_value(&json)?),
            })
        } else {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    AnyRoomEvent::RedactedMessage(from_raw_json_value(&json)?)
                }
                _ => AnyRoomEvent::Message(from_raw_json_value(&json)?),
            })
        }
    }
}

impl<'de> de::Deserialize<'de> for AnySyncRoomEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper { state_key, unsigned, .. } = from_raw_json_value(&json)?;

        if state_key.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    AnySyncRoomEvent::RedactedState(from_raw_json_value(&json)?)
                }
                _ => AnySyncRoomEvent::State(from_raw_json_value(&json)?),
            })
        } else {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    AnySyncRoomEvent::RedactedMessage(from_raw_json_value(&json)?)
                }
                _ => AnySyncRoomEvent::Message(from_raw_json_value(&json)?),
            })
        }
    }
}

/// Any redacted room event.
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum AnyRedactedRoomEvent {
    /// Any message event that has been redacted.
    Message(AnyRedactedMessageEvent),

    /// Any state event that has been redacted.
    State(AnyRedactedStateEvent),
}

impl Redact for AnyRoomEvent {
    type Redacted = AnyRedactedRoomEvent;

    /// Redacts `self`, referencing the given event in `unsigned.redacted_because`.
    ///
    /// Does nothing for events that are already redacted.
    fn redact(self, redaction: SyncRedactionEvent, version: &RoomVersionId) -> Self::Redacted {
        match self {
            Self::Message(ev) => Self::Redacted::Message(ev.redact(redaction, version)),
            Self::State(ev) => Self::Redacted::State(ev.redact(redaction, version)),
            Self::RedactedMessage(ev) => Self::Redacted::Message(ev),
            Self::RedactedState(ev) => Self::Redacted::State(ev),
        }
    }
}

impl From<AnyRedactedRoomEvent> for AnyRoomEvent {
    fn from(ev: AnyRedactedRoomEvent) -> Self {
        match ev {
            AnyRedactedRoomEvent::Message(ev) => Self::RedactedMessage(ev),
            AnyRedactedRoomEvent::State(ev) => Self::RedactedState(ev),
        }
    }
}

/// Any redacted sync room event (room event without a `room_id`, as returned in `/sync` responses)
#[allow(clippy::large_enum_variant, clippy::exhaustive_enums)]
#[derive(Clone, Debug)]
pub enum AnyRedactedSyncRoomEvent {
    /// Any sync message event that has been redacted.
    Message(AnyRedactedSyncMessageEvent),

    /// Any sync state event that has been redacted.
    State(AnyRedactedSyncStateEvent),
}

impl Redact for AnySyncRoomEvent {
    type Redacted = AnyRedactedSyncRoomEvent;

    /// Redacts `self`, referencing the given event in `unsigned.redacted_because`.
    ///
    /// Does nothing for events that are already redacted.
    fn redact(self, redaction: SyncRedactionEvent, version: &RoomVersionId) -> Self::Redacted {
        match self {
            Self::Message(ev) => Self::Redacted::Message(ev.redact(redaction, version)),
            Self::State(ev) => Self::Redacted::State(ev.redact(redaction, version)),
            Self::RedactedMessage(ev) => Self::Redacted::Message(ev),
            Self::RedactedState(ev) => Self::Redacted::State(ev),
        }
    }
}

impl From<AnyRedactedSyncRoomEvent> for AnySyncRoomEvent {
    fn from(ev: AnyRedactedSyncRoomEvent) -> Self {
        match ev {
            AnyRedactedSyncRoomEvent::Message(ev) => Self::RedactedMessage(ev),
            AnyRedactedSyncRoomEvent::State(ev) => Self::RedactedState(ev),
        }
    }
}

impl AnyMessageEventContent {
    /// Get a copy of the event's `m.relates_to` field, if any.
    ///
    /// This is a helper function intended for encryption. There should not be a reason to access
    /// `m.relates_to` without first destructuring an `AnyMessageEventContent` otherwise.
    pub fn relation(&self) -> Option<crate::room::encrypted::Relation> {
        #[cfg(feature = "unstable-pre-spec")]
        use crate::key::verification::{
            accept::AcceptEventContent, cancel::CancelEventContent, done::DoneEventContent,
            key::KeyEventContent, mac::MacEventContent, ready::ReadyEventContent,
            start::StartEventContent,
        };

        match self {
            #[cfg(feature = "unstable-pre-spec")]
            #[rustfmt::skip]
            AnyMessageEventContent::KeyVerificationReady(ReadyEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationStart(StartEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationCancel(CancelEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationAccept(AcceptEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationKey(KeyEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationMac(MacEventContent { relates_to, .. })
            | AnyMessageEventContent::KeyVerificationDone(DoneEventContent { relates_to, .. }) => {
                Some(relates_to.clone().into())
            },
            #[cfg(feature = "unstable-pre-spec")]
            AnyMessageEventContent::Reaction(ev) => Some(ev.relates_to.clone().into()),
            AnyMessageEventContent::RoomEncrypted(ev) => ev.relates_to.clone(),
            AnyMessageEventContent::RoomMessage(ev) => ev.relates_to.clone().map(Into::into),
            AnyMessageEventContent::CallAnswer(_)
            | AnyMessageEventContent::CallInvite(_)
            | AnyMessageEventContent::CallHangup(_)
            | AnyMessageEventContent::CallCandidates(_)
            | AnyMessageEventContent::RoomMessageFeedback(_)
            | AnyMessageEventContent::RoomRedaction(_)
            | AnyMessageEventContent::Sticker(_)
            | AnyMessageEventContent::_Custom(_) => None,
        }
    }
}
