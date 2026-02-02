//! `POST /_matrix/client/*/createRoom`
//!
//! Create a new room.

use std::collections::BTreeMap;

use js_int::Int;
use ruma_common::{
    OwnedUserId,
    power_levels::NotificationPowerLevels,
    serde::{JsonCastable, JsonObject},
};
use ruma_events::{TimelineEventType, room::power_levels::RoomPowerLevelsEventContent};
use serde::Serialize;

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3createroom

    use assign::assign;
    use ruma_common::{
        OwnedRoomId, OwnedUserId, RoomVersionId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        room::RoomType,
        serde::{Raw, StringEnum},
    };
    use ruma_events::{
        AnyInitialStateEvent,
        room::create::{PreviousRoom, RoomCreateEventContent},
    };
    use serde::{Deserialize, Serialize};

    use super::RoomPowerLevelsContentOverride;
    use crate::{PrivOwnedStr, membership::Invite3pid, room::Visibility};

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/createRoom",
            1.1 => "/_matrix/client/v3/createRoom",
        }
    }

    /// Request type for the `create_room` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {
        /// Extra keys to be added to the content of the `m.room.create`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub creation_content: Option<Raw<CreationContent>>,

        /// List of state events to send to the new room.
        ///
        /// Takes precedence over events set by preset, but gets overridden by name and topic keys.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub initial_state: Vec<Raw<AnyInitialStateEvent>>,

        /// A list of user IDs to invite to the room.
        ///
        /// This will tell the server to invite everyone in the list to the newly created room.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub invite: Vec<OwnedUserId>,

        /// List of third party IDs of users to invite.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub invite_3pid: Vec<Invite3pid>,

        /// If set, this sets the `is_direct` flag on room invites.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub is_direct: bool,

        /// If this is included, an `m.room.name` event will be sent into the room to indicate the
        /// name of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// Power level content to override in the default power level event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub power_level_content_override: Option<Raw<RoomPowerLevelsContentOverride>>,

        /// Convenience parameter for setting various default state events based on a preset.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub preset: Option<RoomPreset>,

        /// The desired room alias local part.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias_name: Option<String>,

        /// Room version to set for the room.
        ///
        /// Defaults to homeserver's default if not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_version: Option<RoomVersionId>,

        /// If this is included, an `m.room.topic` event will be sent into the room to indicate
        /// the topic for the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,

        /// A public visibility indicates that the room will be shown in the published room list.
        ///
        /// A private visibility will hide the room from the published room list. Defaults to
        /// `Private`.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub visibility: Visibility,
    }

    /// Response type for the `create_room` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The created room's ID.
        pub room_id: OwnedRoomId,
    }

    impl Request {
        /// Creates a new `Request` will all-default parameters.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }

    /// Extra options to be added to the `m.room.create` event.
    ///
    /// This is the same as the event content struct for `m.room.create`, but without some fields
    /// that servers are supposed to ignore.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct CreationContent {
        /// A list of user IDs to consider as additional creators, and hence grant an "infinite"
        /// immutable power level, from room version 12 onwards.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub additional_creators: Vec<OwnedUserId>,

        /// Whether users on other servers can join this room.
        ///
        /// Defaults to `true` if key does not exist.
        #[serde(
            rename = "m.federate",
            default = "ruma_common::serde::default_true",
            skip_serializing_if = "ruma_common::serde::is_true"
        )]
        pub federate: bool,

        /// A reference to the room this room replaces, if the previous room was upgraded.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub predecessor: Option<PreviousRoom>,

        /// The room type.
        ///
        /// This is currently only used for spaces.
        #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
        pub room_type: Option<RoomType>,
    }

    impl CreationContent {
        /// Creates a new `CreationContent` with all fields defaulted.
        pub fn new() -> Self {
            Self {
                additional_creators: Vec::new(),
                federate: true,
                predecessor: None,
                room_type: None,
            }
        }

        /// Given a `CreationContent` and the other fields that a homeserver has to fill, construct
        /// a `RoomCreateEventContent`.
        pub fn into_event_content(
            self,
            creator: OwnedUserId,
            room_version: RoomVersionId,
        ) -> RoomCreateEventContent {
            assign!(RoomCreateEventContent::new_v1(creator), {
                federate: self.federate,
                room_version: room_version,
                predecessor: self.predecessor,
                room_type: self.room_type
            })
        }

        /// Returns whether all fields have their default value.
        pub fn is_empty(&self) -> bool {
            self.federate && self.predecessor.is_none() && self.room_type.is_none()
        }
    }

    impl Default for CreationContent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// A convenience parameter for setting a few default state events.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum RoomPreset {
        /// `join_rules` is set to `invite` and `history_visibility` is set to `shared`.
        PrivateChat,

        /// `join_rules` is set to `public` and `history_visibility` is set to `shared`.
        PublicChat,

        /// Same as `PrivateChat`, but all initial invitees get the same power level as the
        /// creator.
        TrustedPrivateChat,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}

/// The power level values that can be overridden when creating a room.
///
/// This has the same fields as [`RoomPowerLevelsEventContent`], but most of them are `Option`s.
/// Contrary to [`RoomPowerLevelsEventContent`] which doesn't serialize fields that are set to their
/// default value defined in the Matrix specification, this type serializes all fields that are
/// `Some(_)`, regardless of their value.
///
/// This type is used to allow clients to avoid server behavior observed in the wild that sets
/// custom default values for fields that are not set in the `create_room` request, while a client
/// wants the server to use the default value defined in the Matrix specification for that field.
#[derive(Clone, Debug, Serialize, Default)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RoomPowerLevelsContentOverride {
    /// The level required to ban a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban: Option<Int>,

    /// The level required to send specific event types.
    ///
    /// This is a mapping from event type to power level required.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub events: BTreeMap<TimelineEventType, Int>,

    /// The default level required to send message events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_default: Option<Int>,

    /// The level required to invite a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite: Option<Int>,

    /// The level required to kick a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kick: Option<Int>,

    /// The level required to redact an event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redact: Option<Int>,

    /// The default level required to send state events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_default: Option<Int>,

    /// The power levels for specific users.
    ///
    /// This is a mapping from `user_id` to power level for that user.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub users: BTreeMap<OwnedUserId, Int>,

    /// The default power level for every user in the room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users_default: Option<Int>,

    /// The power level requirements for specific notification types.
    ///
    /// This is a mapping from `key` to power level for that notifications key.
    #[serde(default, skip_serializing_if = "NotificationPowerLevels::is_default")]
    pub notifications: NotificationPowerLevels,
}

impl RoomPowerLevelsContentOverride {
    /// Creates a new, empty [`RoomPowerLevelsContentOverride`] instance.
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<RoomPowerLevelsEventContent> for RoomPowerLevelsContentOverride {
    fn from(value: RoomPowerLevelsEventContent) -> Self {
        let RoomPowerLevelsEventContent {
            ban,
            events,
            events_default,
            invite,
            kick,
            redact,
            state_default,
            users,
            users_default,
            notifications,
            ..
        } = value;

        Self {
            ban: Some(ban),
            events,
            events_default: Some(events_default),
            invite: Some(invite),
            kick: Some(kick),
            redact: Some(redact),
            state_default: Some(state_default),
            users,
            users_default: Some(users_default),
            notifications,
        }
    }
}

impl JsonCastable<RoomPowerLevelsEventContent> for RoomPowerLevelsContentOverride {}

impl JsonCastable<RoomPowerLevelsContentOverride> for RoomPowerLevelsEventContent {}

impl JsonCastable<JsonObject> for RoomPowerLevelsContentOverride {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use assign::assign;
    use js_int::int;
    use maplit::btreemap;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, power_levels::NotificationPowerLevels, user_id,
    };
    use serde_json::json;

    use super::RoomPowerLevelsContentOverride;

    #[test]
    fn serialization_of_power_levels_overridden_values_with_optional_fields_as_none() {
        let power_levels = RoomPowerLevelsContentOverride {
            ban: None,
            events: BTreeMap::new(),
            events_default: None,
            invite: None,
            kick: None,
            redact: None,
            state_default: None,
            users: BTreeMap::new(),
            users_default: None,
            notifications: NotificationPowerLevels::default(),
        };

        assert_to_canonical_json_eq!(power_levels, json!({}));
    }

    #[test]
    fn serialization_of_power_levels_overridden_values_with_all_fields() {
        let user = user_id!("@carl:example.com");
        let power_levels_event = RoomPowerLevelsContentOverride {
            ban: Some(int!(23)),
            events: btreemap! {
                "m.dummy".into() => int!(23)
            },
            events_default: Some(int!(23)),
            invite: Some(int!(23)),
            kick: Some(int!(23)),
            redact: Some(int!(23)),
            state_default: Some(int!(23)),
            users: btreemap! {
                user.to_owned() => int!(23)
            },
            users_default: Some(int!(23)),
            notifications: assign!(NotificationPowerLevels::new(), { room: int!(23) }),
        };

        assert_to_canonical_json_eq!(
            power_levels_event,
            json!({
                "ban": 23,
                "events": {
                    "m.dummy": 23
                },
                "events_default": 23,
                "invite": 23,
                "kick": 23,
                "redact": 23,
                "state_default": 23,
                "users": {
                    "@carl:example.com": 23
                },
                "users_default": 23,
                "notifications": {
                    "room": 23
                },
            })
        );
    }
}
