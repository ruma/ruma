//! `POST /_matrix/client/*/keys/query`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3keysquery

    use std::{collections::BTreeMap, time::Duration};

    use ruma_common::{
        api::ruma_api,
        encryption::{CrossSigningKey, DeviceKeys},
        serde::Raw,
        OwnedDeviceId, OwnedUserId,
    };
    use serde_json::Value as JsonValue;

    ruma_api! {
        metadata: {
            description: "Returns the current devices and identity keys for the given users.",
            method: POST,
            name: "get_keys",
            r0_path: "/_matrix/client/r0/keys/query",
            stable_path: "/_matrix/client/v3/keys/query",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// The time (in milliseconds) to wait when downloading keys from remote servers.
            ///
            /// 10 seconds is the recommended default.
            #[serde(
                with = "ruma_common::serde::duration::opt_ms",
                default,
                skip_serializing_if = "Option::is_none",
            )]
            pub timeout: Option<Duration>,

            /// The keys to be downloaded.
            ///
            /// An empty list indicates all devices for the corresponding user.
            pub device_keys: BTreeMap<OwnedUserId, Vec<OwnedDeviceId>>,

            /// If the client is fetching keys as a result of a device update received in a sync
            /// request, this should be the 'since' token of that sync request, or any later sync token.
            ///
            /// This allows the server to ensure its response contains the keys advertised by the
            /// notification in that sync.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub token: Option<&'a str>,
        }

        #[derive(Default)]
        response: {
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

        error: crate::Error
    }

    impl Request<'_> {
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
