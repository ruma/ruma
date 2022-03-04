//! `POST /_matrix/client/*/account/3pid/bind`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3account3pidbind

    use ruma_common::{api::ruma_api, ClientSecret, SessionId};

    use crate::account::{IdentityServerInfo, IncomingIdentityServerInfo};

    ruma_api! {
        metadata: {
            description: "Bind a 3PID to a user's account on an identity server",
            method: POST,
            name: "bind_3pid",
            r0_path: "/_matrix/client/r0/account/3pid/bind",
            stable_path: "/_matrix/client/v3/account/3pid/bind",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// Client-generated secret string used to protect this session.
            pub client_secret: &'a ClientSecret,

            /// The ID server to send the onward request to as a hostname with an
            /// appended colon and port number if the port is not the default.
            #[serde(flatten)]
            pub identity_server_info: IdentityServerInfo<'a>,

            /// The session identifier given by the identity server.
            pub sid: &'a SessionId,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given client secret, identity server information and
        /// session identifier.
        pub fn new(
            client_secret: &'a ClientSecret,
            identity_server_info: IdentityServerInfo<'a>,
            sid: &'a SessionId,
        ) -> Self {
            Self { client_secret, identity_server_info, sid }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
