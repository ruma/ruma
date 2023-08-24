//! `POST /_matrix/client/*/createRoom`
//!
//! Create a new room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3createroom

    use assign::assign;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        room::RoomType,
        serde::{Raw, StringEnum},
        OwnedRoomId, OwnedUserId, RoomVersionId,
    };
    use ruma_events::{
        room::{
            create::{PreviousRoom, RoomCreateEventContent},
            power_levels::RoomPowerLevelsEventContent,
        },
        AnyInitialStateEvent,
    };
    use serde::{Deserialize, Serialize};

    use crate::{membership::Invite3pid, room::Visibility, PrivOwnedStr};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/createRoom",
            1.1 => "/_matrix/client/v3/createRoom",
        }
    };

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
        pub power_level_content_override: Option<Raw<RoomPowerLevelsEventContent>>,

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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct CreationContent {
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
            Self { federate: true, predecessor: None, room_type: None }
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
    #[derive(Clone, PartialEq, Eq, StringEnum)]
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
