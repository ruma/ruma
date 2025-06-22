//! `GET /_matrix/client/*/thirdparty/location/{protocol}`
//!
//! Fetches third party locations for a protocol.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3thirdpartylocationprotocol

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Location,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/thirdparty/location/{protocol}",
            1.1 => "/_matrix/client/v3/thirdparty/location/{protocol}",
        }
    };

    /// Request type for the `get_location_for_protocol` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The protocol used to communicate to the third party network.
        #[ruma_api(path)]
        pub protocol: String,

        /// One or more custom fields to help identify the third party location.
        #[ruma_api(query_all)]
        pub fields: BTreeMap<String, String>,
    }

    /// Response type for the `get_location_for_protocol` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// List of matched third party locations.
        #[ruma_api(body)]
        pub locations: Vec<Location>,
    }

    impl Request {
        /// Creates a new `Request` with the given protocol.
        pub fn new(protocol: String) -> Self {
            Self { protocol, fields: BTreeMap::new() }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given locations.
        pub fn new(locations: Vec<Location>) -> Self {
            Self { locations }
        }
    }
}
