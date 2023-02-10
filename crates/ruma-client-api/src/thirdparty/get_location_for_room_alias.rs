//! `GET /_matrix/client/*/thirdparty/location`
//!
//! Retrieve an array of third party network locations from a Matrix room alias.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3thirdpartylocation

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Location,
        OwnedRoomAliasId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/location",
            1.1 => "/_matrix/client/v3/thirdparty/location",
        }
    };

    /// Request type for the `get_location_for_room_alias` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The Matrix room alias to look up.
        #[ruma_api(query)]
        pub alias: OwnedRoomAliasId,
    }

    /// Response type for the `get_location_for_room_alias` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
    }

    impl Request {
        /// Creates a new `Request` with the given room alias ID.
        pub fn new(alias: OwnedRoomAliasId) -> Self {
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
