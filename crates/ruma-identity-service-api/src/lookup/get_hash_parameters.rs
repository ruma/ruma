//! `GET /_matrix/identity/*/hash_details`
//!
//! Gets parameters for hashing identifiers from the server. This can include any of the algorithms
//! defined in the spec.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#get_matrixidentityv2hash_details

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::lookup::IdentifierHashingAlgorithm;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/hash_details",
        }
    }

    /// Request type for the `get_hash_parameters` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_hash_parameters` endpoint.
    #[response]
    pub struct Response {
        /// The pepper the client MUST use in hashing identifiers, and MUST supply to the /lookup
        /// endpoint when performing lookups.
        ///
        /// Servers SHOULD rotate this string often.
        pub lookup_pepper: String,

        /// The algorithms the server supports.
        ///
        /// Must contain at least `sha256`.
        pub algorithms: Vec<IdentifierHashingAlgorithm>,
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
