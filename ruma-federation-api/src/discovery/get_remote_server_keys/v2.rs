//! [GET /_matrix/key/v2/query/{serverName}/{keyId}](https://matrix.org/docs/spec/server_server/r0.1.4#get-matrix-key-v2-query-servername-keyid)

use std::time::SystemTime;

use crate::discovery::ServerKey;
use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Query for another server's keys.",
        method: GET,
        name: "get_remote_server_keys",
        path: "/_matrix/key/v2/query/:server_name",
        rate_limited: false,
        requires_authentication: false,
    }

    request: {
        /// The server's DNS name to query
        #[ruma_api(path)]
        pub server_name: String,

        /// A millisecond POSIX timestamp in milliseconds indicating when the
        /// returned certificates will need to be valid until to be useful to
        /// the requesting server.
        ///
        /// If not supplied, the current time as determined by the notary server
        /// is used.
        #[ruma_api(query)]
        #[serde(with = "ruma_serde::time::ms_since_unix_epoch")]
        pub minimum_valid_until_ts: SystemTime,
    }

    response: {
        /// The queried server's keys, signed by the notary server.
        pub server_keys: Vec<ServerKey>,
    }
}
