//! Types to deserialize `m.room.create` events.

use std::{borrow::Cow, ops::Deref};

use ruma_common::{
    room_version_rules::AuthorizationRules, serde::from_raw_json_value, OwnedUserId, RoomVersionId,
    UserId,
};
use serde::{de::IgnoredAny, Deserialize};

use super::Event;

/// A helper type for an [`Event`] of type `m.room.create`.
///
/// This is a type that deserializes each field lazily, when requested.
#[derive(Debug, Clone)]
pub struct RoomCreateEvent<E: Event>(E);

impl<E: Event> RoomCreateEvent<E> {
    /// Construct a new `RoomCreateEvent` around the given event.
    pub fn new(event: E) -> Self {
        Self(event)
    }

    /// The version of the room.
    pub fn room_version(&self) -> Result<RoomVersionId, String> {
        #[derive(Deserialize)]
        struct RoomCreateContentRoomVersion {
            room_version: Option<RoomVersionId>,
        }

        let content: RoomCreateContentRoomVersion =
            from_raw_json_value(self.content()).map_err(|err: serde_json::Error| {
                format!("invalid `room_version` field in `m.room.create` event: {err}")
            })?;
        Ok(content.room_version.unwrap_or(RoomVersionId::V1))
    }

    /// Whether the room is federated.
    pub fn federate(&self) -> Result<bool, String> {
        #[derive(Deserialize)]
        struct RoomCreateContentFederate {
            #[serde(rename = "m.federate")]
            federate: Option<bool>,
        }

        let content: RoomCreateContentFederate =
            from_raw_json_value(self.content()).map_err(|err: serde_json::Error| {
                format!("invalid `m.federate` field in `m.room.create` event: {err}")
            })?;
        Ok(content.federate.unwrap_or(true))
    }

    /// The creator of the room.
    ///
    /// If the `use_room_create_sender` field of `AuthorizationRules` is set, the creator is the
    /// sender of this `m.room.create` event, otherwise it is deserialized from the `creator`
    /// field of this event's content.
    pub fn creator(&self, rules: &AuthorizationRules) -> Result<Cow<'_, UserId>, String> {
        #[derive(Deserialize)]
        struct RoomCreateContentCreator {
            creator: OwnedUserId,
        }

        if rules.use_room_create_sender {
            Ok(Cow::Borrowed(self.sender()))
        } else {
            let content: RoomCreateContentCreator =
                from_raw_json_value(self.content()).map_err(|err: serde_json::Error| {
                    format!("missing or invalid `creator` field in `m.room.create` event: {err}")
                })?;

            Ok(Cow::Owned(content.creator))
        }
    }

    /// Whether the event has a `creator` field.
    pub(crate) fn has_creator(&self) -> Result<bool, String> {
        #[derive(Deserialize)]
        struct RoomCreateContentCreator {
            creator: Option<IgnoredAny>,
        }

        let content: RoomCreateContentCreator =
            from_raw_json_value(self.content()).map_err(|err: serde_json::Error| {
                format!("invalid `creator` field in `m.room.create` event: {err}")
            })?;
        Ok(content.creator.is_some())
    }
}

impl<E: Event> Deref for RoomCreateEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
