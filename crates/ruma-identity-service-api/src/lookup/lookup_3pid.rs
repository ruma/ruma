//! `POST /_matrix/identity/*/lookup`
//!
//! Endpoint to look up set of Matrix IDs which are bound to 3PIDs.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/identity-service-api/#post_matrixidentityv2lookup

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    use crate::lookup::IdentifierHashingAlgorithm;

    const METADATA: Metadata = metadata! {
        description: "Looks up the set of Matrix User IDs which have bound the 3PIDs given, if bindings are available.",
        method: POST,
        name: "lookup_3pid",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/lookup",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The algorithm the client is using to encode the `addresses`. This should be one of the
        /// available options from `/hash_details`.
        pub algorithm: &'a IdentifierHashingAlgorithm,

        /// The pepper from `/hash_details`. This is required even when the `algorithm` does not
        /// make use of it.
        pub pepper: &'a str,

        /// The addresses to look up.
        ///
        /// The format of the entries here depend on the `algorithm` used. Note that queries which
        /// have been incorrectly hashed or formatted will lead to no matches.
        pub addresses: &'a [String],
    }

    #[response]
    pub struct Response {
        /// Any applicable mappings of `addresses` to Matrix User IDs.
        ///
        /// Addresses which do not have associations will not be included, which can make this
        /// property be an empty object.
        pub mappings: BTreeMap<String, OwnedUserId>,
    }

    impl<'a> Request<'a> {
        /// Create a `Request` with algorithm, pepper and addresses to loop up.
        pub fn new(
            algorithm: &'a IdentifierHashingAlgorithm,
            pepper: &'a str,
            addresses: &'a [String],
        ) -> Self {
            Self { algorithm, pepper, addresses }
        }
    }

    impl Response {
        /// Create a `Response` with the BTreeMap which map addresses from the request which were
        /// found to their corresponding User IDs.
        pub fn new(mappings: BTreeMap<String, OwnedUserId>) -> Self {
            Self { mappings }
        }
    }
}
