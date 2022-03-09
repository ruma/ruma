//! `GET /_matrix/app/*/rooms/{roomAlias}`
//!
//! Endpoint to query the existence of a given room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1roomsroomalias

    use ruma_common::{api::ruma_api, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "This endpoint is invoked by the homeserver on an application service to query the existence of a given room alias.",
            method: GET,
            name: "query_room_alias",
            stable_path: "/_matrix/app/v1/rooms/:room_alias",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The room alias being queried.
            #[ruma_api(path)]
            pub room_alias: &'a RoomAliasId,
        }

        #[derive(Default)]
        response: {}
    }

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
