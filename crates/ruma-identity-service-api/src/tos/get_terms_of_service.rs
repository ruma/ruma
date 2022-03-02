//! `GET /_matrix/identity/*/terms`
//!
//! Endpoint to retrieve the terms of service of an identity server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#get_matrixidentityv2terms

    use std::collections::BTreeMap;

    use ruma_common::api::ruma_api;
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Gets all the terms of service offered by the server.",
            method: GET,
            name: "get_terms_of_service",
            stable_path: "/_matrix/identity/v2/terms",
            authentication: None,
            rate_limited: false,
            added: 1.0,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The policies the server offers.
            ///
            /// Mapped from arbitrary ID (unused in this version of the specification) to a Policy Object.
            pub policies: BTreeMap<String, Policies>,
        }
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given `Policies`.
        pub fn new(policies: BTreeMap<String, Policies>) -> Self {
            Self { policies }
        }
    }

    /// Collection of localized policies.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Policies {
        /// The version for the policy.
        ///
        /// There are no requirements on what this might be and could be
        /// "alpha", semantically versioned, or arbitrary.
        pub version: String,

        /// Available languages for the policy.
        ///
        /// The keys could be the language code corresponding to
        /// the given `LocalizedPolicy`, for example "en" or "fr".
        #[serde(flatten)]
        pub localized: BTreeMap<String, LocalizedPolicy>,
    }

    impl Policies {
        /// Create a new `Policies` with the given version and localized map.
        pub fn new(version: String, localized: BTreeMap<String, LocalizedPolicy>) -> Self {
            Self { version, localized }
        }
    }

    /// A localized policy offered by a server.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct LocalizedPolicy {
        /// The localized name of the policy.
        ///
        /// Examples are "Terms of Service", "Conditions d'utilisation".
        pub name: String,

        /// The URL at which the policy is available.
        ///
        /// Examples are `https://example.org/somewhere/terms-2.0-en.html`
        /// and `https://example.org/somewhere/terms-2.0-fr.html`.
        pub url: String,
    }

    impl LocalizedPolicy {
        /// Create a new `LocalizedPolicy` with the given name and url.
        pub fn new(name: String, url: String) -> Self {
            Self { name, url }
        }
    }
}
