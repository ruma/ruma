//! [POST /_matrix/client/r0/keys/query](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-keys-query)

use std::{collections::BTreeMap, time::Duration};

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, UserId};
use serde_json::Value as JsonValue;

use super::{CrossSigningKey, DeviceKeys};

ruma_api! {
    metadata {
        description: "Returns the current devices and identity keys for the given users.",
        method: POST,
        name: "get_keys",
        path: "/_matrix/client/r0/keys/query",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The time (in milliseconds) to wait when downloading keys from remote
        /// servers. 10 seconds is the recommended default.
        #[serde(
            with = "ruma_serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        pub timeout: Option<Duration>,

        /// The keys to be downloaded. An empty list indicates all devices for
        /// the corresponding user.
        pub device_keys: BTreeMap<UserId, Vec<DeviceId>>,

        /// If the client is fetching keys as a result of a device update
        /// received in a sync request, this should be the 'since' token of that
        /// sync request, or any later sync token. This allows the server to
        /// ensure its response contains the keys advertised by the notification
        /// in that sync.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub token: Option<String>,
    }

    response {
        /// If any remote homeservers could not be reached, they are recorded
        /// here. The names of the properties are the names of the unreachable
        /// servers.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub failures: BTreeMap<String, JsonValue>,

        /// Information on the queried devices.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub device_keys: BTreeMap<UserId, BTreeMap<DeviceId, DeviceKeys>>,

        /// Information on the master cross-signing keys of the queried users.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub master_keys: BTreeMap<UserId, CrossSigningKey>,

        /// Information on the self-signing keys of the queried users.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub self_signing_keys: BTreeMap<UserId, CrossSigningKey>,

        /// Information on the user-signing keys of the queried users.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub user_signing_keys: BTreeMap<UserId, CrossSigningKey>,
    }

    error: crate::Error
}
