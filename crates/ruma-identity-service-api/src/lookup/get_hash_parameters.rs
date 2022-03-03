//! `GET /_matrix/identity/*/hash_details`
//!
//! Endpoint to get details about Hashing identifiers.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityv2hash_details

    use ruma_common::api::ruma_api;

    use crate::lookup::IdentifierHashingAlgorithm;

    ruma_api! {
        metadata: {
            description: "Gets parameters for hashing identifiers from the server. This can include any of the algorithms defined in the spec.",
            method: GET,
            name: "get_hash_parameters",
            stable_path: "/_matrix/identity/v2/hash_details",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The pepper the client MUST use in hashing identifiers, and MUST supply to the /lookup endpoint when performing lookups.
            ///
            /// Servers SHOULD rotate this string often.
            pub lookup_pepper: String,

            /// The algorithms the server supports.
            ///
            /// Must contain at least `sha256`.
            pub algorithms: Vec<IdentifierHashingAlgorithm>,
        }
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Create a new `Response` using the given pepper and `Vec` of algorithms.
        pub fn new(lookup_pepper: String, algorithms: Vec<IdentifierHashingAlgorithm>) -> Self {
            Self { lookup_pepper, algorithms }
        }
    }
}
