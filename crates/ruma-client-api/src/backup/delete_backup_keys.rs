//! `DELETE /_matrix/client/*/room_keys/keys`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#delete_matrixclientv3room_keyskeys
    //!
    //! This deletes keys from a backup version, but not the version itself.

    use js_int::UInt;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        description: "Delete all keys from a backup.",
        method: DELETE,
        name: "delete_backup_keys",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/keys",
            1.0 => "/_matrix/client/r0/room_keys/keys",
            1.1 => "/_matrix/client/v3/room_keys/keys",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The backup version from which to delete keys.
        #[ruma_api(query)]
        pub version: &'a str,
    }

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

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version.
        pub fn new(version: &'a str) -> Self {
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
