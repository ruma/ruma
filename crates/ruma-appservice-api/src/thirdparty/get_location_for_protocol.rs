//! `GET /_matrix/app/*/thirdparty/location/{protocol}`
//!
//! Endpoint to retrieve a list of Matrix portal rooms that lead to the matched third party
//! location.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#get_matrixappv1thirdpartylocationprotocol

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, thirdparty::Location};

    ruma_api! {
        metadata: {
            description: "Fetches third party locations for a protocol.",
            method: GET,
            name: "get_location_for_protocol",
            stable_path: "/_matrix/app/v1/thirdparty/location/:protocol",
            rate_limited: false,
            authentication: QueryOnlyAccessToken,
            added: 1.0,
        }

        request: {
            /// The protocol used to communicate to the third party network.
            #[ruma_api(path)]
            pub protocol: &'a str,

            /// One or more custom fields to help identify the third party location.
            // The specification is incorrect for this parameter. See [matrix-spec#560](https://github.com/matrix-org/matrix-spec/issues/560).
            #[ruma_api(query_map)]
            pub fields: BTreeMap<String, String>,
        }

        response: {
            /// List of matched third party locations.
            #[ruma_api(body)]
            pub locations: Vec<Location>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given protocol.
        pub fn new(protocol: &'a str) -> Self {
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
