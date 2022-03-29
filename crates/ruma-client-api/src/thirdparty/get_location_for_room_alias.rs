//! `GET /_matrix/client/*/thirdparty/location`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3thirdpartylocation

    use ruma_common::{api::ruma_api, thirdparty::Location, RoomAliasId};

    ruma_api! {
        metadata: {
            description: "Retrieve an array of third party network locations from a Matrix room alias.",
            method: GET,
            name: "get_location_for_room_alias",
            r0_path: "/_matrix/client/r0/thirdparty/location",
            stable_path: "/_matrix/client/v3/thirdparty/location",
            rate_limited: false,
            authentication: AccessToken,
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

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room alias ID.
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
