//! `POST /_matrix/client/*/keys/query`
//!
//! Returns the current devices and identity keys for the given users.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3keysquery

    use std::{collections::BTreeMap, time::Duration};

    use ruma_common::{
        api::{request, response, Metadata},
        encryption::{CrossSigningKey, DeviceKeys},
        metadata,
        serde::Raw,
        OwnedDeviceId, OwnedUserId,
    };
    use serde_json::Value as JsonValue;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/keys/query",
            1.1 => "/_matrix/client/v3/keys/query",
        }
    };

    /// Request type for the `get_keys` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {
        /// The time (in milliseconds) to wait when downloading keys from remote servers.
        ///
        /// 10 seconds is the recommended default.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub timeout: Option<Duration>,

        /// The keys to be downloaded.
        ///
        /// An empty list indicates all devices for the corresponding user.
        pub device_keys: BTreeMap<OwnedUserId, Vec<OwnedDeviceId>>,
    }

    /// Response type for the `get_keys` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// If any remote homeservers could not be reached, they are recorded here.
        ///
        /// The names of the properties are the names of the unreachable servers.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub failures: BTreeMap<String, JsonValue>,

        /// Information on the queried devices.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_keys: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Raw<DeviceKeys>>>,

        /// Information on the master cross-signing keys of the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub master_keys: BTreeMap<OwnedUserId, Raw<CrossSigningKey>>,

        /// Information on the self-signing keys of the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub self_signing_keys: BTreeMap<OwnedUserId, Raw<CrossSigningKey>>,

        /// Information on the user-signing keys of the queried users.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub user_signing_keys: BTreeMap<OwnedUserId, Raw<CrossSigningKey>>,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
