//! [POST /_matrix/client/r0/account/3pid/bind](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-bind)

use ruma_api::ruma_api;

use super::{IdentityServerInfo, IncomingIdentityServerInfo};

ruma_api! {
    metadata: {
        description: "Bind a 3PID to a user's account on an identity server",
        method: POST,
        name: "bind_3pid",
        path: "/_matrix/client/r0/account/3pid/bind",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// Client-generated secret string used to protect this session.
        pub client_secret: &'a str,

        /// The ID server to send the onward request to as a hostname with an
        /// appended colon and port number if the port is not the default.
        #[serde(flatten)]
        pub identity_server_info: IdentityServerInfo<'a>,

        /// The session identifier given by the identity server.
        pub sid: &'a str,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given client secret, identity server information and
    /// session identifier.
    pub fn new(
        client_secret: &'a str,
        identity_server_info: IdentityServerInfo<'a>,
        sid: &'a str,
    ) -> Self {
        Self { client_secret, identity_server_info, sid }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}
