//! `DELETE /_matrix/client/*/room_keys/version/{version}`
//!
//! Delete a backup version.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#delete_matrixclientv3room_keysversionversion
    //!
    //! This deletes a backup version and its room keys.

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: DELETE,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/room_keys/version/:version",
            1.0 => "/_matrix/client/r0/room_keys/version/:version",
            1.1 => "/_matrix/client/v3/room_keys/version/:version",
        }
    };

    /// Request type for the `delete_backup_version` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The backup version to delete.
        #[ruma_api(path)]
        pub version: &'a str,
    }

    /// Response type for the `delete_backup_version` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
