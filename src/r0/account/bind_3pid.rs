//! [POST /_matrix/client/r0/account/3pid/bind](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-account-3pid-bind)

use ruma_api::ruma_api;

use super::IdentityServerInfo;

ruma_api! {
    metadata {
        description: "Bind a 3PID to a user's account on an identity server",
        method: POST,
        name: "bind_3pid",
        path: "/_matrix/client/r0/account/3pid/bind",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: String,
        /// The ID server to send the onward request to as a hostname with an
        /// appended colon and port number if the port is not the default.
        #[serde(flatten)]
        pub identity_server_info: IdentityServerInfo,
        /// The session identifier given by the identity server.
        pub sid: String,
    }

    response {}

    error: crate::Error
}
