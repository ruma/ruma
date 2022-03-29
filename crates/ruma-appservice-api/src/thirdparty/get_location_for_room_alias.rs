//! `GET /_matrix/app/*/thirdparty/location`
//!
//! Endpoint to retrieve an array of third party network locations from a Matrix room alias.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1thirdpartylocation

    use ruma_common::{api::ruma_api, thirdparty::Location, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Retrieve an array of third party network locations from a Matrix room alias.",
            method: GET,
            name: "get_location_for_room_alias",
            stable_path: "/_matrix/app/v1/thirdparty/location",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The Matrix room alias to look up.
            #[ruma_api(query)]
            pub alias: &'a RoomAliasId,
        }

        response: {
            /// List of matched third party locations.
            #[ruma_api(body)]
            pub locations: Vec<Location>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias id.
        pub fn new(alias: &'a RoomAliasId) -> Self {
            Self { alias }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given locations.
        pub fn new(locations: Vec<Location>) -> Self {
            Self { locations }
        }
    }
}
