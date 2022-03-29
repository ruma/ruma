//! `GET /_matrix/key/*/query/{serverName}/{keyId}`
//!
//! Query for another server's keys. The receiving (notary) server must sign the keys returned by
//! the queried server.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixkeyv2queryservernamekeyid

    use ruma_common::{api::ruma_api, serde::Raw, MilliSecondsSinceUnixEpoch, ServerName};

    use crate::discovery::ServerSigningKeys;

    ruma_api! {
        metadata: {
            description: "Query for another server's keys.",
            method: GET,
            name: "get_remote_server_keys",
            // Note: The spec has an additional, deprecated path parameter on this. We may want to
            // support an additional parameter at the end, even if it is ignored.
            stable_path: "/_matrix/key/v2/query/:server_name",
            rate_limited: false,
            authentication: None,
            added: 1.0,
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
            #[serde(default = "MilliSecondsSinceUnixEpoch::now")]
            pub minimum_valid_until_ts: MilliSecondsSinceUnixEpoch,
        }

        response: {
            /// The queried server's keys, signed by the notary server.
            pub server_keys: Vec<Raw<ServerSigningKeys>>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given server name and `minimum_valid_until` timestamp.
        pub fn new(
            server_name: &'a ServerName,
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
