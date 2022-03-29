//! `DELETE /_matrix/client/*/room_keys/version/{version}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#delete_matrixclientv3room_keysversionversion
    //!
    //! This deletes a backup version and its room keys.

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Delete a backup version.",
            method: DELETE,
            name: "delete_backup_version",
            unstable_path: "/_matrix/client/unstable/room_keys/version/:version",
            r0_path: "/_matrix/client/r0/room_keys/version/:version",
            stable_path: "/_matrix/client/v3/room_keys/version/:version",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The backup version to delete.
            #[ruma_api(path)]
            pub version: &'a str,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given version, room_id and sessions.
        pub fn new(version: &'a str) -> Self {
            Self { version }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
