//! [DELETE /_matrix/client/r0/room_keys/version/{version}](https://matrix.org/docs/spec/client_server/unstable#delete-matrix-client-r0-room-keys-version-version)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Delete an existing backup.",
        method: DELETE,
        name: "delete_backup",
        path: "/_matrix/client/r0/room_keys/version/:version",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The backup version.
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
        Self
    }
}
