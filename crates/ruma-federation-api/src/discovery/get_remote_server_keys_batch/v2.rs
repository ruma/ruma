//! [POST /_matrix/key/v2/query](https://matrix.org/docs/spec/server_server/r0.1.4#post-matrix-key-v2-query)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_identifiers::{ServerNameBox, ServerSigningKeyId};
use serde::{Deserialize, Serialize};

use crate::discovery::ServerSigningKeys;

ruma_api! {
    metadata: {
        description: "Query for keys from multiple servers in a batch format.",
        method: POST,
        name: "get_remote_server_keys_batch",
        path: "/_matrix/key/v2/query",
        rate_limited: false,
        authentication: None,
    }

    request: {
        /// The time until which the returned certificates will need to be valid to be useful to
        /// the requesting server.
        ///
        /// If not supplied, the current time as determined by the notary server is used.
        #[ruma_api(query)]
        #[serde(default = "MilliSecondsSinceUnixEpoch::now")]
        pub minimum_valid_until_ts: MilliSecondsSinceUnixEpoch,

        /// The query criteria.
        ///
        /// The outer string key on the object is the server name (eg: matrix.org). The inner
        /// string key is the Key ID to query for the particular server. If no key IDs are given to
        /// be queried, the notary server should query for all keys. If no servers are given, the
        /// notary server must return an empty server_keys array in the response.
        ///
        /// The notary server may return multiple keys regardless of the Key IDs given.
        pub server_keys: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, QueryCriteria>>,

    }

    response: {
        /// The queried server's keys, signed by the notary server.
        pub server_keys: Vec<ServerSigningKeys>,
    }
}

impl Request {
    /// Creates a new `Request` with the given query criteria and `minimum_valid_until` timestamp.
    pub fn new(
        server_keys: BTreeMap<ServerNameBox, BTreeMap<ServerSigningKeyId, QueryCriteria>>,
        minimum_valid_until_ts: MilliSecondsSinceUnixEpoch,
    ) -> Self {
        Self { server_keys, minimum_valid_until_ts }
    }
}

impl Response {
    /// Creates a new `Response` with the given keys.
    pub fn new(server_keys: Vec<ServerSigningKeys>) -> Self {
        Self { server_keys }
    }
}

/// The query criteria.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct QueryCriteria {
    /// A millisecond POSIX timestamp in milliseconds indicating when the
    /// returned certificates will need to be valid until to be useful to the
    /// requesting server.
    ///
    /// If not supplied, the current time as determined by the notary server is
    /// used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_valid_until_ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl QueryCriteria {
    /// Creates empty `QueryCriteria`.
    pub fn new() -> Self {
        Default::default()
    }
}
