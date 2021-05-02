//! [GET /_matrix/key/v2/query/{serverName}/{keyId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-key-v2-query-servername-keyid)

use std::time::SystemTime;

use ruma_api::ruma_api;
use ruma_identifiers::ServerName;

use crate::discovery::ServerSigningKeys;

ruma_api! {
    metadata: {
        description: "Query for another server's keys.",
        method: GET,
        name: "get_remote_server_keys",
        // Note: The spec has an additional, deprecated path parameter on this. We may want to
        // support an additional parameter at the end, even if it is ignored.
        path: "/_matrix/key/v2/query/:server_name",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The server's DNS name to query
        #[ruma_api(path)]
        pub server_name: &'a ServerName,

        /// A millisecond POSIX timestamp in milliseconds indicating when the returned certificates
        /// will need to be valid until to be useful to the requesting server.
        ///
        /// If not supplied, the current time as determined by the receiving server is used.
        #[ruma_api(query)]
        #[serde(default = "SystemTime::now", with = "ruma_serde::time::ms_since_unix_epoch")]
        pub minimum_valid_until_ts: SystemTime,
    }

    response: {
        /// The queried server's keys, signed by the notary server.
        pub server_keys: Vec<ServerSigningKeys>,
    }
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given server name and `minimum_valid_until` timestamp.
    pub fn new(server_name: &'a ServerName, minimum_valid_until_ts: SystemTime) -> Self {
        Self { server_name, minimum_valid_until_ts }
    }
}

impl Response {
    /// Creates a new `Response` with the given keys.
    pub fn new(server_keys: Vec<ServerSigningKeys>) -> Self {
        Self { server_keys }
    }
}
