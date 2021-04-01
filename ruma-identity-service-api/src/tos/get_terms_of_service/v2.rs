//! [GET /_matrix/identity/v2/terms](https://matrix.org/docs/spec/identity_service/r0.3.0#get-matrix-identity-v2-terms)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

ruma_api! {
        metadata: {
            description: "Gets all the terms of service offered by the server.",
            method: GET,
            name: "get_terms_of_service",
            path: "/_matrix/identity/v2/terms",
            authentication: None,
            rate_limited: false,
        }

        #[derive(Default)]
        request: {}

        response: {
            /// The policies the server offers.
            ///
            /// Mapped from arbitrary ID (unused in this version of the specification) to a Policy Object.
            pub policies: BTreeMap<String, Policies>
        }
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
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

/// A localized policy offered by a server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalizedPolicy {
    /// The localized name of the policy.
    ///
    /// Examples are "Terms of Service", "Conditions d'utilisation".
    pub name: String,

    /// The URL at wich the policy is available.
    ///
    /// Examples are "https://example.org/somewhere/terms-2.0-en.html"
    /// and "https://example.org/somewhere/terms-2.0-fr.html".
    pub url: String,
}
