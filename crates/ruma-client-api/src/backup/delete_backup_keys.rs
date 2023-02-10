//! `DELETE /_matrix/client/*/room_keys/keys`
//!
//! Delete all keys from a backup.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#delete_matrixclientv3room_keyskeys
    //!
    //! This deletes keys from a backup version, but not the version itself.

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys",
            1.0 => "/_matrix/client/r0/room_keys/keys",
            1.1 => "/_matrix/client/v3/room_keys/keys",
        }
    };

    /// Request type for the `delete_backup_keys` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The backup version from which to delete keys.
        #[ruma_api(query)]
        pub version: String,
    }

    /// Response type for the `delete_backup_keys` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// An opaque string representing stored keys in the backup.
        ///
        /// Clients can compare it with the etag value they received in the request of their last
        /// key storage request.
        pub etag: String,

        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    impl Request {
        /// Creates a new `Request` with the given version.
        pub fn new(version: String) -> Self {
            Self { version }
        }
    }

    impl Response {
        /// Creates an new `Response` with the given etag and count.
        pub fn new(etag: String, count: UInt) -> Self {
            Self { etag, count }
        }
    }
}
