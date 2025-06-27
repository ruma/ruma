//! `GET /_matrix/client/v1/summary/{roomIdOrAlias}`
//!
//! Returns a short description of the state of a room.

pub mod v1 {
    //! `v1` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1room_summaryroomidoralias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        room::RoomSummary,
        OwnedRoomOrAliasId, OwnedServerName,
    };
    use ruma_events::room::member::MembershipState;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessTokenOptional,
        history: {
            unstable => "/_matrix/client/unstable/im.nheko.summary/rooms/{room_id_or_alias}/summary",
            1.15 => "/_matrix/client/v1/room_summary/{room_id_or_alias}",
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
        /// The summary of the room.
        #[serde(flatten)]
        pub summary: RoomSummary,

        /// The current membership of this user in the room.
        ///
        /// This field will not be present when called unauthenticated, but is required when called
        /// authenticated. It should be `leave` if the server doesn't know about the room, since
        /// for all other membership states the server would know about the room already.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub membership: Option<MembershipState>,
    }

    impl Request {
        /// Creates a new `Request` with the given room or alias ID and via server names.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId, via: Vec<OwnedServerName>) -> Self {
            Self { room_id_or_alias, via }
        }
    }

    impl Response {
        /// Creates a new [`Response`] with the given summary.
        pub fn new(summary: RoomSummary) -> Self {
            Self { summary, membership: None }
        }
    }

    impl From<RoomSummary> for Response {
        fn from(value: RoomSummary) -> Self {
            Self::new(value)
        }
    }
}
