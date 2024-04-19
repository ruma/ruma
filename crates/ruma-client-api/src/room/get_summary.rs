//! `GET /_matrix/client/v1/summary/{roomIdOrAlias}`
//!
//! Experimental API enabled with MSC3266.
//!
//! Returns a short description of the state of a room.

pub mod msc3266 {
    //! `MSC3266` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3266

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        room::RoomType,
        space::SpaceRoomJoinRule,
        EventEncryptionAlgorithm, OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, OwnedRoomOrAliasId,
        OwnedServerName, RoomVersionId,
    };
    use ruma_events::room::member::MembershipState;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessTokenOptional,
        history: {
            unstable => "/_matrix/client/unstable/im.nheko.summary/rooms/:room_id_or_alias/summary",
        }
    };

    /// Request type for the `get_summary` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Alias or ID of the room to be summarized.
        #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// A list of servers the homeserver should attempt to use to peek at the room.
        ///
        /// Defaults to an empty `Vec`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ruma_api(query)]
        pub via: Vec<OwnedServerName>,
    }

    /// Response type for the `get_summary` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// ID of the room (useful if it's an alias).
        pub room_id: OwnedRoomId,

        /// The canonical alias for this room, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub canonical_alias: Option<OwnedRoomAliasId>,

        /// Avatar of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<OwnedMxcUri>,

        /// Whether guests can join the room.
        pub guest_can_join: bool,

        /// Name of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// Member count of the room.
        pub num_joined_members: UInt,

        /// Topic of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,

        /// Whether the room history can be read without joining.
        pub world_readable: bool,

        /// Join rule of the room.
        pub join_rule: SpaceRoomJoinRule,

        /// Type of the room, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_type: Option<RoomType>,

        /// Version of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_version: Option<RoomVersionId>,

        /// The current membership of this user in the room.
        ///
        /// This field will not be present when called unauthenticated, but is required when called
        /// authenticated. It should be `leave` if the server doesn't know about the room, since
        /// for all other membership states the server would know about the room already.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub membership: Option<MembershipState>,

        /// If the room is encrypted, the algorithm used for this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub encryption: Option<EventEncryptionAlgorithm>,
    }

    impl Request {
        /// Creates a new `Request` with the given room or alias ID and via server names.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId, via: Vec<OwnedServerName>) -> Self {
            Self { room_id_or_alias, via }
        }
    }

    impl Response {
        /// Creates a new [`Response`] with all the mandatory fields set.
        pub fn new(
            room_id: OwnedRoomId,
            join_rule: SpaceRoomJoinRule,
            guest_can_join: bool,
            num_joined_members: UInt,
            world_readable: bool,
        ) -> Self {
            Self {
                room_id,
                canonical_alias: None,
                avatar_url: None,
                guest_can_join,
                name: None,
                num_joined_members,
                topic: None,
                world_readable,
                join_rule,
                room_type: None,
                room_version: None,
                membership: None,
                encryption: None,
            }
        }
    }
}
