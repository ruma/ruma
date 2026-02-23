//! `POST /_matrix/identity/*/lookup`
//!
//! Looks up the set of Matrix User IDs which have bound the 3PIDs given, if bindings are available.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv2lookup

    use std::collections::BTreeMap;

    use ruma_common::{
        UserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::lookup::IdentifierHashingAlgorithm;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/lookup",
        }
    }

    /// Request type for the `lookup_3pid` endpoint.
    #[request]
    pub struct Request {
        /// The algorithm the client is using to encode the `addresses`. This should be one of the
        /// available options from `/hash_details`.
        pub algorithm: IdentifierHashingAlgorithm,

        /// The pepper from `/hash_details`. This is required even when the `algorithm` does not
        /// make use of it.
        pub pepper: String,

        /// The addresses to look up.
        ///
        /// The format of the entries here depend on the `algorithm` used. Note that queries which
        /// have been incorrectly hashed or formatted will lead to no matches.
        pub addresses: Vec<String>,
    }

    /// Response type for the `lookup_3pid` endpoint.
    #[response]
    pub struct Response {
        /// Any applicable mappings of `addresses` to Matrix User IDs.
        ///
        /// Addresses which do not have associations will not be included, which can make this
        /// property be an empty object.
        pub mappings: BTreeMap<String, UserId>,
    }

    impl Request {
        /// Create a `Request` with algorithm, pepper and addresses to loop up.
        pub fn new(
            algorithm: IdentifierHashingAlgorithm,
            pepper: String,
            addresses: Vec<String>,
        ) -> Self {
            Self { algorithm, pepper, addresses }
        }
    }

    impl Response {
        /// Create a `Response` with the BTreeMap which map addresses from the request which were
        /// found to their corresponding User IDs.
        pub fn new(mappings: BTreeMap<String, UserId>) -> Self {
            Self { mappings }
        }
    }
}
