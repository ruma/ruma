//! `GET /.well-known/matrix/policy_server` ([spec])
//!
//! Gets public key information for a [Policy Server].
//!
//! Note that this endpoint is not necessarily handled by the homeserver or Policy Server. It may be
//! served by another webserver.
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#getwell-knownmatrixpolicy_server
//! [Policy Server]: https://spec.matrix.org/latest/client-server-api/#policy-servers

use std::collections::BTreeMap;

use ruma_common::{
    SigningKeyAlgorithm,
    api::{auth_scheme::NoAccessToken, request, response},
    metadata,
    serde::Base64,
};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAccessToken,
    path: "/.well-known/matrix/policy_server",
}

/// Request type for the `discover_policy_server` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {}

/// Response type for the `discover_policy_server` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// The public keys for the Policy Server.
    ///
    /// MUST contain at least `ed25519`.
    pub public_keys: BTreeMap<SigningKeyAlgorithm, Base64>,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given ed25519 public key.
    pub fn new(ed25519_public_key: Base64) -> Self {
        Self { public_keys: [(SigningKeyAlgorithm::Ed25519, ed25519_public_key)].into() }
    }
}
