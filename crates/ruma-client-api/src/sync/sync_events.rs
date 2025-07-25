//! `GET /_matrix/client/*/sync`
//!
//! Get all new events from all rooms since the last sync or a given point in time.

use js_int::UInt;
use ruma_common::{
    serde::{from_raw_json_value, JsonCastable, JsonObject},
    OwnedUserId, UserId,
};
use ruma_events::{
    AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, OriginalStateEvent,
    OriginalSyncStateEvent, PossiblyRedactedStateEventContent, RedactContent, RedactedStateEvent,
    RedactedStateEventContent, RedactedSyncStateEvent, StateEvent, StateEventType,
    StaticStateEventContent, StrippedStateEvent, SyncStateEvent,
};
use serde::{de, Deserialize, Serialize};
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

/// A state event that has either the sync or stripped format.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
#[allow(clippy::large_enum_variant)]
pub enum AnySyncOrStrippedStateEvent {
    /// An event using the sync format.
    Sync(AnySyncStateEvent),

    /// An event using the stripped format.
    Stripped(AnyStrippedStateEvent),
}

impl AnySyncOrStrippedStateEvent {
    /// Returns the `type` of this event.
    pub fn event_type(&self) -> StateEventType {
        match self {
            Self::Sync(event) => event.event_type(),
            Self::Stripped(event) => event.event_type(),
        }
    }

    /// Returns this event's `sender` field.
    pub fn sender(&self) -> &UserId {
        match self {
            Self::Sync(event) => event.sender(),
            Self::Stripped(event) => event.sender(),
        }
    }

    /// Returns this event's `state_key` field.
    pub fn state_key(&self) -> &str {
        match self {
            Self::Sync(event) => event.state_key(),
            Self::Stripped(event) => event.state_key(),
        }
    }
}

impl<'de> Deserialize<'de> for AnySyncOrStrippedStateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AnySyncOrStrippedStateEventDeHelper {
            event_id: Option<de::IgnoredAny>,
            origin_server_ts: Option<de::IgnoredAny>,
        }

        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let AnySyncOrStrippedStateEventDeHelper { event_id, origin_server_ts } =
            from_raw_json_value(&json)?;

        Ok(if event_id.is_some() && origin_server_ts.is_some() {
            Self::Sync(from_raw_json_value(&json)?)
        } else {
            Self::Stripped(from_raw_json_value(&json)?)
        })
    }
}

impl From<AnySyncStateEvent> for AnySyncOrStrippedStateEvent {
    fn from(value: AnySyncStateEvent) -> Self {
        Self::Sync(value)
    }
}

impl From<AnyStrippedStateEvent> for AnySyncOrStrippedStateEvent {
    fn from(value: AnyStrippedStateEvent) -> Self {
        Self::Stripped(value)
    }
}

impl JsonCastable<AnySyncOrStrippedStateEvent> for AnySyncStateEvent {}

impl JsonCastable<AnySyncOrStrippedStateEvent> for AnyStrippedStateEvent {}

impl JsonCastable<AnySyncOrStrippedStateEvent> for AnyStateEvent {}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for OriginalStateEvent<C> where
    C: StaticStateEventContent
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for OriginalSyncStateEvent<C> where
    C: StaticStateEventContent
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for RedactedStateEvent<C> where
    C: RedactedStateEventContent
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for RedactedSyncStateEvent<C> where
    C: RedactedStateEventContent
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for StateEvent<C>
where
    C: StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for SyncStateEvent<C>
where
    C: StaticStateEventContent + RedactContent,
    <C as RedactContent>::Redacted: RedactedStateEventContent,
{
}

impl<C> JsonCastable<AnySyncOrStrippedStateEvent> for StrippedStateEvent<C> where
    C: PossiblyRedactedStateEventContent
{
}

impl JsonCastable<JsonObject> for AnySyncOrStrippedStateEvent {}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{event_id, user_id};
    use ruma_events::{
        room::member::MembershipState, AnyStrippedStateEvent, AnySyncStateEvent, SyncStateEvent,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::AnySyncOrStrippedStateEvent;

    #[test]
    fn deserialize_any_sync_or_stripped_state_event() {
        let user_id = user_id!("@patrick:localhost");
        let event_id = event_id!("$abcdefgh");
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
            from_json_value::<AnySyncOrStrippedStateEvent>(stripped_event_json).unwrap(),
            AnySyncOrStrippedStateEvent::Stripped(AnyStrippedStateEvent::RoomMember(
                stripped_member_event
            ))
        );
        assert_eq!(stripped_member_event.sender, user_id);
        assert_eq!(stripped_member_event.state_key, user_id);
        assert_eq!(stripped_member_event.content.membership, MembershipState::Join);

        // Sync format.
        let sync_event_json = json!({
            "content": content,
            "type": "m.room.member",
            "state_key": user_id,
            "sender": user_id,
            "event_id": event_id,
            "origin_server_ts": 1_000_000,
        });
        assert_matches!(
            from_json_value::<AnySyncOrStrippedStateEvent>(sync_event_json).unwrap(),
            AnySyncOrStrippedStateEvent::Sync(AnySyncStateEvent::RoomMember(
                SyncStateEvent::Original(sync_member_event)
            ))
        );
        assert_eq!(sync_member_event.sender, user_id);
        assert_eq!(sync_member_event.state_key, user_id);
        assert_eq!(sync_member_event.event_id, event_id);
        assert_eq!(sync_member_event.origin_server_ts.0, uint!(1_000_000));
        assert_eq!(sync_member_event.content.membership, MembershipState::Join);
    }
}
