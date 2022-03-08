//! `POST /_matrix/client/*/createRoom`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3createroom

    use assign::assign;
    use ruma_common::{
        api::ruma_api,
        events::{
            room::{
                create::{PreviousRoom, RoomCreateEventContent},
                power_levels::RoomPowerLevelsEventContent,
            },
            AnyInitialStateEvent,
        },
        room::RoomType,
        RoomId, RoomName, RoomVersionId, UserId,
    };
    use ruma_serde::{Raw, StringEnum};
    use serde::{Deserialize, Serialize};

    use crate::{
        membership::{IncomingInvite3pid, Invite3pid},
        room::Visibility,
        PrivOwnedStr,
    };

    ruma_api! {
        metadata: {
            description: "Create a new room.",
            method: POST,
            name: "create_room",
            r0_path: "/_matrix/client/r0/createRoom",
            stable_path: "/_matrix/client/v3/createRoom",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// Extra keys to be added to the content of the `m.room.create`.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub creation_content: Option<Raw<CreationContent>>,

            /// List of state events to send to the new room.
            ///
            /// Takes precedence over events set by preset, but gets overridden by name and topic keys.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub initial_state: &'a [Raw<AnyInitialStateEvent>],

            /// A list of user IDs to invite to the room.
            ///
            /// This will tell the server to invite everyone in the list to the newly created room.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub invite: &'a [Box<UserId>],

            /// List of third party IDs of users to invite.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub invite_3pid: &'a [Invite3pid<'a>],

            /// If set, this sets the `is_direct` flag on room invites.
            #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
            pub is_direct: bool,

            /// If this is included, an `m.room.name` event will be sent into the room to indicate the
            /// name of the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub name: Option<&'a RoomName>,

            /// Power level content to override in the default power level event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub power_level_content_override: Option<Raw<RoomPowerLevelsEventContent>>,

            /// Convenience parameter for setting various default state events based on a preset.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub preset: Option<RoomPreset>,

            /// The desired room alias local part.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_alias_name: Option<&'a str>,

            /// Room version to set for the room.
            ///
            /// Defaults to homeserver's default if not specified.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_version: Option<&'a RoomVersionId>,

            /// If this is included, an `m.room.topic` event will be sent into the room to indicate
            /// the topic for the room.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub topic: Option<&'a str>,

            /// A public visibility indicates that the room will be shown in the published room list.
            ///
            /// A private visibility will hide the room from the published room list. Defaults to
            /// `Private`.
            #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
            pub visibility: Visibility,
        }

        response: {
            /// The created room's ID.
            pub room_id: Box<RoomId>,
        }

        error: crate::Error
    }

    impl Request<'_> {
        /// Creates a new `Request` will all-default parameters.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room id.
        pub fn new(room_id: Box<RoomId>) -> Self {
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
            default = "ruma_serde::default_true",
            skip_serializing_if = "ruma_serde::is_true"
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
            creator: Box<UserId>,
            room_version: RoomVersionId,
        ) -> RoomCreateEventContent {
            assign!(RoomCreateEventContent::new(creator), {
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
    ///
    /// This type can hold an arbitrary string. To check for formats that are not available as a
    /// documented variant here, use its string representation, obtained through `.as_str()`.
    #[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
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

    impl RoomPreset {
        /// Creates a string slice from this `RoomPreset`.
        pub fn as_str(&self) -> &str {
            self.as_ref()
        }
    }
}
