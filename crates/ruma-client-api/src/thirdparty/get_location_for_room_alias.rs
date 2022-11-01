//! `GET /_matrix/client/*/thirdparty/location`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3thirdpartylocation

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Location,
        RoomAliasId,
    };

    const METADATA: Metadata = metadata! {
        description: "Retrieve an array of third party network locations from a Matrix room alias.",
        method: GET,
        name: "get_location_for_room_alias",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/location",
            1.1 => "/_matrix/client/v3/thirdparty/location",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The Matrix room alias to look up.
        #[ruma_api(query)]
        pub alias: &'a RoomAliasId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
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
