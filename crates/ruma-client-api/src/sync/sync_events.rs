//! `GET /_matrix/client/*/sync`
//!
//! Get all new events from all rooms since the last sync or a given point in time.

use js_int::UInt;
use ruma_common::{
    serde::{from_raw_json_value, JsonCastable, JsonObject},
    EventId, MilliSecondsSinceUnixEpoch, OwnedUserId, UserId,
};
use ruma_events::{
    AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, OriginalStateEvent,
    OriginalSyncStateEvent, PossiblyRedactedStateEventContent, RedactContent, RedactedStateEvent,
    RedactedStateEventContent, RedactedSyncStateEvent, StateEvent, StateEventType,
    StaticStateEventContent, StrippedStateEvent, SyncStateEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

pub mod v3;

#[cfg(feature = "unstable-msc4186")]
pub mod v5;

/// Unread notifications count.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<UInt>,

    /// The total number of unread notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<UInt>,
}

impl UnreadNotificationsCount {
    /// Creates an empty `UnreadNotificationsCount`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no notification count updates.
    pub fn is_empty(&self) -> bool {
        self.highlight_count.is_none() && self.notification_count.is_none()
    }
}

/// Information on E2E device updates.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DeviceLists {
    /// List of users who have updated their device identity keys or who now
    /// share an encrypted room with the client since the previous sync.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<OwnedUserId>,

    /// List of users who no longer share encrypted rooms since the previous sync
    /// response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<OwnedUserId>,
}

impl DeviceLists {
    /// Creates an empty `DeviceLists`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns true if there are no device list updates.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.left.is_empty()
    }
}

/// Possible event formats that may appear in stripped state.
#[derive(Debug, Clone)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[allow(clippy::large_enum_variant)]
pub enum StrippedState {
    /// A stripped state event.
    Stripped(AnyStrippedStateEvent),

    /// An event using the sync format.
    #[cfg(feature = "unstable-msc4319")]
    Sync(AnySyncStateEvent),
}

impl StrippedState {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> StateEventType {
        match self {
            Self::Stripped(event) => event.event_type(),
            #[cfg(feature = "unstable-msc4319")]
            Self::Sync(event) => event.event_type(),
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Stripped(event) => event.sender(),
            #[cfg(feature = "unstable-msc4319")]
            Self::Sync(event) => event.sender(),
        }
    }

    /// Returns this event's `state_key` field.
    pub fn state_key(&self) -> &str {
        match self {
            Self::Stripped(event) => event.state_key(),
            #[cfg(feature = "unstable-msc4319")]
            Self::Sync(event) => event.state_key(),
        }
    }

    /// Returns this event's `event_id` field, if there is one.
    pub fn event_id(&self) -> Option<&EventId> {
        match self {
            Self::Stripped(_) => None,
            #[cfg(feature = "unstable-msc4319")]
            Self::Sync(event) => Some(event.event_id()),
        }
    }

    /// Returns this event's `origin_server_ts` field, if there is one.
    pub fn origin_server_ts(&self) -> Option<MilliSecondsSinceUnixEpoch> {
        match self {
            Self::Stripped(_) => None,
            #[cfg(feature = "unstable-msc4319")]
            Self::Sync(event) => Some(event.origin_server_ts()),
        }
    }
}

impl<'de> Deserialize<'de> for StrippedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;

        #[cfg(feature = "unstable-msc4319")]
        {
            use serde::de;

            #[derive(Deserialize)]
            struct StrippedStateDeHelper {
                event_id: Option<de::IgnoredAny>,
                origin_server_ts: Option<de::IgnoredAny>,
            }

            let StrippedStateDeHelper { event_id, origin_server_ts } = from_raw_json_value(&json)?;

            if event_id.is_some() && origin_server_ts.is_some() {
                return from_raw_json_value(&json).map(Self::Sync);
            }
        }

        from_raw_json_value(&json).map(Self::Stripped)
    }
}

impl JsonCastable<StrippedState> for AnySyncStateEvent {}

impl JsonCastable<StrippedState> for AnyStrippedStateEvent {}

impl JsonCastable<AnyStrippedStateEvent> for StrippedState {}

impl JsonCastable<StrippedState> for AnyStateEvent {}

impl<C> JsonCastable<StrippedState> for OriginalStateEvent<C> where C: StaticStateEventContent {}

impl<C> JsonCastable<StrippedState> for OriginalSyncStateEvent<C> where C: StaticStateEventContent {}

impl<C> JsonCastable<StrippedState> for RedactedStateEvent<C> where C: RedactedStateEventContent {}

impl<C> JsonCastable<StrippedState> for RedactedSyncStateEvent<C> where C: RedactedStateEventContent {}

impl<C> JsonCastable<StrippedState> for StateEvent<C>
where
    C: StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
}

impl<C> JsonCastable<StrippedState> for SyncStateEvent<C>
where
    C: StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
}

impl<C> JsonCastable<StrippedState> for StrippedStateEvent<C> where
    C: PossiblyRedactedStateEventContent
{
}

impl JsonCastable<JsonObject> for StrippedState {}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::user_id;
    use ruma_events::{room::member::MembershipState, AnyStrippedStateEvent};
    use serde_json::{from_value as from_json_value, json};

    use crate::sync::sync_events::StrippedState;

    #[test]
    fn deserialize_stripped_state() {
        let user_id = user_id!("@patrick:localhost");
        let content = json!({
            "membership": "join",
        });

        // Stripped format.
        let stripped_event_json = json!({
            "content": content,
            "type": "m.room.member",
            "state_key": user_id,
            "sender": user_id,
        });
        assert_matches!(
            from_json_value::<StrippedState>(stripped_event_json).unwrap(),
            StrippedState::Stripped(AnyStrippedStateEvent::RoomMember(stripped_member_event))
        );
        assert_eq!(stripped_member_event.sender, user_id);
        assert_eq!(stripped_member_event.state_key, user_id);
        assert_eq!(stripped_member_event.content.membership, MembershipState::Join);

        #[cfg(feature = "unstable-msc4319")]
        {
            use js_int::uint;
            use ruma_common::event_id;
            use ruma_events::{AnySyncStateEvent, SyncStateEvent};

            let event_id = event_id!("$abcdefgh");

            // Sync format.
            let sync_event_json = json!({
                "content": content,
                "event_id": event_id,
                "origin_server_ts": 1_000_000,
                "sender": user_id,
                "state_key": user_id,
                "type": "m.room.member",
            });
            assert_matches!(
                from_json_value::<StrippedState>(sync_event_json).unwrap(),
                StrippedState::Sync(AnySyncStateEvent::RoomMember(SyncStateEvent::Original(
                    sync_member_event
                )))
            );
            assert_eq!(sync_member_event.content.membership, MembershipState::Join);
            assert_eq!(sync_member_event.event_id, event_id);
            assert_eq!(sync_member_event.origin_server_ts.0, uint!(1_000_000));
            assert_eq!(sync_member_event.sender, user_id);
            assert_eq!(sync_member_event.state_key, user_id);
        }
    }
}
