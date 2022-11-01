//! `GET /_matrix/app/*/rooms/{roomAlias}`
//!
//! Endpoint to query the existence of a given room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/application-service-api/#get_matrixappv1roomsroomalias

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomAliasId,
    };

    const METADATA: Metadata = metadata! {
        description: "This endpoint is invoked by the homeserver on an application service to query the existence of a given room alias.",
        method: GET,
        name: "query_room_alias",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/app/v1/rooms/:room_alias",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The room alias being queried.
        #[ruma_api(path)]
        pub room_alias: &'a RoomAliasId,
    }

    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias.
        pub fn new(room_alias: &'a RoomAliasId) -> Self {
            Self { room_alias }
        }
    }

    impl Response {
        /// Create an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
