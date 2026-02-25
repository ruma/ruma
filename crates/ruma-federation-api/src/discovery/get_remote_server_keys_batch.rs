//! `POST /_matrix/key/*/query`
//!
//! Query for keys from multiple servers in a batch format. The receiving (notary) server must sign
//! the keys returned by the queried servers.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#post_matrixkeyv2query

    use std::collections::BTreeMap;

    use ruma_common::{
        MilliSecondsSinceUnixEpoch, ServerName, ServerSigningKeyId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
        serde::Raw,
    };
    use serde::{Deserialize, Serialize};

    use crate::discovery::ServerSigningKeys;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: NoAuthentication,
        path: "/_matrix/key/v2/query",
    }

    /// Request type for the `get_remote_server_keys_batch` endpoint.
    #[request]
    pub struct Request {
        /// The query criteria.
        ///
        /// The outer string key on the object is the server name (eg: matrix.org). The inner
        /// string key is the Key ID to query for the particular server. If no key IDs are given to
        /// be queried, the notary server should query for all keys. If no servers are given, the
        /// notary server must return an empty server_keys array in the response.
        ///
        /// The notary server may return multiple keys regardless of the Key IDs given.
        pub server_keys: BTreeMap<ServerName, BTreeMap<ServerSigningKeyId, QueryCriteria>>,
    }

    /// Response type for the `get_remote_server_keys_batch` endpoint.
    #[response]
    pub struct Response {
        /// The queried server's keys, signed by the notary server.
        pub server_keys: Vec<Raw<ServerSigningKeys>>,
    }

    impl Request {
        /// Creates a new `Request` with the given query criteria.
        pub fn new(
            server_keys: BTreeMap<ServerName, BTreeMap<ServerSigningKeyId, QueryCriteria>>,
        ) -> Self {
            Self { server_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given keys.
        pub fn new(server_keys: Vec<Raw<ServerSigningKeys>>) -> Self {
            Self { server_keys }
        }
    }

    /// The query criteria.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct QueryCriteria {
        /// A millisecond POSIX timestamp in milliseconds indicating when the
        /// returned certificates will need to be valid until to be useful to the
        /// requesting server.
        ///
        /// If not supplied, the current time as determined by the notary server is
        /// used.
        // This doesn't use `serde(default)` because the default would then be
        // determined by the client rather than the server (and it would take more
        // bandwidth because `skip_serializing_if` couldn't be used).
        #[serde(skip_serializing_if = "Option::is_none")]
        pub minimum_valid_until_ts: Option<MilliSecondsSinceUnixEpoch>,
    }

    impl QueryCriteria {
        /// Creates empty `QueryCriteria`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
