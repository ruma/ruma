//! `GET /_matrix/client/*/rooms/{roomId}/aliases`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3roomsroomidaliases

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomAliasId, RoomId,
    };

    const METADATA: Metadata = metadata! {
        description: "Get a list of aliases maintained by the local server for the given room.",
        method: GET,
        name: "aliases",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc2432/rooms/:room_id/aliases",
            1.0 => "/_matrix/client/r0/rooms/:room_id/aliases",
            1.1 => "/_matrix/client/v3/rooms/:room_id/aliases",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The room ID to get aliases of.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// The server's local aliases on the room.
        pub aliases: Vec<OwnedRoomAliasId>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given aliases.
        pub fn new(aliases: Vec<OwnedRoomAliasId>) -> Self {
            Self { aliases }
        }
    }
}
