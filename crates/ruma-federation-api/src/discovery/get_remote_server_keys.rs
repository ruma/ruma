//! `GET /_matrix/key/*/query/{serverName}`
//!
//! Query for another server's keys. The receiving (notary) server must sign the keys returned by
//! the queried server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixkeyv2queryservername

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        MilliSecondsSinceUnixEpoch, OwnedServerName,
    };

    use crate::discovery::ServerSigningKeys;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/key/v2/query/{server_name}",
        }
    };

    /// Request type for the `get_remote_server_keys` endpoint.
    #[request]
    pub struct Request {
        /// The server's DNS name to query
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// A millisecond POSIX timestamp in milliseconds indicating when the returned certificates
        /// will need to be valid until to be useful to the requesting server.
        ///
        /// If not supplied, the current time as determined by the receiving server is used.
        #[ruma_api(query)]
        #[serde(default = "MilliSecondsSinceUnixEpoch::now")]
        pub minimum_valid_until_ts: MilliSecondsSinceUnixEpoch,
    }

    /// Response type for the `get_remote_server_keys` endpoint.
    #[response]
    pub struct Response {
        /// The queried server's keys, signed by the notary server.
        pub server_keys: Vec<Raw<ServerSigningKeys>>,
    }

    impl Request {
        /// Creates a new `Request` with the given server name and `minimum_valid_until` timestamp.
        pub fn new(
            server_name: OwnedServerName,
            minimum_valid_until_ts: MilliSecondsSinceUnixEpoch,
        ) -> Self {
            Self { server_name, minimum_valid_until_ts }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given keys.
        pub fn new(server_keys: Vec<Raw<ServerSigningKeys>>) -> Self {
            Self { server_keys }
        }
    }
}
