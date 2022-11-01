//! `PUT /_matrix/client/*/directory/room/{roomAlias}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3directoryroomroomalias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomAliasId, RoomId,
    };

    const METADATA: Metadata = metadata! {
        description: "Add an alias to a room.",
        method: PUT,
        name: "create_alias",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/room/:room_alias",
            1.1 => "/_matrix/client/v3/directory/room/:room_alias",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The room alias to set.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,

        /// The room ID to set.
        pub room_id: &'a RoomId,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias and room id.
        pub fn new(room_alias: &'a RoomAliasId, room_id: &'a RoomId) -> Self {
            Self { room_alias, room_id }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
