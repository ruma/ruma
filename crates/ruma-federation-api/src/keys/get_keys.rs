//! `POST /_matrix/federation/*/user/keys/query`
//!
//! Get the current devices and identity keys for the given users.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixfederationv1userkeysquery

    use std::collections::BTreeMap;

    use ruma_common::{
        DeviceId, UserId,
        api::{request, response},
        encryption::{CrossSigningKey, DeviceKeys},
        metadata,
        serde::Raw,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/user/keys/query",
    }

    /// Request type for the `get_keys` endpoint.
    #[request]
    pub struct Request {
        /// The keys to be downloaded.
        ///
        /// Gives all keys for a given user if the list of device ids is empty.
        pub device_keys: BTreeMap<UserId, Vec<DeviceId>>,
    }

    /// Response type for the `get_keys` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// Keys from the queried devices.
        pub device_keys: BTreeMap<UserId, BTreeMap<DeviceId, Raw<DeviceKeys>>>,

        /// Information on the master cross-signing keys of the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub master_keys: BTreeMap<UserId, Raw<CrossSigningKey>>,

        /// Information on the self-signing keys of the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub self_signing_keys: BTreeMap<UserId, Raw<CrossSigningKey>>,
    }

    impl Request {
        /// Creates a new `Request` asking for the given device keys.
        pub fn new(device_keys: BTreeMap<UserId, Vec<DeviceId>>) -> Self {
            Self { device_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given device keys.
        pub fn new(device_keys: BTreeMap<UserId, BTreeMap<DeviceId, Raw<DeviceKeys>>>) -> Self {
            Self { device_keys, ..Default::default() }
        }
    }
}
