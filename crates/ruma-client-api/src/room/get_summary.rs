//! `GET /_matrix/client/v1/summary/{roomIdOrAlias}`
//!
//! Experimental API enabled with MSC3266.
//!
//! Returns a short description of the state of a room, with state events.

pub mod msc3266 {
    //! `MSC3266` ([MSC])
    //!
    //! [MSC]: https://github.com/deepbluev7/matrix-doc/blob/room-summaries/proposals/3266-room-summary.md

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

    /// Request type for the `summary/room_id_or_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Alias or ID of the room to be summarized.
        #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// An optional list of servers the invited homeserver should attempt to peek at the room.
        #[serde(default)]
        #[ruma_api(query)]
        pub via: Vec<OwnedServerName>,
    }

    /// Response type for the `summary/room_id_or_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// ID of the room (useful if it's an alias).
        pub room_id: OwnedRoomId,

        /// A canonical alias for this room, if set.
        pub canonical_alias: Option<OwnedRoomAliasId>,

        /// Avatar of the room.
        pub avatar_url: Option<OwnedMxcUri>,

        /// Whether guests can join the room.
        pub guest_can_join: bool,

        /// Name of the room.
        pub name: Option<String>,

        /// Member count of the room.
        pub num_joined_members: u64,

        /// Topic of the room.
        pub topic: Option<String>,

        /// Whether the room history can be read without joining.
        pub world_readable: bool,

        /// Join rule of the room.
        ///
        /// Technically, just a `JoinRule`, but without the extra info for restricted variants.
        pub join_rule: SpaceRoomJoinRule,

        /// Type of the room, if any.
        pub room_type: Option<RoomType>,

        /// Version of the room.
        pub room_version: Option<RoomVersionId>,

        /// The current membership of this user in the room. Usually `leave` if the room is fetched
        /// over federation.
        ///
        /// This field will not be present when called unauthenticated, but is required when called
        /// authenticated. It should be `leave` if the serer doesn't know about the room, since for
        /// all other membership states the server would know about the room already.
        pub membership: Option<MembershipState>,

        /// If the room is encrypted, this specified the algorithm used for this room.
        pub encryption: Option<EventEncryptionAlgorithm>,
    }

    impl Request {
        /// Creates a new `Request` with the given room or alias ID and via server names.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId, via: Vec<OwnedServerName>) -> Self {
            Self { room_id_or_alias, via }
        }
    }
}
