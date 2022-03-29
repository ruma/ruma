//! `DELETE /_matrix/client/*/room_keys/keys`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#delete_matrixclientv3room_keyskeys
    //!
    //! This deletes keys from a backup version, but not the version itself.

    use js_int::UInt;
    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Delete all keys from a backup.",
            method: DELETE,
            name: "delete_backup_keys",
            unstable_path: "/_matrix/client/unstable/room_keys/keys",
            r0_path: "/_matrix/client/r0/room_keys/keys",
            stable_path: "/_matrix/client/v3/room_keys/keys",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version from which to delete keys.
            #[ruma_api(query)]
            pub version: &'a str,
        }

        response: {
            /// An opaque string representing stored keys in the backup.
            ///
            /// Clients can compare it with the etag value they received in the request of their last
            /// key storage request.
            pub etag: String,

            /// The number of keys stored in the backup.
            pub count: UInt,
        }

        error: crate::Error
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
