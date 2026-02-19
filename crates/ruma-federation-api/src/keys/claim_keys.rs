//! `POST /_matrix/federation/*/user/keys/claim`
//!
//! Claim one-time keys for use in pre-key messages.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixfederationv1userkeysclaim

    use std::collections::BTreeMap;

    use ruma_common::{
        DeviceId, OneTimeKeyAlgorithm, OneTimeKeyId, UserId,
        api::{request, response},
        encryption::OneTimeKey,
        metadata,
        serde::Raw,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/user/keys/claim",
    }

    /// Request type for the `claim_keys` endpoint.
    #[request]
    pub struct Request {
        /// The keys to be claimed.
        pub one_time_keys: OneTimeKeyClaims,
    }

    /// Response type for the `claim_keys` endpoint.
    #[response]
    pub struct Response {
        /// One-time keys for the queried devices
        pub one_time_keys: OneTimeKeys,
    }

    impl Request {
        /// Creates a new `Request` with the given one time key claims.
        pub fn new(one_time_keys: OneTimeKeyClaims) -> Self {
            Self { one_time_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given one time keys.
        pub fn new(one_time_keys: OneTimeKeys) -> Self {
            Self { one_time_keys }
        }
    }

    /// A claim for one time keys
    pub type OneTimeKeyClaims = BTreeMap<UserId, BTreeMap<DeviceId, OneTimeKeyAlgorithm>>;

    /// One time keys for use in pre-key messages
    pub type OneTimeKeys =
        BTreeMap<UserId, BTreeMap<DeviceId, BTreeMap<OneTimeKeyId, Raw<OneTimeKey>>>>;
}
