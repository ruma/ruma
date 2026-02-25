//! `GET /_matrix/app/*/rooms/{roomAlias}`
//!
//! Endpoint to query the existence of a given room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#get_matrixappv1roomsroomalias

    use ruma_common::{
        RoomAliasId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        path: "/_matrix/app/v1/rooms/{room_alias}",
    }

    /// Request type for the `query_room_alias` endpoint.
    #[request]
    pub struct Request {
        /// The room alias being queried.
        #[ruma_api(path)]
        pub room_alias: RoomAliasId,
    }

    /// Response type for the `query_room_alias` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room alias.
        pub fn new(room_alias: RoomAliasId) -> Self {
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
